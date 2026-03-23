# CLI-002: `rsfdl list`

**Interface Spec ID:** CLI-002
**Interface:** CLI (headless)
**Implementiert:** SFDL-001, SFDL-002, SFDL-003, DL-002

---

## Beschreibung

Listet alle Dateien eines SFDL-Containers auf, inklusive Ausschluss-Markierungen. Nützlich für Vorschau und Scripting.

## Syntax

```
rsfdl list <datei.sfdl> [--password <pw>] [--json] [--exclude <pattern>] [--no-exclude] [--show-excluded]
```

## Parameter

| Parameter             | Pflicht | Beschreibung                                        |
|-----------------------|---------|-----------------------------------------------------|
| `<datei.sfdl>`        | ja      | Pfad zur SFDL-Datei                                 |
| `--password <pw>`     | nein    | Passwort für verschlüsselte Container               |
| `--json`              | nein    | Ausgabe als JSON                                    |
| `--exclude <pattern>` | nein    | Zusätzliches Ausschlussmuster (mehrfach verwendbar) |
| `--no-exclude`        | nein    | Alle Ausschlussmuster deaktivieren                  |
| `--show-excluded`     | nein    | Ausgeschlossene Dateien mit anzeigen (markiert)     |

## Verhalten

1. System öffnet, parst und entschlüsselt die SFDL-Datei (→ SFDL-001, SFDL-002).
2. System löst den Container-Inhalt auf (→ SFDL-003).
3. System wendet Ausschlussmuster an (→ DL-002).
4. System listet die Dateien auf.

## Ausgabe (Standard)

```
Paket: Movie.Pack.2024
  movie.part01.rar          1.5 GB
  movie.part02.rar          1.5 GB
  movie.part03.rar          1.2 GB
  movie.nfo                    2 KB  [excluded]
  movie.jpg                   85 KB  [excluded]

Paket: Subs
  subs.de.srt                120 KB

47 Dateien (4.2 GB), 5 ausgeschlossen
```

- `[excluded]` wird nur mit `--show-excluded` angezeigt. Ohne das Flag werden ausgeschlossene Dateien nicht aufgelistet.

## Ausgabe (JSON)

```json
{
    "packages": [
        {
            "name": "Movie.Pack.2024",
            "files": [
                {
                    "filename": "movie.part01.rar",
                    "size_bytes": 1610612736,
                    "excluded": false
                },
                {
                    "filename": "movie.nfo",
                    "size_bytes": 2048,
                    "excluded": true,
                    "exclude_pattern": "*.nfo"
                }
            ]
        }
    ],
    "summary": {
        "total_files": 47,
        "selected_files": 42,
        "excluded_files": 5,
        "total_bytes": 4509715660,
        "selected_bytes": 4509500000
    }
}
```

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Erfolg                                   |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungültiges SFDL-Format                   |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
| 5    | FTP-Fehler bei BulkFolder-Auflösung      |
