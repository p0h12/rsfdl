# CLI-003: `rsfdl download`

**Interface Spec ID:** CLI-003
**Interface:** CLI (headless)
**Implementiert:** DL-001 bis DL-008, POST-001, POST-002, POST-003

---

## Beschreibung

Öffnet einen SFDL-Container und lädt die selektierten Dateien herunter. Kernkommando der CLI.

## Syntax

```
rsfdl download <datei.sfdl> [Optionen]
```

## Parameter

| Parameter             | Pflicht | Standard       | Beschreibung                               |
|-----------------------|---------|----------------|--------------------------------------------|
| `<datei.sfdl>`        | ja      | —              | Pfad zur SFDL-Datei                        |
| `--password <pw>`     | nein    | Auto-Liste     | Passwort für verschlüsselte Container      |
| `--output <dir>`      | nein    | Einstellung    | Zielverzeichnis                            |
| `--threads <n>`       | nein    | 3              | Max. parallele Downloads                   |
| `--max-speed <KB/s>`  | nein    | 0 (unbegrenzt) | Bandbreitenbegrenzung                      |
| `--exclude <pattern>` | nein    | Einstellung    | Zusätzliches Ausschlussmuster (mehrfach)   |
| `--no-exclude`        | nein    | false          | Alle Ausschlussmuster deaktivieren         |
| `--retries <n>`       | nein    | 3              | Max. Retry-Versuche                        |
| `--retry-delay <s>`   | nein    | 10             | Wartezeit zwischen Retries                 |
| `--strict-disk-check` | nein    | false          | Bei zu wenig Speicherplatz abbrechen       |
| `--extract`           | nein    | false          | Auto-Extraktion nach Download              |
| `--delete-archives`   | nein    | false          | Archive nach Extraktion löschen            |
| `--verify`            | nein    | false          | Hash-Verifikation nach Download            |
| `--speedreport`       | nein    | false          | Speed-Report auf stdout ausgeben           |
| `--json`              | nein    | false          | Progress und Ergebnis als JSON (→ CLI-004) |
| `--quiet`             | nein    | false          | Keine Fortschrittsanzeige, nur Ergebnis    |

## Verhalten

1. System öffnet und entschlüsselt den Container (→ SFDL-001, SFDL-002).
2. System löst den Inhalt auf (→ SFDL-003).
3. System wendet Ausschlussmuster an (→ DL-002).
4. System erstellt die Selektion (→ DL-001): Alle nicht-ausgeschlossenen Dateien.
5. System prüft den Speicherplatz (→ DL-003).
6. System startet den Download (→ DL-004).
7. Fortschrittsanzeige auf stderr (→ CLI-004).
8. Nach Abschluss: optionale Verifikation (→ POST-001), Extraktion (→ POST-002).
9. Ergebnis-Zusammenfassung auf stdout.
10. Optional: Speed-Report auf stdout (→ POST-003).

## Fortschrittsanzeige (stderr)

```
[2/47] movie.part01.rar  ████████████░░░░  75%  12.3 MB/s  ETA 0:42
       movie.part02.rar  ████░░░░░░░░░░░░  25%   8.1 MB/s  ETA 2:15
       movie.part03.rar  ░░░░░░░░░░░░░░░░   0%  wartend...
Gesamt: 1.8 GB / 4.2 GB (43%)  Ø 10.2 MB/s  ETA 4:02
```

- Bei `--quiet`: Keine Fortschrittsanzeige. Nur Ergebnis.
- Bei `--json`: JSON-Lines auf stderr pro Progress-Event (→ CLI-004).
- Fortschrittsanzeige verwendet `\r` für In-Place-Updates (nur wenn stderr ein Terminal ist).

## Ergebnis-Ausgabe (stdout)

```
Download abgeschlossen: 45 OK, 2 fehlgeschlagen, 5 übersprungen
Dauer: 6:42  Grösse: 4.2 GB  Ø 10.7 MB/s
Fehlgeschlagen:
  movie.part03.rar: ConnectionError nach 3 Versuchen
  subs.de.srt: FileNotFound (550)
```

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Alle Dateien erfolgreich                 |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungültiges SFDL-Format                   |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
| 5    | FTP-Fehler bei BulkFolder-Auflösung      |
| 6    | Nicht genug Speicherplatz (strict mode)  |
| 10   | Teilweise fehlgeschlagen                 |
| 11   | Alle Downloads fehlgeschlagen            |
| 12   | Abbruch durch Signal (SIGINT/SIGTERM)    |

## Signal-Handling

- `SIGINT` (Ctrl+C einmal): Graceful Shutdown → DL-006, dann Ergebnis ausgeben.
- `SIGINT` (Ctrl+C zweimal innerhalb 2s): Sofortiger Abbruch.
- `SIGTERM`: Wie einmaliges SIGINT.
