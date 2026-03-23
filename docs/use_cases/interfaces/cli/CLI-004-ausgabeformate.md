# CLI-004: Ausgabeformate und Konventionen

**Interface Spec ID:** CLI-004
**Interface:** CLI (headless)
**Implementiert:** Querschnittlich für alle CLI-Kommandos

---

## Beschreibung

Definiert die allgemeinen Ausgabe-Konventionen der CLI, insbesondere das Zusammenspiel von stdout, stderr und JSON-Modus.

## Kanäle

| Kanal    | Inhalt                                                     |
|----------|------------------------------------------------------------|
| `stdout` | Ergebnis-Daten (Container-Info, Dateilisten, Speed-Report) |
| `stderr` | Fortschritt, Warnungen, Fehlermeldungen, Logging           |

Diese Trennung ermöglicht: `rsfdl list file.sfdl > filelist.txt` ohne Fortschritts-Rauschen.

## Menschenlesbare Ausgabe (Standard)

- Fortschritt auf stderr mit `\r` für In-Place-Updates (wenn Terminal).
- Wenn stderr kein Terminal ist (Pipe/Redirect): Eine Zeile pro abgeschlossener Datei.
- Ergebnis auf stdout als formatierter Text.

## JSON-Modus (`--json`)

Alle Kommandos unterstützen `--json` für maschinenlesbare Ausgabe.

### stdout: Ergebnis als JSON-Objekt

```json
{
    "status": "partial",
    "completed": 45,
    "failed": 2,
    "skipped": 5,
    "total_bytes": 4509715660,
    "duration_seconds": 402,
    "avg_speed_bps": 10785000,
    "failures": [
        {
            "filename": "movie.part03.rar",
            "error_type": "ConnectionError",
            "retries": 3
        },
        {
            "filename": "subs.de.srt",
            "error_type": "FileNotFound",
            "ftp_code": 550
        }
    ]
}
```

### stderr: JSON-Lines für Progress-Events

Ein JSON-Objekt pro Zeile, ein Event pro Änderung:

```json
{
    "event": "task_started",
    "filename": "movie.part01.rar",
    "bytes_total": 1610612736
}
{
    "event": "task_progress",
    "filename": "movie.part01.rar",
    "bytes_downloaded": 67108864,
    "speed_bps": 12300000
}
{
    "event": "task_completed",
    "filename": "movie.part01.rar",
    "bytes_total": 1610612736,
    "duration_seconds": 131
}
{
    "event": "task_failed",
    "filename": "movie.part03.rar",
    "error_type": "ConnectionError",
    "retry": 2
}
{
    "event": "session_completed",
    "status": "partial",
    "completed": 45,
    "failed": 2
}
```

## Quiet-Modus (`--quiet`)

- stderr: Keine Fortschrittsanzeige. Nur Fehler und Warnungen.
- stdout: Nur Ergebnis-Zusammenfassung.
- Kombinierbar mit `--json`.

## Logging

- Gesteuert über `RSFDL_LOG` Umgebungsvariable oder `--log-level`.
- Werte: `error`, `warn`, `info`, `debug`, `trace`
- Standard: `warn`
- Log-Ausgabe auf stderr, prefixed mit Timestamp und Level.

```
2026-02-20T14:30:00Z [DEBUG] FTP RETR /path/to/file.rar
2026-02-20T14:30:00Z [WARN]  Server returned 421, retrying in 20s
```

## Farben

- Farben werden nur verwendet, wenn stderr ein Terminal ist.
- `NO_COLOR` Umgebungsvariable deaktiviert Farben (gemäss no-color.org).
- Rot: Fehler. Gelb: Warnungen. Grün: Erfolg. Blau: Info/Fortschritt.
