# UC-14: Archive automatisch entpacken

## Overview

- **Use Case ID**: UC-14
- **Use Case Name**: Archive automatisch entpacken
- **Primary Actor**: System (automatisch nach Download)
- **Goal**: Heruntergeladene RAR- und ZIP-Archive automatisch entpacken, um dem Benutzer manuelle Extraktion zu ersparen
- **Status**: Draft
- **Priority**: SHOULD
- **Phase**: P2
- **Requirement**: FR-16

## Preconditions

1. Alle Dateien eines Pakets sind erfolgreich heruntergeladen (oder: alle Downloads einer Session abgeschlossen)
2. Auto-Extraktion ist in den Einstellungen aktiviert (`auto_extract_archives = true`)
3. Download-Verzeichnis enthält Archiv-Dateien (RAR oder ZIP)

## Main Success Scenario

1. `DownloadManager::run()` beendet alle Downloads und sendet `ProgressEvent::AllDone`
2. System prüft ob `auto_extract_archives` aktiviert ist
3. System scannt das Download-Verzeichnis (pro Paket) nach Archiv-Dateien
4. **RAR Multi-Part**: System identifiziert den ersten Teil (`.rar`, `.part01.rar`) und ignoriert Folge-Teile
5. **ZIP**: System identifiziert `.zip`-Dateien
6. System startet Extraktion in das selbe Verzeichnis (neben die Archive)
7. Fortschritt wird per `ProgressEvent::Extraction*`-Events gemeldet
8. Nach erfolgreicher Extraktion: Optional Archive löschen (wenn `delete_archives_after_extraction = true`)
9. System sendet `ProgressEvent::ExtractionAllDone` mit Zusammenfassung

## Alternative Flows

### A1: Kein Archiv erkannt

**Trigger:** Keine RAR- oder ZIP-Dateien im Download-Verzeichnis gefunden
**Flow:**

1. Keine Extraktion findet statt
2. `ProgressEvent::ExtractionAllDone { total: 0, ... }` wird gesendet
3. Download gilt als normal abgeschlossen

### A2: Extraktion schlägt fehl (beschädigtes Archiv)

**Trigger:** `unrar` oder `zip`-Crate meldet Fehler (korrupte Daten, unvollständiges Archiv)
**Flow:**

1. System sendet `ProgressEvent::ExtractionFailed { path, error }`
2. Archivdateien bleiben erhalten (werden nicht gelöscht)
3. Extraktion weiterer Archive wird fortgesetzt (ein fehlerhaftes Archiv blockiert nicht die anderen)
4. `ExtractionAllDone` enthält die Anzahl fehlgeschlagener Extraktionen

### A3: Passwortgeschütztes Archiv

**Trigger:** Archiv verlangt ein Passwort
**Flow:**

1. System erkennt Password-Requirement beim Öffnen des Archivs
2. System sendet `ProgressEvent::ExtractionFailed { error: "Passwort-geschütztes Archiv" }`
3. Archivdateien bleiben erhalten
4. Extraktion weiterer Archive wird fortgesetzt

### A4: Feature ist deaktiviert

**Trigger:** `auto_extract_archives = false` (Standard)
**Flow:**

1. Nach `AllDone` findet keine Extraktion statt
2. Kein `Extraction*`-Event wird gesendet

### A5: Fehlende Teile bei Multi-Part-RAR

**Trigger:** Erster Teil (`.part01.rar`) vorhanden, aber nicht alle Folge-Teile
**Flow:**

1. UnRAR-Bibliothek meldet Fehler beim Entpacken
2. Behandlung wie A2 (Fehler, Dateien bleiben erhalten)

### A6: Verschachtelte Archive

**Trigger:** Extrahierter Inhalt enthält weitere Archive
**Flow:**

1. Keine rekursive Extraktion (Scope-Begrenzung für Phase 2)
2. Verschachtelte Archive bleiben als Dateien liegen

## Postconditions

### Success Postconditions

- Archiv-Inhalt liegt entpackt im Download-Verzeichnis neben den Archivdateien
- Bei aktiviertem `delete_archives_after_extraction`: Alle Teile des Archivs sind gelöscht
- `ExtractionResult` enthält korrekte Statistiken (extrahiert, fehlgeschlagen)
- UI ist über den Extraktionsfortschritt informiert (via ProgressEvents)

### Failure Postconditions

- Bei fehlgeschlagener Extraktion: Archivdateien bleiben unverändert erhalten
- Fehler wird gemeldet, beeinflusst aber nicht den Download-Erfolg
- Andere Archive werden trotzdem extrahiert

## Business Rules

### BR-001: Multi-Part-RAR Erkennung

Der erste Teil eines Multi-Part-RAR-Archivs wird anhand folgender Muster erkannt:

| Muster                      | Beispiel            | Beschreibung                                     |
|-----------------------------|---------------------|--------------------------------------------------|
| `*.rar` (ohne `.partN.rar`) | `movie.rar`         | Standalone RAR oder erster Teil (altes Schema)   |
| `*.part01.rar`              | `movie.part01.rar`  | Erster Teil (neues Schema, variable Stellenzahl) |
| `*.part001.rar`             | `movie.part001.rar` | Erster Teil mit 3 Stellen                        |
| `*.part1.rar`               | `movie.part1.rar`   | Erster Teil mit 1 Stelle                         |

Folge-Teile (`.part02.rar`, `.r00`, `.r01`, etc.) werden **nicht** als Einstiegspunkt verwendet. Sie werden nur beim Löschen nach Extraktion berücksichtigt.

**Regex für Hauptdatei:**

```
(?i)^(?!.*\.part(?!0*1\.rar$)\d+\.rar$).*\.(rar|r0*1)$
```

### BR-002: RAR-Teile für Löschung

Alle Dateien, die zum selben Archiv gehören, werden nach erfolgreicher Extraktion gelöscht:

| Muster             | Beispiel                                   |
|--------------------|--------------------------------------------|
| `*.rar`            | `movie.rar`                                |
| `*.part[0-9]+.rar` | `movie.part01.rar`, `movie.part02.rar`     |
| `*.r[0-9]{2}`      | `movie.r00`, `movie.r01`, ..., `movie.r99` |

### BR-003: Extraktion-Zielverzeichnis

Archive werden in das selbe Verzeichnis entpackt, in dem sie liegen (neben den Archivdateien). Es wird kein zusätzliches Unterverzeichnis erstellt.

### BR-004: Überschreib-Verhalten

Bereits existierende Dateien werden beim Entpacken überschrieben (`overwrite = true`). Dies ist konsistent mit dem Verhalten aller Referenz-Implementierungen.

### BR-005: Keine rekursive Extraktion

Verschachtelte Archive (Archive innerhalb von Archiven) werden nicht rekursiv entpackt. Dies vereinfacht die Implementierung und kann in einer späteren Phase ergänzt werden.

### BR-006: Nicht-blockierend

Die Extraktion ist ein separater Schritt nach dem Download. Ein fehlgeschlagenes Entpacken beeinflusst nicht den Rückgabewert von `DownloadManager::run()` — der Download war trotzdem erfolgreich. Die Extraktion meldet ihren eigenen Erfolg/Misserfolg.

### BR-007: Standard-Einstellung

Auto-Extraktion ist standardmässig **deaktiviert** (`auto_extract_archives: false`). Ebenso Archiv-Löschung (`delete_archives_after_extraction: false`).

## Beteiligte Module

- `core/src/extraction/mod.rs` — **Neues Modul**: `ExtractionManager`, Archive-Erkennung, Extraktion
- `core/src/extraction/detector.rs` — **Neu**: Archiv-Erkennung, Multi-Part-Gruppierung
- `core/src/extraction/rar.rs` — **Neu**: RAR-Extraktion via `unrar` Crate
- `core/src/extraction/zip.rs` — **Neu**: ZIP-Extraktion via `zip` Crate
- `core/src/download/progress.rs` — Neue `Extraction*`-Varianten für `ProgressEvent`
- `core/src/settings.rs` — Neue Felder `auto_extract_archives`, `delete_archives_after_extraction`
- `gui/src/views/main_view.rs` — Extraktions-Fortschritt anzeigen
- `gui/src/views/settings_view.rs` — Checkboxen für Extraktions-Settings
- `cli/src/main.rs` — Extraktions-Fortschritt in indicatif-Bars anzeigen

## Akzeptanzkriterien (aus acceptance-tests.md)

- **AT-21**: Auto-Extraktion RAR Multi-Part — Paket mit `archive.part01.rar`, `.part02.rar`, `.part03.rar` wird nach Download automatisch extrahiert
- **AT-22**: Auto-Extraktion ZIP — `files.zip` wird nach Download entpackt
- **AT-23**: Auto-Extraktion deaktiviert — Bei deaktivierter Einstellung findet keine Extraktion statt

## API Design

```rust
// --- core/src/settings.rs (Erweiterung) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // ... bestehende Felder ...

    /// Automatisch Archive entpacken nach Download-Abschluss.
    /// Standard: false (deaktiviert).
    #[serde(default)]
    pub auto_extract_archives: bool,

    /// Archive nach erfolgreicher Extraktion löschen.
    /// Standard: false (Archive bleiben erhalten).
    #[serde(default)]
    pub delete_archives_after_extraction: bool,
}

// --- core/src/download/progress.rs (Erweiterung) ---

#[derive(Debug, Clone)]
pub enum ProgressEvent {
    // ... bestehende Varianten ...

    /// Extraktion eines Archivs gestartet.
    ExtractionStarted {
        archive_path: PathBuf,
        archive_name: String,
    },
    /// Fortschritt der Extraktion eines Archivs.
    ExtractionProgress {
        archive_path: PathBuf,
        percent: u8, // 0–100
    },
    /// Extraktion eines Archivs erfolgreich abgeschlossen.
    ExtractionCompleted {
        archive_path: PathBuf,
    },
    /// Extraktion eines Archivs fehlgeschlagen.
    ExtractionFailed {
        archive_path: PathBuf,
        error: String,
    },
    /// Alle Extraktionen abgeschlossen.
    ExtractionAllDone {
        total_archives: u32,
        extracted: u32,
        failed: u32,
    },
}

// --- core/src/extraction/mod.rs (NEU) ---

pub mod detector;
pub mod rar;
pub mod zip;

use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use crate::download::progress::ProgressEvent;

/// Ergebnis einer Extraktionssession.
pub struct ExtractionResult {
    pub total_archives: u32,
    pub extracted: u32,
    pub failed: u32,
}

/// Erkennt und extrahiert Archive in einem Verzeichnis.
/// Sendet ProgressEvents für Fortschrittsanzeige.
///
/// Wird nach abgeschlossenem Download aufgerufen.
/// Fehler sind nicht-fatal — der Rückgabewert ist immer Ok.
pub async fn extract_archives(
    directory: &Path,
    delete_after: bool,
    progress_tx: &mpsc::UnboundedSender<ProgressEvent>,
) -> ExtractionResult;

// --- core/src/extraction/detector.rs (NEU) ---

use std::path::{Path, PathBuf};

/// Ein erkanntes Archiv mit allen zugehörigen Dateien.
#[derive(Debug, Clone)]
pub struct DetectedArchive {
    /// Pfad zur Hauptdatei (erster Teil bei Multi-Part).
    pub main_file: PathBuf,
    /// Archivtyp.
    pub archive_type: ArchiveType,
    /// Alle Dateien, die zu diesem Archiv gehören (inkl. main_file).
    /// Relevant für Löschung nach Extraktion.
    pub all_parts: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    Rar,
    Zip,
}

/// Scannt ein Verzeichnis (nicht-rekursiv) nach Archiven.
/// Gruppiert Multi-Part-RAR-Dateien zu einem einzigen `DetectedArchive`.
/// Gibt nur Hauptdateien zurück (kein `.part02.rar`, `.r00`, etc.).
pub fn detect_archives(directory: &Path) -> Vec<DetectedArchive>;

/// Prüft ob ein Dateiname der erste Teil eines Multi-Part-RAR ist
/// oder ein Standalone-RAR.
pub fn is_main_rar(file_name: &str) -> bool;

/// Prüft ob ein Dateiname ein Teil eines RAR-Archivs ist
/// (Haupt- oder Folge-Teil). Für Löschlogik.
pub fn is_rar_part(file_name: &str) -> bool;

/// Findet alle RAR-Teile, die zur selben Archivgruppe gehören.
/// Basiert auf dem gemeinsamen Basisnamen.
pub fn find_related_rar_parts(main_file: &Path, directory: &Path) -> Vec<PathBuf>;

// --- core/src/extraction/rar.rs (NEU) ---

use std::path::Path;

/// Extrahiert ein RAR-Archiv (inkl. Multi-Part) in das Zielverzeichnis.
/// Bei Multi-Part: main_file ist der erste Teil (.rar oder .part01.rar).
///
/// Gibt Fortschritt als Prozent (0–100) über den Callback zurück.
pub fn extract_rar(
    main_file: &Path,
    dest_dir: &Path,
    on_progress: impl Fn(u8),
) -> Result<(), ExtractionError>;

// --- core/src/extraction/zip.rs (NEU) ---

use std::path::Path;

/// Extrahiert eine ZIP-Datei in das Zielverzeichnis.
///
/// Gibt Fortschritt als Prozent (0–100) über den Callback zurück.
pub fn extract_zip(
    zip_file: &Path,
    dest_dir: &Path,
    on_progress: impl Fn(u8),
) -> Result<(), ExtractionError>;

// --- core/src/error.rs (Erweiterung) ---

/// Fehler bei der Archiv-Extraktion.
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("RAR extraction failed: {0}")]
    Rar(String),

    #[error("ZIP extraction failed: {0}")]
    Zip(String),

    #[error("Password-protected archive")]
    PasswordProtected,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Implementierungsdetails

### Crate-Abhängigkeiten

```toml
# core/Cargo.toml
[dependencies]
unrar = "0.5"     # RAR-Extraktion (FFI-Binding zu UnRAR-Bibliothek)
zip = "2"         # ZIP-Extraktion (reine Rust-Implementierung)
regex = "1"       # Bereits vorhanden, für Archiv-Erkennung
```

**Hinweis**: Das `unrar` Crate benötigt die UnRAR-Bibliothek auf dem System:

- macOS: `brew install unrar` oder bundled
- Linux: `apt install unrar` / `dnf install unrar`
- Windows: `unrar.dll` muss im PATH oder neben der Binary liegen

### Archiv-Erkennung (detector.rs)

```rust
use regex::Regex;
use std::fs;

pub fn detect_archives(directory: &Path) -> Vec<DetectedArchive> {
    let mut archives = Vec::new();

    let entries: Vec<_> = fs::read_dir(directory)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().is_some_and(|ft| ft.is_file()))
        .collect();

    // 1. Finde ZIP-Dateien
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_lowercase().ends_with(".zip") {
            archives.push(DetectedArchive {
                main_file: entry.path(),
                archive_type: ArchiveType::Zip,
                all_parts: vec![entry.path()],
            });
        }
    }

    // 2. Finde RAR-Hauptdateien und gruppiere Teile
    for entry in &entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if is_main_rar(&name) {
            let parts = find_related_rar_parts(&entry.path(), directory);
            archives.push(DetectedArchive {
                main_file: entry.path(),
                archive_type: ArchiveType::Rar,
                all_parts: parts,
            });
        }
    }

    archives
}

pub fn is_main_rar(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    // Standalone .rar (not .partN.rar where N > 1)
    // oder .part01.rar / .part001.rar / .part1.rar (erster Teil)
    lazy_static! {
        static ref MAIN_RAR: Regex = Regex::new(
            r"(?i)^(?!.*\.part(?!0*1\.rar$)\d+\.rar$).*\.rar$"
        ).unwrap();
    }
    MAIN_RAR.is_match(file_name)
}

pub fn is_rar_part(file_name: &str) -> bool {
    lazy_static! {
        static ref RAR_PART: Regex = Regex::new(
            r"(?i)\.(rar|part\d+\.rar|r\d{2,3})$"
        ).unwrap();
    }
    RAR_PART.is_match(file_name)
}
```

### RAR-Extraktion (rar.rs)

```rust
use unrar::Archive;

pub fn extract_rar(
    main_file: &Path,
    dest_dir: &Path,
    on_progress: impl Fn(u8),
) -> Result<(), ExtractionError> {
    let archive = Archive::new(main_file)
        .open_for_processing()
        .map_err(|e| ExtractionError::Rar(e.to_string()))?;

    let mut current = Some(archive);
    while let Some(header) = current.take() {
        let entry = match header.read_header() {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(e) => return Err(ExtractionError::Rar(e.to_string())),
        };

        current = Some(
            entry
                .extract_to(dest_dir)
                .map_err(|e| {
                    if e.to_string().contains("password") {
                        ExtractionError::PasswordProtected
                    } else {
                        ExtractionError::Rar(e.to_string())
                    }
                })?,
        );
    }

    on_progress(100);
    Ok(())
}
```

**Hinweis**: Das `unrar`-Crate bietet keinen granularen Progress-Callback pro Byte. Fortschritt kann nur pro extrahierter Datei (Header) geschätzt werden. Für Phase 2 ist file-level Progress ausreichend.

### ZIP-Extraktion (zip.rs)

```rust
use zip::ZipArchive;
use std::fs;
use std::io;

pub fn extract_zip(
    zip_file: &Path,
    dest_dir: &Path,
    on_progress: impl Fn(u8),
) -> Result<(), ExtractionError> {
    let file = fs::File::open(zip_file)
        .map_err(ExtractionError::Io)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| ExtractionError::Zip(e.to_string()))?;

    let total = archive.len();
    for i in 0..total {
        let mut entry = archive.by_index(i)
            .map_err(|e| ExtractionError::Zip(e.to_string()))?;

        let out_path = dest_dir.join(entry.mangled_name());

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;
        }

        let percent = ((i + 1) as f64 / total as f64 * 100.0) as u8;
        on_progress(percent);
    }

    Ok(())
}
```

### Integration: Aufruf nach Download

Die Extraktion wird **nach** `DownloadManager::run()` als separater Schritt aufgerufen. Sie ist nicht Teil des Download-Managers, sondern wird vom Aufrufer (CLI oder GUI) orchestriert:

**CLI:**

```rust
// In cli/src/main.rs, nach download_manager.run():
let result = manager.run(progress_tx.clone()).await?;

if settings.auto_extract_archives && result.completed > 0 {
    let extraction = extract_archives(
        &settings.download_directory,
        settings.delete_archives_after_extraction,
        &progress_tx,
    ).await;
    // Extraction-Events werden via progress_tx an die indicatif-Bars gemeldet
}
```

**GUI:**

```rust
// In gui/src/views/main_view.rs, nach AllDone-Event:
ProgressEvent::AllDone { .. } => {
    if state.settings.read().auto_extract_archives {
        let dir = state.settings.read().download_directory.clone();
        let delete = state.settings.read().delete_archives_after_extraction;
        let tx = progress_tx.clone();
        tokio::spawn(async move {
            extract_archives(&dir, delete, &tx).await;
        });
    }
}
```

### Archiv-Löschung nach Extraktion

```rust
fn delete_archive_parts(archive: &DetectedArchive) -> Result<(), std::io::Error> {
    for part in &archive.all_parts {
        if part.exists() {
            std::fs::remove_file(part)?;
            tracing::info!(?part, "Deleted archive part");
        }
    }
    Ok(())
}
```

Die Löschung erfolgt nur nach **erfolgreicher** Extraktion. Bei Fehlern bleiben alle Teile erhalten.

### Settings-Persistenz

Neue Felder in `settings.json`:

```json
{
  "auto_extract_archives": false,
  "delete_archives_after_extraction": false,
  ...
}
```

`#[serde(default)]` stellt Kompatibilität mit bestehenden Settings-Dateien sicher.

### Entscheidungen und Trade-offs

| Entscheidung              | Gewählt                         | Alternative                | Begründung                                                   |
|---------------------------|---------------------------------|----------------------------|--------------------------------------------------------------|
| Extraktionsbibliothek RAR | `unrar` Crate (FFI)             | `unrar` CLI-Tool           | Kein externer Prozess nötig, bessere Fehlerbehandlung        |
| Extraktionsbibliothek ZIP | `zip` Crate (pure Rust)         | —                          | Kein nativer Dependency, gut maintained                      |
| Trigger                   | Nach `AllDone` (alle Downloads) | Per-Paket                  | Einfacher, weniger Komplexität, Referenz-Impls machen beides |
| Zielverzeichnis           | Neben den Archiven              | Separates Unterverzeichnis | Konsistent mit allen Referenz-Impls                          |
| Rekursive Extraktion      | Nein                            | Ja (wie goSFDLSauger)      | Scope-Begrenzung Phase 2, kann später ergänzt werden         |
| Passwort-Support          | Fehler melden                   | Auto-Crack (wie SFDL.NET)  | Zu komplex für Phase 2, kann in Phase 3 ergänzt werden       |
| Fortschrittsgranularität  | Pro Datei im Archiv             | Pro Byte                   | `unrar` Crate limitiert auf File-Level                       |
