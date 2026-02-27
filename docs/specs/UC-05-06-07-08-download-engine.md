# UC-05/06/07/08: Download-Engine (Start, Fortschritt, Abbruch, Resume)

## Scope

Kern-Download-Logik, die von GUI und CLI gleichermassen konsumiert wird:

- DownloadManager: Orchestrierung paralleler Downloads
- DownloadItem: Per-File-Status und Resume-Logik
- ProgressEvent: Fortschrittsmeldungen via Channel
- FtpClient: Verbindungsaufbau und Datei-Transfer
- Parallelismus via Semaphore
- Cancellation: Global + Per-File via CancellationToken
- Resume: Erkennung lokaler Teildownloads, Fortsetzung ab Offset

## Akzeptanzkriterien (aus ACCEPTANCE-TESTS.md)

- AT-09: FTP-Verbindung herstellen
- AT-10: Datei herunterladen
- AT-11: Parallele Downloads (max_download_threads wird eingehalten)
- AT-12: Fortschrittsanzeige (ProgressEvents monoton steigend)
- AT-13: Download abbrechen (CancellationToken, Dateien bleiben erhalten)
- AT-14: Download fortsetzen (Resume ab letztem Byte via FTP REST)
- AT-15: Bereits vollständige Datei überspringen

## Beteiligte Module

- `core/src/download/manager.rs` — `DownloadManager`, `DownloadResult`
- `core/src/download/item.rs` — `DownloadItem`, `DownloadStatus`, `ResumeAction`
- `core/src/download/progress.rs` — `ProgressEvent`
- `core/src/ftp/client.rs` — `FtpClient`, `ListEntry`
- `core/src/error.rs` — `DownloadError`, `FtpError`
- `core/src/settings.rs` — `AppSettings` (Download-relevante Felder)

## API Design

```rust
// --- DownloadManager ---

pub struct DownloadManager {
    container: SfdlContainer,
    dest_dir: PathBuf,
    max_threads: u32,
    resume_downloads: bool,
    create_package_subfolder: bool,
    ftp_timeout_seconds: u32,
    cancel_token: CancellationToken,
    cancel_rx: mpsc::UnboundedReceiver<Uuid>,
}

/// Ergebnis-Zusammenfassung nach Abschluss aller Downloads.
pub struct DownloadResult {
    pub total_files: u32,
    pub completed: u32,
    pub failed: u32,
    pub cancelled: u32,
    pub skipped: u32,
}

impl DownloadManager {
    /// Erstellt einen neuen DownloadManager.
    /// Returns: (manager, global_cancel_token, per_file_cancel_sender)
    pub fn new(
        container: SfdlContainer,
        settings: &AppSettings,
    ) -> (Self, CancellationToken, mpsc::UnboundedSender<Uuid>);

    /// Führt die gesamte Download-Session aus.
    /// Sendet ProgressEvents an progress_tx.
    pub async fn run(
        self,
        progress_tx: mpsc::UnboundedSender<ProgressEvent>,
    ) -> Result<DownloadResult, DownloadError>;
}

// --- DownloadItem ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    Pending, Running, Completed, Failed, Cancelled, Skipped,
}

#[derive(Debug, Clone, Copy)]
pub enum ResumeAction {
    StartFresh,
    Resume(u64),        // Byte-Offset
    AlreadyComplete,
}

pub struct DownloadItem {
    pub id: Uuid,
    pub file_item: FileItem,
    pub local_path: PathBuf,
    pub status: DownloadStatus,
    pub bytes_downloaded: u64,
    pub error_message: Option<String>,
}

impl DownloadItem {
    /// Erstellt DownloadItem aus FileItem.
    /// Lokaler Pfad: base_dir / [package_name] / directory_path / file_name
    pub fn from_file_item(
        file_item: &FileItem,
        base_dir: &Path,
        package_name: &str,
        create_package_subfolder: bool,
    ) -> Self;

    /// Prüft lokalen Dateistatus für Resume-Entscheidung.
    pub fn check_local_state(&self) -> ResumeAction;

    /// Fortschritt in Prozent (0.0–100.0).
    pub fn progress_percent(&self) -> f64;
}

// --- ProgressEvent ---

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Started { item_id: Uuid, file_name: String, total_bytes: u64 },
    BytesWritten { item_id: Uuid, bytes_delta: u64, total_written: u64 },
    Completed { item_id: Uuid },
    Skipped { item_id: Uuid, file_name: String },
    Failed { item_id: Uuid, error: String },
    Cancelled { item_id: Uuid },
    AllDone { total_files: u32, completed: u32, failed: u32, cancelled: u32, skipped: u32 },
}

// --- FtpClient ---

pub struct FtpClient { stream: AsyncNativeTlsFtpStream }

impl FtpClient {
    /// Verbindet und authentifiziert mit SFDL Connection-Settings.
    pub async fn connect(conn: &Connection, timeout_seconds: u32) -> Result<Self, FtpError>;

    /// Lädt eine Datei mit Streaming, Resume, Progress-Reporting und Cancellation.
    /// Gibt total geschriebene Bytes zurück (inkl. Resume-Offset).
    pub async fn download_file(
        &mut self,
        remote_path: &str,
        local_path: &Path,
        resume_offset: u64,
        item_id: Uuid,
        progress_tx: &mpsc::UnboundedSender<ProgressEvent>,
        cancel_token: &CancellationToken,
    ) -> Result<u64, DownloadError>;

    /// Trennt die FTP-Verbindung.
    pub async fn disconnect(mut self);
}
```

## Implementierungsdetails

### Download-Ablauf (`DownloadManager::run`)

```
1. File-Liste aus Container-Packages aufbauen
   - Direkte FileItems → DownloadItem::from_file_item()
   - BulkFolder → resolve_all_bulk_folders() → DownloadItem::from_file_item()
2. Resume-Check für jede Datei (wenn resume_downloads=true)
   - AlreadyComplete → ProgressEvent::Skipped, Counter++
   - Resume/StartFresh → in Download-Queue
3. Per-File CancellationTokens erstellen (Child-Tokens vom Global-Token)
4. Cancel-Listener-Task spawnen (liest item_ids aus cancel_rx)
5. Parallele Downloads via Semaphore (max_threads)
6. Pro Datei: eigener tokio::spawn Task
7. Nach Abschluss aller Tasks: ProgressEvent::AllDone senden
```

### Parallelismus

- `Semaphore::new(max_threads as usize)` begrenzt gleichzeitige FTP-Verbindungen
- Jeder File-Task wartet auf `sem.acquire()` bevor er verbindet
- Eine FTP-Verbindung pro Datei (Connection-per-File Pattern)

### Cancellation-Architektur

```
Global CancellationToken
├── Per-File Child Token (file_cancel)     ← cancel_rx Listener
├── Per-File Child Token (file_cancel)     ← cancel_rx Listener
└── ...

GUI/CLI Cancel-Button → global_cancel.cancel()  → Alle Files stoppen
GUI Per-File Cancel   → file_cancel_tx.send(id) → Cancel-Listener → child_token.cancel()
CLI Ctrl+C            → global_cancel.cancel()  → Alle Files stoppen
```

- Cancel-Listener-Task: `tokio::select!` über `cancel_rx.recv()` und `global_cancel.cancelled()`
- In `download_file()`: `cancel_token.is_cancelled()` wird nach jedem 32KB-Chunk geprüft
- Abgebrochene Datei bleibt auf Disk (Teildownload für Resume)

### Resume-Logik

```rust
check_local_state():
  Wenn Datei nicht existiert       → StartFresh
  Wenn local_size >= file_size > 0 → AlreadyComplete
  Wenn local_size > 0              → Resume(local_size)
  Sonst                            → StartFresh
```

- Bei `Resume(offset)`: `stream.resume_transfer(offset)` vor `retr_as_stream()`
- Lokale Datei wird im Append-Modus geöffnet
- Bei `StartFresh`: Datei wird mit `File::create()` neu erstellt

### Pfad-Konstruktion

```
base_dir / [package_name] / directory_path (ohne führende /) / file_name
```

- `create_package_subfolder=true`: Package-Name als Unterordner
- `create_package_subfolder=false` oder leerer Package-Name: kein Unterordner
- `directory_path` wird mit `trim_start_matches('/')` normalisiert
- Verzeichnisse werden mit `create_dir_all()` vor dem Download erstellt

### FTP-Verbindung

- suppaftp mit `AsyncNativeTlsFtpStream`
- Passive Mode (Standard für SFDL)
- Auth: `auth_required=true` → Credentials, sonst `anonymous`
- Binary Transfer Mode für Downloads
- TLS noch nicht implementiert — Warnung wenn `ssl_protocol != None`
- Timeout: `tokio::time::timeout()` um `connect()` (0 = kein Timeout)

### Fehler-Isolation

- Einzelne Dateifehler brechen die Session **nicht** ab
- Fehlgeschlagene Datei → `ProgressEvent::Failed`, Status `Failed`
- Andere Dateien laufen weiter
- `JoinError` (Task-Panic) → wird als `Failed` gezählt
- Nur BulkFolder-Auflösungsfehler brechen die Session ab (`DownloadError::Ftp`)

### GUI-Integration

- GUI konsumiert `ProgressEvent` via `mpsc::unbounded_channel`
- Throttling: `BytesWritten` Events werden in 100ms-Intervallen in Signals geschrieben
- Signals: `download_phase`, `file_states`, `global_progress`, `cancel_token`, `file_cancel_tx`
- Phase-Übergänge: `Idle` → `Downloading` → `Done` → (Reset) → `Idle`
- Per-File Cancel: `file_cancel_tx.send(item_id)` vom Cancel-Button in `ProgressPanel`

### CLI-Integration

- CLI konsumiert denselben `ProgressEvent`-Channel
- indicatif `MultiProgress` für Progress-Bars
- Ctrl+C → globaler `CancellationToken`
- Details siehe UC-12 Spec
