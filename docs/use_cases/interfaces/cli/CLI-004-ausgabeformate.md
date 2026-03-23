# Use Case: Ausgabeformate und Konventionen

## Overview

**Use Case ID:** CLI-004
**Use Case Name:** Ausgabeformate und Konventionen
**Primary Actor:** Benutzer / Scripting-Tool
**Goal:** Einheitliche und maschinenlesbare Ausgabe aller CLI-Kommandos sicherstellen.
**Implements:** Querschnittlich für alle CLI-Kommandos
**Interface:** CLI (headless)
**Status:** Stable

## Preconditions

- Der Benutzer ruft ein rsfdl CLI-Kommando auf.

## Main Success Scenario

1. System leitet Ergebnis-Daten (Container-Info, Dateilisten, Speed-Report) auf stdout.
2. System leitet Fortschritt, Warnungen und Fehlermeldungen auf stderr.
3. Benutzer kann stdout in eine Datei umleiten, ohne Fortschritts-Rauschen.

## Alternative Flows

### A1: JSON-Modus

**Trigger:** Benutzer gibt `--json` an
**Flow:**

1. stdout: Ergebnis als JSON-Objekt.
2. stderr: JSON-Lines pro Progress-Event (bei download).
3. Keine menschenlesbaren Texte.

### A2: Quiet-Modus

**Trigger:** Benutzer gibt `--quiet` an
**Flow:**

1. stderr: Keine Fortschrittsanzeige. Nur Fehler und Warnungen.
2. stdout: Nur Ergebnis-Zusammenfassung.
3. Kombinierbar mit `--json`.

### A3: Nicht-Terminal (Pipe/Redirect)

**Trigger:** stderr ist kein Terminal
**Flow:**

1. Keine `\r`-basierten In-Place-Updates.
2. Eine Zeile pro abgeschlossener Datei.

## Postconditions

### Success Postconditions

- Ausgabe erfolgte auf dem korrekten Kanal (stdout/stderr).
- Bei `--json`: Ausgabe ist valides JSON.

### Failure Postconditions

- Fehlermeldung auf stderr.

## Business Rules

### BR-CLI-004-001: Kanaltrennung

| Kanal    | Inhalt                                                     |
|----------|------------------------------------------------------------|
| `stdout` | Ergebnis-Daten (Container-Info, Dateilisten, Speed-Report) |
| `stderr` | Fortschritt, Warnungen, Fehlermeldungen, Logging           |

### BR-CLI-004-002: Fortschrittsanzeige

- Terminal: `\r` für In-Place-Updates mit Progress-Bars.
- Nicht-Terminal: Eine Zeile pro Datei-Abschluss.
- Debouncing: Max. 10 Updates/Sekunde.

### BR-CLI-004-003: JSON-Format

**stdout (Ergebnis-Objekt):**

| Feld             | Typ    | Beschreibung                          |
|------------------|--------|---------------------------------------|
| status           | string | "success", "partial", "failed"        |
| completed        | number | Anzahl erfolgreich heruntergeladener Dateien |
| failed           | number | Anzahl fehlgeschlagener Dateien       |
| skipped          | number | Anzahl übersprungener Dateien         |
| total_bytes      | number | Gesamtgrösse in Bytes                 |
| duration_seconds | number | Dauer in Sekunden                     |
| avg_speed_bps    | number | Durchschnittsgeschwindigkeit in Bytes/s |
| failures[]       | array  | Liste der Fehler (filename, error_type, retries) |

**stderr (Progress-Events als JSON-Lines, ein Objekt pro Zeile):**

| Event-Typ        | Felder                                             |
|------------------|----------------------------------------------------|
| task_started     | filename, bytes_total                              |
| task_progress    | filename, bytes_downloaded, speed_bps              |
| task_completed   | filename, bytes_total, duration_seconds            |
| task_failed      | filename, error_type, retry                        |
| session_completed| status, completed, failed                          |

### BR-CLI-004-004: Logging

- Gesteuert über `RSFDL_LOG` Umgebungsvariable oder `--log-level`.
- Werte: `error`, `warn`, `info`, `debug`, `trace`. Standard: `warn`.
- Log-Ausgabe auf stderr, prefixed mit Timestamp und Level.

### BR-CLI-004-005: Farben

- Farben nur wenn stderr ein Terminal ist.
- `NO_COLOR` Umgebungsvariable deaktiviert Farben (gemäss no-color.org).
- Rot: Fehler. Gelb: Warnungen. Grün: Erfolg. Blau: Info/Fortschritt.
