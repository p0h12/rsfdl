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

stdout (Ergebnis):
```json
{
    "status": "partial",
    "completed": 45,
    "failed": 2,
    "skipped": 5,
    "total_bytes": 4509715660,
    "duration_seconds": 402,
    "avg_speed_bps": 10785000,
    "failures": [...]
}
```

stderr (Progress-Events als JSON-Lines):
```json
{"event": "task_started", "filename": "movie.part01.rar", "bytes_total": 1610612736}
{"event": "task_progress", "filename": "movie.part01.rar", "bytes_downloaded": 67108864, "speed_bps": 12300000}
{"event": "task_completed", "filename": "movie.part01.rar", "bytes_total": 1610612736, "duration_seconds": 131}
{"event": "session_completed", "status": "partial", "completed": 45, "failed": 2}
```

### BR-CLI-004-004: Logging

- Gesteuert über `RSFDL_LOG` Umgebungsvariable oder `--log-level`.
- Werte: `error`, `warn`, `info`, `debug`, `trace`. Standard: `warn`.
- Log-Ausgabe auf stderr, prefixed mit Timestamp und Level.

### BR-CLI-004-005: Farben

- Farben nur wenn stderr ein Terminal ist.
- `NO_COLOR` Umgebungsvariable deaktiviert Farben (gemäss no-color.org).
- Rot: Fehler. Gelb: Warnungen. Grün: Erfolg. Blau: Info/Fortschritt.
