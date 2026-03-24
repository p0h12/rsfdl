# Use Case: Einstellungen verwalten

## Overview

**Use Case ID:** CFG-001
**Use Case Name:** Einstellungen verwalten
**Primary Actor:** Benutzer
**Goal:** Applikations-Einstellungen laden, ändern und zurücksetzen, damit sie über Neustarts hinweg erhalten bleiben.
**Requirements:** FR-11, C-03
**Status:** Stable

## Preconditions

- Der Konfigurationspfad wurde von der aufrufenden Schicht (CLI, GUI, Mobile) bereitgestellt.

## Main Success Scenario

### Variante A: Einstellungen laden

1. System erhält den Konfigurationspfad (-> BR-CFG-001).
2. System prüft, ob die Konfigurationsdatei existiert.
3. System liest und parst die TOML-Datei.
4. System validiert die Werte (-> BR-CFG-003).
5. System erstellt ein Settings-Objekt und gibt es zurück.

### Variante B: Einstellungen ändern

1. Actor ändert einen oder mehrere Einstellungswerte.
2. System validiert die neuen Werte (-> BR-CFG-003).
3. System aktualisiert das Settings-Objekt im Speicher.
4. System schreibt die aktualisierte TOML-Datei auf Disk.

### Variante C: Einstellungen zurücksetzen

1. Actor fordert einen Reset auf Standardwerte an.
2. System erstellt ein Settings-Objekt mit Standardwerten (-> BR-CFG-002).
3. System schreibt die Standardwerte auf Disk.

## Alternative Flows

### A1: Konfigurationsdatei nicht vorhanden

**Trigger:** Datei existiert nicht (Variante A, Schritt 2)
**Flow:**

1. System erstellt ein Settings-Objekt mit Standardwerten (-> BR-CFG-002).
2. System schreibt die Standardwerte als neue Datei auf Disk.
3. Use Case endet erfolgreich mit Standardwerten.

### A2: Konfigurationsdatei beschädigt / ungültiges TOML

**Trigger:** Datei kann nicht geparst werden (Variante A, Schritt 3)
**Flow:**

1. System meldet: "Konfigurationsdatei beschädigt. Standardwerte werden verwendet."
2. System benennt die beschädigte Datei als `.bak` um.
3. System erstellt ein Settings-Objekt mit Standardwerten (-> BR-CFG-002).
4. Use Case endet erfolgreich mit Standardwerten.

### A3: Validierung beim Laden fehlgeschlagen

**Trigger:** Einzelne Werte liegen ausserhalb der gültigen Bereiche (Variante A, Schritt 4)
**Flow:**

1. System ersetzt ungültige Werte durch die jeweiligen Standardwerte (-> BR-CFG-002).
2. System meldet: "Ungültige Werte korrigiert: [Feldliste]."
3. Use Case fährt mit Schritt 5 fort.

### A4: Validierung beim Ändern fehlgeschlagen

**Trigger:** Neuer Wert verletzt Validierungsregeln (Variante B, Schritt 2)
**Flow:**

1. System meldet: "Ungültiger Wert für [Feld]: [Grund]."
2. Der alte Wert bleibt erhalten.
3. Use Case endet ohne Änderung.

### A5: Schreibfehler auf Disk

**Trigger:** Dateisystem-Fehler beim Schreiben (Variante B Schritt 4, Variante C Schritt 3, A1 Schritt 2)
**Flow:**

1. System meldet: "Konfigurationsdatei konnte nicht geschrieben werden: [Fehlerdetail]."
2. Das Settings-Objekt im Speicher bleibt aktuell.
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Settings-Objekt mit aktuellen Werten ist im Speicher verfügbar.
- Bei Variante B/C: Konfigurationsdatei auf Disk entspricht dem Settings-Objekt.
- Bei A1: Neue Konfigurationsdatei mit Standardwerten existiert auf Disk.

### Failure Postconditions

- Bei A4: Settings-Objekt bleibt unverändert; Konfigurationsdatei auf Disk bleibt unverändert.
- Bei A5: Settings-Objekt im Speicher ist aktuell, aber die Datei auf Disk ist veraltet oder fehlt.

## Business Rules

### BR-CFG-001: Dateipfad

- Der Konfigurationspfad wird von der aufrufenden Schicht (CLI, GUI, Mobile) übergeben.
- Der Core ermittelt keinen Pfad selbst — er nimmt ihn als Parameter entgegen.
- Empfohlene Standardpfade pro Plattform:
    - Linux: `~/.config/rsfdl/settings.toml`
    - macOS: `~/Library/Application Support/rsfdl/settings.toml`
    - Windows: `%APPDATA%\rsfdl\settings.toml`
    - iOS/Android: vom OS-Framework bereitgestelltes App-Verzeichnis
- Umgebungsvariable `RSFDL_CONFIG` überschreibt den Standardpfad (CLI/Desktop).

### BR-CFG-002: Standardwerte

| Feld                         | Standardwert                                    |
|------------------------------|-------------------------------------------------|
| download_directory           | ~/Downloads/rsfdl                               |
| max_threads                  | 3                                               |
| max_speed_kbps               | 0                                               |
| max_retries                  | 3                                               |
| retry_delay_seconds          | 10                                              |
| auto_extract                 | false                                           |
| delete_archives_after_extract| false                                           |
| strict_disk_check            | false                                           |
| ftp_timeout_seconds          | 30                                              |
| exclusion_patterns           | ["*.nfo", "*.jpg", "*.png", "*.txt", "*sample*"]|
| auto_passwords               | []                                              |
| speedreport_template         | ""                                              |

### BR-CFG-003: Validierung

- `max_threads`: 1–20
- `max_speed_kbps`: >= 0 (0 = unbegrenzt)
- `max_retries`: 0–50
- `retry_delay_seconds`: 1–3600
- `download_directory`: Pfad muss existieren oder erstellbar sein
- `exclusion_patterns`: Jedes Muster muss gültiger Glob-Syntax entsprechen

### BR-CFG-004: CLI-Überschreibung

- CLI-Parameter überschreiben gespeicherte Werte für die aktuelle Ausführung.
- CLI-Parameter werden nicht in die Datei zurückgeschrieben.
- Priorität: CLI-Parameter > Konfigurationsdatei > Standardwerte

### BR-CFG-005: Passwort-Speicherung

- Passwoerter in der `auto_passwords` Liste werden aktuell im Klartext in der TOML-Datei gespeichert.
- Geplant: OS-spezifischer Keyring oder AES mit maschinengebundenem Schluessel (NFR-06).
- Die GUI zeigt Passwoerter verdeckt an (Bullet-Zeichen), CLI maskiert sie in der `config show` Ausgabe.

## Input

- `path`: Dateisystempfad zur Konfigurationsdatei (von der UI-Schicht bereitgestellt)
- `action`: Load | Save | Reset
- `changes`: Key-Value-Paare (nur bei Save)

## Output

- `Settings`-Objekt mit aktuellen Werten
- Fehlermeldung bei Validierungs- oder Schreibfehlern
