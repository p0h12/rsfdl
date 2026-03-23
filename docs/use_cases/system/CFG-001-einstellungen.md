# CFG-001: Einstellungen verwalten

**Use Case ID:** CFG-001
**Requirements:** FR-11
**Primary Actor:** Benutzer
**Preconditions:** —
**Postconditions (Laden):** Settings-Objekt mit aktuellen Werten im Speicher.
**Postconditions (Speichern):** Settings-Datei auf Disk aktualisiert.

---

## Main Success Scenario

### Variante A: Einstellungen laden

1. System erhält den Konfigurationspfad (→ BR-CFG-001).
2. System prüft, ob die Konfigurationsdatei existiert.
3. **[Datei vorhanden]** System liest und parst die TOML-Datei.
4. System validiert die Werte (→ BR-CFG-003).
5. System erstellt ein Settings-Objekt und gibt es zurück.

### Variante B: Einstellungen ändern

1. Actor ändert einen oder mehrere Einstellungswerte.
2. System validiert die neuen Werte (→ BR-CFG-003).
3. System aktualisiert das Settings-Objekt im Speicher.
4. System schreibt die aktualisierte TOML-Datei auf Disk.

### Variante C: Einstellungen zurücksetzen

1. Actor fordert einen Reset auf Standardwerte an.
2. System erstellt ein Settings-Objekt mit Standardwerten (→ BR-CFG-002).
3. System schreibt die Standardwerte auf Disk.

## Alternative Paths

**2a. (Laden) Datei nicht vorhanden:**
2a.1. System erstellt ein Settings-Objekt mit Standardwerten (→ BR-CFG-002).
2a.2. System schreibt die Standardwerte als neue Datei.

**3a. (Laden) Datei beschädigt / ungültiges TOML:**
3a.1. System meldet: „Konfigurationsdatei beschädigt. Standardwerte werden verwendet."
3a.2. System erstellt Settings mit Standardwerten.
3a.3. Beschädigte Datei wird als `.bak` umbenannt.

**2b. (Ändern) Validierung fehlgeschlagen:**
2b.1. System meldet: „Ungültiger Wert für [Feld]: [Grund]."
2b.2. Alter Wert bleibt erhalten.

## Business Rules

**BR-CFG-001: Dateipfad**

- Der Konfigurationspfad wird von der aufrufenden Schicht (CLI, GUI, Mobile) übergeben.
- Der Core ermittelt keinen Pfad selbst — er nimmt ihn als Parameter entgegen.
- Empfohlene Standardpfade pro Plattform:
    - Linux: `~/.config/rsfdl/settings.toml`
    - macOS: `~/Library/Application Support/rsfdl/settings.toml`
    - Windows: `%APPDATA%\rsfdl\settings.toml`
    - iOS/Android: vom OS-Framework bereitgestelltes App-Verzeichnis
- Umgebungsvariable `RSFDL_CONFIG` überschreibt den Standardpfad (CLI/Desktop).

**BR-CFG-002: Standardwerte**

```toml
download_directory = "~/Downloads/rsfdl"
max_threads = 3
max_speed_kbps = 0
max_retries = 3
retry_delay_seconds = 10
auto_extract = false
delete_archives_after_extract = false
strict_disk_check = false
exclusion_patterns = ["*.nfo", "*.jpg", "*.png", "*.txt", "*sample*"]
password_list = []
speedreport_username = ""
speedreport_template = "[Standard-Template, siehe POST-003]"
```

**BR-CFG-003: Validierung**

- `max_threads`: 1–20
- `max_speed_kbps`: >= 0 (0 = unbegrenzt)
- `max_retries`: 0–50
- `retry_delay_seconds`: 1–3600
- `download_directory`: Pfad muss existieren oder erstellbar sein
- `exclusion_patterns`: Jedes Muster muss gültiger Glob-Syntax entsprechen

**BR-CFG-004: CLI-Überschreibung**

- CLI-Parameter überschreiben gespeicherte Werte für die aktuelle Ausführung.
- CLI-Parameter werden nicht in die Datei zurückgeschrieben.
- Priorität: CLI-Parameter > Konfigurationsdatei > Standardwerte

**BR-CFG-005: Passwort-Speicherung**

- Passwörter in der `password_list` werden verschlüsselt gespeichert (NFR-06).
- Verschlüsselung: OS-spezifischer Keyring oder AES mit maschinengebundenem Schlüssel.

## Input

- `path`: Dateisystempfad zur Konfigurationsdatei (von der UI-Schicht bereitgestellt)
- `action`: Load | Save | Reset
- `changes`: Key-Value-Paare (für Save)

## Output

- `Settings`-Objekt mit aktuellen Werten
