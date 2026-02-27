# UC-03: Container-Inhalt anzeigen

## Scope

Anzeige des geparsten (und ggf. entschlüsselten) SFDL-Containers:

- Container-Header: Beschreibung, Uploader, Server, Port
- Paketliste mit Dateien: Name, Pfad, Grösse
- BulkFolder-Auflösung via FTP (rekursives Directory-Listing)
- Gesamtgrösse und Dateianzahl

## Akzeptanzkriterien (aus ACCEPTANCE-TESTS.md)

- AT-17: BulkFolder-Auflösung (3 Dateien in /release/ → 3 FileItems)

## Beteiligte Module

- `core/src/ftp/listing.rs` — `resolve_bulk_folder()`, `resolve_container_bulk_folders()`, `resolve_all_bulk_folders()`, `recursive_list()`, `normalize_path()`
- `core/src/ftp/client.rs` — `FtpClient::list_dir()`, `ListEntry`
- `gui/src/components/container_info.rs` — `ContainerInfo` Komponente
- `gui/src/components/header.rs` — `finish_container_load()`, `resolve_bulk_folders_async()`
- `gui/src/state.rs` — `AppState::all_files()`, `total_size()`, `bulk_folder_count()`, `resolving_bulk_folders`
- `cli/src/commands/list.rs` — List-Befehl mit `--resolve` Flag

## API Design

```rust
// --- BulkFolder-Auflösung ---

/// Löst einen einzelnen BulkFolder auf, indem das FTP-Verzeichnis rekursiv gelistet wird.
pub async fn resolve_bulk_folder(
    conn: &Connection,
    bulk: &BulkFolder,
) -> Result<Vec<FileItem>, FtpError>;

/// Löst alle BulkFolders aller Packages eines Containers auf.
/// Für Preview-Zwecke (vor dem Download).
pub async fn resolve_container_bulk_folders(
    conn: &Connection,
    packages: &[Package],
) -> Result<Vec<FileItem>, FtpError>;

/// Löst alle BulkFolders sequentiell auf (eine Verbindung pro Folder).
pub async fn resolve_all_bulk_folders(
    conn: &Connection,
    folders: &[BulkFolder],
) -> Result<Vec<FileItem>, FtpError>;

// --- FTP Directory Listing ---

pub struct ListEntry {
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
}

/// Listet Verzeichnisinhalt, parst suppaftp LIST-Output.
pub async fn list_dir(&mut self, path: &str) -> Result<Vec<ListEntry>, FtpError>;

// --- GUI State Helpers ---

/// Flat-Liste aller FileItems über alle Packages.
pub fn all_files(&self) -> Vec<FileItem>;

/// Gesamtgrösse aller Dateien in Bytes.
pub fn total_size(&self) -> u64;

/// Anzahl unaufgelöster BulkFolders.
pub fn bulk_folder_count(&self) -> usize;
```

## Implementierungsdetails

### Rekursive FTP-Verzeichnislistung

```
recursive_list(client, path, bulk, items):
  entries = client.list_dir(path)
  für jeden entry (ausser "." und ".."):
    full = normalize_path(path + "/" + entry.name)
    wenn entry.is_directory:
      recursive_list(client, full, bulk, items)   // Rekursion
    sonst:
      items.push(FileItem {
        file_name: entry.name,
        directory_root: bulk.bulk_folder_path,
        directory_path: path,
        full_path: full,
        file_size: entry.size,
        hash_type: None,
        package_name: bulk.package_name,
      })
```

- Rekursion via `Box::pin` (async recursive function)
- `normalize_path()` entfernt doppelte Slashes (`//` → `/`)
- `ListEntry` wird aus `suppaftp::list::File::try_from()` geparst
- Eine FTP-Verbindung pro BulkFolder (sequentiell)

### GUI: Container-Anzeige

**ContainerInfo-Komponente** zeigt:

- Beschreibung (fett), Server:Port, Uploader
- Files: `selected_count/file_count`
- Bulk Folders: Anzahl (nur wenn > 0)
- Selected: `format_bytes(selected_size) / format_bytes(total_size)`
- Spinner mit "Resolving bulk folders via FTP..." während Auflösung

### GUI: BulkFolder-Auflösung

```
finish_container_load(state, container, path):
  1. State zurücksetzen (container, selected_files, download_phase, etc.)
  2. selected_files = vec![true; file_count]
  3. Async-Task spawnen: resolve_bulk_folders_async()

resolve_bulk_folders_async(state):
  1. Prüfen ob BulkFolders vorhanden
  2. resolving_bulk_folders = true
  3. resolve_container_bulk_folders(conn, packages)
  4. Resolved Files nach package_name gruppieren
  5. Zu jeweiligen Packages hinzufügen (file_list.extend)
  6. bulk_folder_list leeren, bulk_folder_mode = false
  7. selected_files neu initialisieren (vec![true; new_count])
  8. resolving_bulk_folders = false
```

- Clone-Pattern für Signal-Borrow: `let mut container = { state.container.read().clone() };`
- Fehler bei Auflösung: `error_message` Signal setzen, kein Abbruch

### CLI: List-Befehl

```
rsfdl-cli list <datei.sfdl> [-p <password>] [-r|--resolve]
```

- Ohne `--resolve`: BulkFolders werden als `[DIR] /path/` angezeigt
- Mit `--resolve`: FTP-Verbindung, rekursives Listing, Dateien mit Grössen
- Output-Format: Package-Gruppierung, Dateiname, Grösse, Hash-Typ
- Am Ende: Gesamtanzahl und Gesamtgrösse
