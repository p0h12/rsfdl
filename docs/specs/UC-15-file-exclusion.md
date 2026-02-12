# UC-15: Dateien per Muster ausschliessen

## Overview

- **Use Case ID**: UC-15
- **Use Case Name**: Dateien per Muster ausschliessen
- **Primary Actor**: Benutzer
- **Goal**: Unerwünschte Dateien automatisch vom Download ausschliessen (z.B. `.nfo`, `.jpg`, Samples)
- **Status**: Draft

## Preconditions

1. Container ist geparst und (ggf.) entschlüsselt — Dateiliste ist verfügbar
2. AppSettings sind geladen (enthalten ggf. gespeicherte Ausschluss-Muster)

## Main Success Scenario

1. System lädt Ausschluss-Muster aus `AppSettings.file_exclusion_patterns`
2. System prüft jeden `FileItem.file_name` gegen alle Muster (Glob-Matching, case-insensitive)
3. Dateien, die einem Muster entsprechen, werden mit `DownloadStatus::Excluded` markiert
4. GUI zeigt ausgeschlossene Dateien ausgegraut in der Dateiliste an
5. Gesamtgrösse und Dateianzahl werden ohne ausgeschlossene Dateien berechnet
6. Beim Download-Start werden ausgeschlossene Dateien übersprungen
7. Benutzer kann Ausschluss manuell übersteuern (Datei per Checkbox wieder auswählen)

## Alternative Flows

### A1: Keine Muster konfiguriert

**Trigger:** `file_exclusion_patterns` ist leer
**Flow:**
1. Keine Dateien werden ausgeschlossen
2. Alle Dateien sind standardmässig ausgewählt (wie bisher)

### A2: Alle Dateien ausgeschlossen

**Trigger:** Alle Dateien im Container entsprechen einem Ausschluss-Muster
**Flow:**
1. System zeigt Warnung: "Alle Dateien sind ausgeschlossen"
2. Kein Download wird gestartet
3. Benutzer kann Muster anpassen oder einzelne Dateien manuell auswählen

### A3: CLI mit --exclude Parameter

**Trigger:** Benutzer gibt `--exclude <pattern>` an
**Flow:**
1. CLI-Muster werden zusätzlich zu gespeicherten Mustern angewandt
2. Ausgeschlossene Dateien werden als "skipped" im Output markiert
3. Nur nicht-ausgeschlossene Dateien werden heruntergeladen

### A4: Ungültiges Glob-Muster

**Trigger:** Ein Muster in der Liste ist kein gültiger Glob-Ausdruck
**Flow:**
1. System loggt Warnung: "Invalid exclusion pattern: {pattern}"
2. Ungültiges Muster wird übersprungen
3. Restliche Muster werden normal angewandt

## Postconditions

### Success Postconditions

- Ausgeschlossene Dateien haben Status `Excluded` und werden nicht heruntergeladen
- Nicht-ausgeschlossene Dateien werden normal heruntergeladen
- Gesamtgrösse der Auswahl ist korrekt berechnet (ohne ausgeschlossene Dateien)

### Failure Postconditions

- Bei ungültigen Mustern: Warnung im Log, restliche Muster gelten
- Keine Dateien gehen verloren — Ausschluss verhindert nur den Download

## Business Rules

### BR-001: Glob-Matching auf Dateiname

Muster werden nur gegen `FileItem.file_name` geprüft, nicht gegen den vollen Pfad.
Case-insensitive Matching (`.NFO` wird von `*.nfo` erfasst).

### BR-002: Muster-Syntax

Glob-Syntax mit `*` (beliebige Zeichen) und `?` (ein Zeichen):
- `*.nfo` — alle `.nfo`-Dateien
- `*.jpg` — alle JPEG-Dateien
- `*sample*` — alles mit "sample" im Namen
- `*.r[0-9][0-9]` — veraltete RAR-Teile (`.r00`, `.r01`, ...)

### BR-003: Priorität von Ausschluss vs. manuelle Auswahl

Ausschluss-Muster setzen die initiale Auswahl. Der Benutzer kann in der GUI einzelne Dateien manuell wieder auswählen (Checkbox). Die manuelle Auswahl übersteuert den Muster-Ausschluss.

### BR-004: CLI-Muster additiv

CLI `--exclude` Muster werden **zusätzlich** zu den gespeicherten Mustern angewandt. Es gibt keine Möglichkeit, gespeicherte Muster per CLI zu deaktivieren.

### BR-005: Standard-Blacklist

Bei erster Benutzung (leere Settings) ist `file_exclusion_patterns` leer — keine Standard-Blacklist wird automatisch gesetzt. Benutzer müssen Muster explizit konfigurieren.

## Beteiligte Module

- `core/src/settings.rs` — `AppSettings` erhält neues Feld `file_exclusion_patterns: Vec<String>`
- `core/src/filter.rs` — **Neues Modul**: `filter_excluded_files()`, `matches_exclusion_pattern()`
- `core/src/download/manager.rs` — Aufruft Filter vor Download-Start
- `gui/src/views/main_view.rs` — Initiale Selection basierend auf Exclusion-Filter
- `gui/src/views/settings_view.rs` — UI für Muster-Verwaltung (Textarea)
- `cli/src/main.rs` — Neuer `--exclude` Parameter am `Download`-Subcommand

## Akzeptanzkriterien (aus ACCEPTANCE-TESTS.md)

- AT-24: Datei-Ausschluss per Muster — Container mit gemischten Dateien, Muster schliessen `.nfo`, `.jpg`, `*sample*` aus
- AT-25: Datei-Ausschluss CLI — `--exclude "*.nfo"` überspringt `.nfo`-Dateien

## API Design

```rust
// --- core/src/settings.rs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    // ... bestehende Felder ...
    /// Glob-Muster für Dateien, die vom Download ausgeschlossen werden.
    /// Leer = keine Ausschlüsse. Case-insensitive Matching auf file_name.
    #[serde(default)]
    pub file_exclusion_patterns: Vec<String>,
}

// --- core/src/filter.rs (NEU) ---

use crate::sfdl::models::FileItem;

/// Prüft ob ein Dateiname einem der Ausschluss-Muster entspricht.
/// Glob-Matching, case-insensitive.
pub fn is_excluded(file_name: &str, patterns: &[String]) -> bool;

/// Gibt eine Liste von Booleans zurück (true = ausgeschlossen),
/// indexiert analog zu einer flachen Dateiliste über alle Packages.
/// Kann direkt als initiale `selected_files` Invertierung verwendet werden.
pub fn compute_exclusion_mask(
    files: &[FileItem],
    patterns: &[String],
) -> Vec<bool>;
```

## Implementierungsdetails

### Glob-Matching

Nutze das `glob-match` Crate (leichtgewichtig, keine Dateisystem-Abhängigkeit):

```rust
use glob_match::glob_match;

pub fn is_excluded(file_name: &str, patterns: &[String]) -> bool {
    let lower = file_name.to_lowercase();
    patterns.iter().any(|p| glob_match(&p.to_lowercase(), &lower))
}
```

Alternativ reicht eine einfache Eigenimplementierung für `*` und `?` — Entscheidung bei Implementierung.

### Integration in Download-Flow

**GUI:**
```
Container geladen → compute_exclusion_mask() → selected_files invertiert initialisieren
  selected_files[i] = !exclusion_mask[i]
```

Die bestehende UC-04 Logik (Checkbox-Auswahl) bleibt unverändert. Der Ausschluss setzt nur die initiale Auswahl — alles Weitere läuft über den bestehenden `selected_files`-Mechanismus.

**CLI:**
```
Container geladen → CLI --exclude + settings patterns zusammenführen
  → compute_exclusion_mask() → ausgeschlossene Dateien aus packages entfernen
  → DownloadManager::new() mit gefiltertem Container
```

### Settings-Persistenz

Neues Feld in `settings.json`:
```json
{
  "file_exclusion_patterns": ["*.nfo", "*.jpg", "*sample*"],
  ...
}
```

`#[serde(default)]` stellt sicher, dass bestehende Settings-Dateien ohne das Feld kompatibel bleiben (leerer Vec als Default).

### CLI-Erweiterung

```rust
// In Commands::Download:
/// Exclude files matching glob pattern (can be repeated)
#[arg(long)]
exclude: Vec<String>,
```

Die CLI-Muster werden mit den Settings-Mustern zusammengeführt:
```rust
let mut patterns = settings.file_exclusion_patterns.clone();
patterns.extend(cli_exclude_patterns);
```

### GUI: Einstellungen

In `SettingsView` wird ein Textarea-Feld hinzugefügt (analog zur Auto-Password-Liste):
- Label: "Datei-Ausschluss-Muster (ein Muster pro Zeile)"
- Placeholder: `*.nfo\n*.jpg\n*sample*`
- Parsing: `lines().map(trim).filter(not_empty).collect()`
