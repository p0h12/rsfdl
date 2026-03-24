# CLI-Konventionen (Cross-Cutting)

**Spec ID:** CLI-CC
**Gilt für:** Alle CLI-Kommandos (CLI-001 bis CLI-006)
**Status:** Stable

Dieses Dokument ist kein Use Case. Es definiert Regeln die querschnittlich für alle CLI-Befehle gelten.

---

## BR-CLI-001: Kanaltrennung

| Kanal    | Inhalt                                                     |
|----------|------------------------------------------------------------|
| `stdout` | Ergebnis-Daten (Container-Info, Dateilisten, Speed-Report) |
| `stderr` | Fortschritt, Warnungen, Fehlermeldungen, Logging           |

Diese Trennung ermöglicht: `rsfdl list file.sfdl > filelist.txt` ohne Fortschritts-Rauschen.

## BR-CLI-002: Fortschrittsanzeige

- Terminal: `\r` für In-Place-Updates mit Progress-Bars.
- Nicht-Terminal (Pipe/Redirect): Eine Zeile pro Datei-Abschluss.
- Debouncing: Max. 10 Updates/Sekunde.

## BR-CLI-003: JSON-Format

Alle Kommandos mit `--json` Flag geben maschinenlesbares JSON aus.

**stdout (Ergebnis-Objekt):**

| Feld             | Typ    | Beschreibung                                     |
|------------------|--------|--------------------------------------------------|
| status           | string | "success", "partial", "failed"                   |
| completed        | number | Anzahl erfolgreich heruntergeladener Dateien     |
| failed           | number | Anzahl fehlgeschlagener Dateien                  |
| skipped          | number | Anzahl übersprungener Dateien                    |
| total_bytes      | number | Gesamtgrösse in Bytes                            |
| duration_seconds | number | Dauer in Sekunden                                |
| avg_speed_bps    | number | Durchschnittsgeschwindigkeit in Bytes/s          |
| failures[]       | array  | Liste der Fehler (filename, error_type, retries) |

**stderr (Progress-Events als JSON-Lines, ein Objekt pro Zeile):**

| Event-Typ         | Felder                                  |
|-------------------|-----------------------------------------|
| task_started      | filename, bytes_total                   |
| task_progress     | filename, bytes_downloaded, speed_bps   |
| task_completed    | filename, bytes_total, duration_seconds |
| task_failed       | filename, error_type, retry             |
| session_completed | status, completed, failed               |

## BR-CLI-004: Logging

- Gesteuert über `RSFDL_LOG` Umgebungsvariable oder `--log-level`.
- Werte: `error`, `warn`, `info`, `debug`, `trace`. Standard: `warn`.
- Log-Ausgabe auf stderr, prefixed mit Timestamp und Level.

## BR-CLI-005: Farben

- Farben nur wenn stderr ein Terminal ist.
- `NO_COLOR` Umgebungsvariable deaktiviert Farben (gemäss no-color.org).
- Rot: Fehler. Gelb: Warnungen. Grün: Erfolg. Blau: Info/Fortschritt.

## BR-CLI-006: Parameter-Priorität

- Priorität: **CLI-Parameter > Konfigurationsdatei > Standardwerte**.
- Alle Parameter mit Standard "CFG" lesen ihren Defaultwert aus der Konfigurationsdatei (-> CFG-001).
- CLI-Parameter überschreiben diese Werte nur für die aktuelle Ausführung.
- Überschriebene Werte werden nicht in die Konfigurationsdatei zurückgeschrieben.

## BR-CLI-007: Exit-Code-Schema

Gemeinsame Exit-Codes für alle CLI-Befehle:

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Erfolg                                   |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungültiges SFDL-Format                   |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |

Zusätzliche Exit-Codes für `download` (CLI-003):

| Code | Bedeutung                             |
|------|---------------------------------------|
| 5    | FTP-Fehler bei BulkFolder-Auflösung   |
| 6    | Nicht genügend Speicherplatz          |
| 10   | Teilweise fehlgeschlagen              |
| 11   | Alle Downloads fehlgeschlagen         |
| 12   | Abbruch durch Signal (SIGINT/SIGTERM) |

## BR-CLI-008: Quiet-Modus

- `--quiet`: Keine Fortschrittsanzeige auf stderr. Nur Fehler und Warnungen.
- stdout: Nur Ergebnis-Zusammenfassung.
- Kombinierbar mit `--json`.
