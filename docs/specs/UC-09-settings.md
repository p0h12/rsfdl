# UC-09: Einstellungen konfigurieren

## Scope

Persistente Benutzereinstellungen:
- `AppSettings` Struct mit allen konfigurierbaren Feldern
- JSON-Datei-Persistenz im plattformspezifischen Config-Verzeichnis
- GUI: Settings-View mit Eingabefeldern, Browse-Dialog, Save-Button
- CLI: Einstellungen via Kommandozeilen-Parameter pro Aufruf

## Beteiligte Module

- `core/src/settings.rs` — `AppSettings` Struct, `Default`-Impl
- `core/src/settings.rs` — `AppSettings`, `load_settings()`, `save_settings()`, `default_settings_path()`
- `gui/src/views/settings_view.rs` — `SettingsView` Komponente, `save_settings_to_file()`
- `gui/src/state.rs` — `AppState::load_settings_from_file()`, `settings: Signal<AppSettings>`

## API Design

```rust
// --- AppSettings ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_directory: PathBuf,       // Default: dirs::download_dir() oder "."
    pub max_download_threads: u32,         // Default: 3, Range: 1–10
    pub max_retries: u32,                  // Default: 3, Range: 0–10
    pub retry_wait_seconds: u32,           // Default: 10, Range: 1–120
    pub auto_password_list: Vec<String>,   // Default: leer
    pub resume_downloads: bool,            // Default: true
    pub create_package_subfolder: bool,    // Default: true
    pub ftp_timeout_seconds: u32,          // Default: 30, Range: 5–300
}

// --- Persistenz ---

/// Plattformspezifischer Pfad zur Settings-Datei.
/// macOS:   ~/Library/Application Support/rsfdl/settings.json
/// Linux:   ~/.config/rsfdl/settings.json
/// Windows: C:\Users\<user>\AppData\Roaming\rsfdl\settings.json
pub fn default_settings_path() -> PathBuf;

/// Lädt Settings aus JSON-Datei. Gibt Defaults zurück bei fehlender oder ungültiger Datei.
pub fn load_settings(path: &Path) -> AppSettings;

/// Speichert Settings als Pretty-JSON. Erstellt Parent-Directories bei Bedarf.
pub fn save_settings(path: &Path, settings: &AppSettings) -> io::Result<()>;
```

## Implementierungsdetails

### JSON-Format

```json
{
  "download_directory": "/Users/example/Downloads",
  "max_download_threads": 3,
  "max_retries": 3,
  "retry_wait_seconds": 10,
  "auto_password_list": ["pw1", "pw2"],
  "resume_downloads": true,
  "create_package_subfolder": true,
  "ftp_timeout_seconds": 30
}
```

### Laden

- `load_settings()` liest Datei als String, deserialisiert mit `serde_json::from_str()`
- Bei fehlender Datei: `AppSettings::default()` (kein Fehler)
- Bei ungültigem JSON: `AppSettings::default()` + `tracing::warn!()` Log
- `download_directory` Default: `dirs::download_dir()`, Fallback `"."`
- `default_settings_path()` nutzt `dirs::config_dir()`, Fallback `"."`

### Speichern

- `save_settings()` erstellt Parent-Directories mit `create_dir_all()`
- Schreibt Pretty-JSON via `serde_json::to_string_pretty()`
- Atomarität: Direkt `fs::write()` (kein Temp-File-Pattern)

### GUI: Settings-View

**Eingabefelder:**

| Feld | UI-Element | Clamp |
|---|---|---|
| Download Directory | Readonly-Input + Browse-Button | — |
| Max Download Threads | Number-Input | 1–10 |
| Max Retries | Number-Input | 0–10 |
| Retry Wait (seconds) | Number-Input | 1–120 |
| FTP Timeout (seconds) | Number-Input | 5–300 |
| Resume Downloads | Checkbox | — |
| Create Package Subfolder | Checkbox | — |
| Auto Password List | Textarea (1 Passwort pro Zeile) | — |

**Workflow:**
1. Settings-View liest aktuellen State aus `state.settings`
2. Änderungen schreiben direkt in `state.settings.write()`
3. "Save" Button → `save_settings_to_file()` → `save_settings(path, &settings)`
4. "Back" Button → `current_view = AppView::Main`
5. Browse-Dialog: `rfd::AsyncFileDialog::new().pick_folder()`

**Auto-Password-Liste:**
- Textarea mit Newline-Trennung
- Beim Speichern: `lines().map(trim).filter(not_empty).collect()`
- Leere Zeilen werden ignoriert

### GUI: Settings-Laden beim Start

```rust
AppState::new():
  settings = load_settings_from_file()
  → default_settings_path()
  → load_settings(&path)
```

### CLI: Keine persistente Settings

- CLI liest keine Settings-Datei
- Alle Parameter kommen aus Kommandozeilen-Args
- `AppSettings::default()` als Basis, überschrieben durch `-d`, `-t`
