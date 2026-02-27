# UC-09: Einstellungen konfigurieren

## Overview

- **Use Case ID**: UC-09
- **Use Case Name**: Einstellungen konfigurieren
- **Primary Actor**: Benutzer
- **Goal**: App-Einstellungen anzeigen, bearbeiten und persistieren — sowohl über GUI als auch CLI
- **Status**: Draft

## Preconditions

1. App ist gestartet (GUI oder CLI)
2. Config-Verzeichnis ist beschreibbar (oder Default-Fallback auf `"."`)

## Main Success Scenario

### GUI

1. Benutzer öffnet Einstellungen-Dialog
2. System zeigt alle Einstellungen mit aktuellen Werten an
3. Benutzer bearbeitet Felder (siehe Eingabefelder-Tabelle)
4. Benutzer klickt "Save"
5. System speichert Einstellungen als JSON-Datei
6. Benutzer kehrt zur Hauptansicht zurück

### CLI: `config show`

1. Benutzer ruft `rsfdl config show` auf
2. System lädt Einstellungen aus der Settings-Datei (oder Defaults)
3. System gibt den Dateipfad und alle Einstellungen auf stdout aus

### CLI: `config edit`

1. Benutzer ruft `rsfdl config edit` auf
2. Falls Settings-Datei nicht existiert: System erstellt sie mit Default-Werten
3. System öffnet die Datei im Editor (`$EDITOR`, Fallback: `vi` / Windows: `notepad`)
4. Benutzer bearbeitet und speichert die Datei
5. Änderungen werden beim nächsten Laden wirksam

### CLI: Overrides bei Download

- Einstellungen können pro Aufruf per Flag überschrieben werden, ohne die Datei zu ändern
- z.B. `--dest`, `--threads`, `--exclude`, `--retries`, `--timeout`

## Alternative Flows

### A1: Settings-Datei fehlt

**Trigger:** Erster Start oder Datei wurde gelöscht
**Flow:**

1. `load_settings()` gibt `AppSettings::default()` zurück
2. Kein Fehler, App startet normal mit Defaults

### A2: Settings-Datei enthält ungültiges JSON

**Trigger:** Datei wurde manuell falsch bearbeitet
**Flow:**

1. `load_settings()` loggt `tracing::warn!("Failed to parse settings file: {e}")`
2. Default-Werte werden verwendet
3. App startet normal

### A3: Settings-Datei hat unbekannte/fehlende Felder

**Trigger:** Datei stammt von älterer Version (neue Felder fehlen)
**Flow:**

1. `serde(default)` füllt fehlende Felder mit Defaults
2. Unbekannte Felder werden ignoriert
3. Rückwärtskompatibel

### A4: `$EDITOR` nicht gesetzt (CLI)

**Trigger:** `rsfdl config edit` ohne `$EDITOR` Umgebungsvariable
**Flow:**

1. System fällt auf `vi` (Unix) bzw. `notepad` (Windows) zurück
2. Editor wird gestartet

### A5: Config-Verzeichnis nicht beschreibbar

**Trigger:** Fehlende Schreibrechte
**Flow:**

1. `save_settings()` gibt `io::Error` zurück
2. GUI: Fehlermeldung "Einstellungen konnten nicht gespeichert werden"
3. CLI `config edit`: Datei kann nicht erstellt werden → Fehlermeldung

## Postconditions

### Success

- Einstellungen sind in `settings.json` persistiert
- Nächster App-Start lädt die gespeicherten Werte
- CLI-Overrides ändern die Datei nicht

### Failure

- Bei ungültiger/fehlender Datei: Default-Werte werden verwendet
- Kein Crash/Panic in keinem Szenario

## Business Rules

### BR-001: Plattformspezifischer Pfad

| Plattform | Pfad                                                |
|-----------|-----------------------------------------------------|
| macOS     | `~/Library/Application Support/rsfdl/settings.json` |
| Linux     | `~/.config/rsfdl/settings.json`                     |
| Windows   | `%APPDATA%\rsfdl\settings.json`                     |

Fallback wenn `dirs::config_dir()` `None` liefert: `./settings.json`

### BR-002: CLI lädt Settings-Datei

Das `download`-Kommando lädt die Settings-Datei als Basis. CLI-Flags überschreiben einzelne Werte, ohne die Datei zu
ändern. Reihenfolge: `Default → Settings-Datei → CLI-Flags`.

### BR-003: `config show` Output-Format

Menschenlesbares Key-Value-Format auf stdout. Erste Zeile zeigt den Dateipfad:

```
Settings file: /Users/example/.config/rsfdl/settings.json

download_directory       = /Users/example/Downloads
max_download_threads     = 3
max_retries              = 3
retry_wait_seconds       = 10
ftp_timeout_seconds      = 30
resume_downloads         = true
create_package_subfolder = true
auto_extract_archives    = false
delete_archives_after_extraction = false
file_exclusion_patterns  = *.scr, *.lnk, *.nfo
auto_password_list       = (2 entries)
```

Passwörter werden **nicht** im Klartext angezeigt — nur die Anzahl.

### BR-004: `config edit` erstellt Datei bei Bedarf

Falls die Settings-Datei noch nicht existiert, wird sie mit `AppSettings::default()` erstellt, bevor der Editor geöffnet
wird. So hat der Benutzer ein vollständiges JSON als Ausgangslage.

## Beteiligte Module

- `core/src/settings.rs` — `AppSettings`, `load_settings()`, `save_settings()`, `default_settings_path()`
- `gui/src/views/settings_view.rs` — `SettingsView` Komponente
- `gui/src/state.rs` — `AppState::load_settings_from_file()`, `settings: Signal<AppSettings>`
- `cli/src/main.rs` — `Config` Subcommand mit `show` / `edit`
- `cli/src/commands/config.rs` — **Neu**: `run_show()`, `run_edit()`

## API Design

```rust
// --- core/src/settings.rs (bestehend) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_directory: PathBuf,              // Default: dirs::download_dir() oder "."
    pub max_download_threads: u32,                // Default: 3, Range: 1–10
    pub max_retries: u32,                         // Default: 3, Range: 0–10
    pub retry_wait_seconds: u32,                  // Default: 10, Range: 1–120
    pub auto_password_list: Vec<String>,           // Default: leer
    pub resume_downloads: bool,                    // Default: true
    pub create_package_subfolder: bool,            // Default: true
    pub ftp_timeout_seconds: u32,                  // Default: 30, Range: 5–300
    #[serde(default)]
    pub file_exclusion_patterns: Vec<String>,      // Default: ["*.scr", "*.lnk", "*.nfo"]
    #[serde(default)]
    pub auto_extract_archives: bool,               // Default: false
    #[serde(default)]
    pub delete_archives_after_extraction: bool,    // Default: false
}

pub fn default_settings_path() -> PathBuf;
pub fn load_settings(path: &Path) -> AppSettings;
pub fn save_settings(path: &Path, settings: &AppSettings) -> io::Result<()>;
```

```rust
// --- cli/src/main.rs (Ergänzung) ---

#[derive(Subcommand)]
enum Commands {
    // ... bestehende: Info, List, Download ...

    /// Manage settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current settings
    Show,
    /// Edit settings in $EDITOR
    Edit,
}
```

```rust
// --- cli/src/commands/config.rs (NEU) ---

use rsfdl_core::settings::{default_settings_path, load_settings, save_settings, AppSettings};

/// Zeigt alle Einstellungen auf stdout an.
pub fn run_show() {
    let path = default_settings_path();
    let settings = load_settings(&path);
    // Pfad + formatierte Einstellungen ausgeben
}

/// Öffnet die Settings-Datei im Editor.
/// Erstellt die Datei mit Defaults, falls sie nicht existiert.
pub fn run_edit() -> io::Result<()> {
    let path = default_settings_path();
    if !path.exists() {
        save_settings(&path, &AppSettings::default())?;
    }
    // $EDITOR oder Fallback starten
}
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
  "ftp_timeout_seconds": 30,
  "file_exclusion_patterns": ["*.scr", "*.lnk", "*.nfo"],
  "auto_extract_archives": false,
  "delete_archives_after_extraction": false
}
```

### Laden

- `load_settings()` liest Datei als String, deserialisiert mit `serde_json::from_str()`
- Bei fehlender Datei: `AppSettings::default()` (kein Fehler)
- Bei ungültigem JSON: `AppSettings::default()` + `tracing::warn!()` Log
- `download_directory` Default: `dirs::download_dir()`, Fallback `"."`
- `default_settings_path()` nutzt `dirs::config_dir()`, Fallback `"."`
- `#[serde(default)]` auf neueren Feldern für Rückwärtskompatibilität

### Speichern

- `save_settings()` erstellt Parent-Directories mit `create_dir_all()`
- Schreibt Pretty-JSON via `serde_json::to_string_pretty()`
- Atomarität: Direkt `fs::write()` (kein Temp-File-Pattern)

### CLI: Editor starten

```rust
fn open_editor(path: &Path) -> io::Result<()> {
    let editor = std::env::var("EDITOR")
        .unwrap_or_else(|_| {
            if cfg!(windows) { "notepad".into() } else { "vi".into() }
        });
    std::process::Command::new(&editor)
        .arg(path)
        .status()
        .map(|_| ())
}
```

### CLI: Download-Override-Kette

```
AppSettings::default()
  → überschrieben durch load_settings(&path)   // persistierte Werte
  → überschrieben durch CLI-Flags              // --dest, --threads, etc.
```

Die Settings-Datei wird **nicht** verändert. Overrides gelten nur für den aktuellen Aufruf.

### GUI: Settings-View

**Eingabefelder:**

| Feld                             | UI-Element                      | Clamp |
|----------------------------------|---------------------------------|-------|
| Download Directory               | Readonly-Input + Browse-Button  | —     |
| Max Download Threads             | Number-Input                    | 1–10  |
| Max Retries                      | Number-Input                    | 0–10  |
| Retry Wait (seconds)             | Number-Input                    | 1–120 |
| FTP Timeout (seconds)            | Number-Input                    | 5–300 |
| Resume Downloads                 | Checkbox                        | —     |
| Create Package Subfolder         | Checkbox                        | —     |
| Auto-Extract Archives            | Checkbox                        | —     |
| Delete Archives After Extraction | Checkbox                        | —     |
| Auto Password List               | Textarea (1 Passwort pro Zeile) | —     |
| File Exclusion Patterns          | Textarea (1 Muster pro Zeile)   | —     |

**Workflow:**

1. Settings-View liest aktuellen State aus `state.settings`
2. Änderungen schreiben direkt in `state.settings.write()`
3. "Save" Button → `save_settings_to_file()` → `save_settings(path, &settings)`
4. "Back" Button → `current_view = AppView::Main`
5. Browse-Dialog: `rfd::AsyncFileDialog::new().pick_folder()`

## Akzeptanzkriterien (aus acceptance-tests.md)

- AT-39: CLI `config show` zeigt Einstellungen an
- AT-40: CLI `config show` mit Defaults (keine Datei)
- AT-41: CLI `config edit` öffnet Editor
- AT-42: CLI Download Override ändert Datei nicht
- AT-43: Korrupte Settings-Datei fällt auf Defaults zurück
