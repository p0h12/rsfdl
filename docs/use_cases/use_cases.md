# Use Cases — rsfdl

## Actors

| Actor                | Description                                                           |
|----------------------|-----------------------------------------------------------------------|
| Benutzer             | Interagiert mit CLI oder GUI um SFDL-Container zu verarbeiten         |
| System (Timer/Event) | Automatische Aktionen nach Download (Retry, Verifikation, Extraktion) |

## System Use Cases

| ID                                               | Use Case                              | Requirements | Actor    |
|--------------------------------------------------|---------------------------------------|--------------|----------|
| [SFDL-001](system/SFDL-001-datei-oeffnen.md)     | SFDL-Datei öffnen und parsen          | FR-01        | Benutzer |
| [SFDL-002](system/SFDL-002-entschluesseln.md)    | Container entschlüsseln               | FR-02        | Benutzer |
| [SFDL-003](system/SFDL-003-inhalt-aufloesen.md)  | Container-Inhalt auflösen             | FR-03        | Benutzer |
| [CR-001](system/CR-001-container-erstellen.md)   | SFDL-Container erstellen              | FR-20        | Benutzer |
| [CR-002](system/CR-002-ftp-listing.md)           | FTP-Verzeichnis auflisten             | FR-21        | System   |
| [CR-003](system/CR-003-bulkfolder.md)            | BulkFolder-Modus                      | FR-22        | System   |
| [CR-004](system/CR-004-verschluesseln.md)        | Container verschlüsseln               | FR-23        | System   |
| [CR-005](system/CR-005-serialisieren.md)         | Container serialisieren               | FR-24        | System   |
| [CR-006](system/CR-006-metadaten.md)             | Metadaten setzen                      | FR-25        | Benutzer |
| [DL-001](system/DL-001-dateien-auswaehlen.md)    | Dateien zum Download auswählen        | FR-04, FR-17 | Benutzer |
| [DL-002](system/DL-002-ausschlussmuster.md)      | Ausschlussmuster anwenden             | FR-17        | System   |
| [DL-003](system/DL-003-speicherplatz.md)         | Speicherplatz prüfen                  | FR-19        | System   |
| [DL-004](system/DL-004-ftp-download.md)          | FTP-Download durchführen              | FR-05, FR-08 | Benutzer |
| [DL-005](system/DL-005-download-resume.md)       | Download fortsetzen                   | FR-06        | System   |
| [DL-006](system/DL-006-download-cancel.md)       | Download abbrechen                    | FR-07        | Benutzer |
| [DL-007](system/DL-007-retry-logik.md)           | Fehlgeschlagene Downloads wiederholen | FR-10        | System   |
| [DL-008](system/DL-008-bandbreite.md)            | Bandbreite begrenzen                  | FR-15        | Benutzer |
| [POST-001](system/POST-001-hash-verifikation.md) | Dateien verifizieren                  | FR-09        | System   |
| [POST-002](system/POST-002-auto-extraktion.md)   | Archive extrahieren                   | FR-16        | System   |
| [POST-003](system/POST-003-speed-report.md)      | Speed-Report generieren               | FR-18        | Benutzer |
| [CFG-001](system/CFG-001-einstellungen.md)       | Einstellungen verwalten               | FR-11        | Benutzer |

## CLI Interface Specs

| ID                                                  | Use Case         | Implementiert                        |
|-----------------------------------------------------|------------------|--------------------------------------|
| [CLI-001](interfaces/cli/CLI-001-info.md)           | `rsfdl info`     | SFDL-001, SFDL-002, SFDL-003         |
| [CLI-002](interfaces/cli/CLI-002-list.md)           | `rsfdl list`     | SFDL-001, SFDL-002, SFDL-003, DL-002 |
| [CLI-003](interfaces/cli/CLI-003-download.md)       | `rsfdl download` | DL-001 bis DL-008, POST-*            |
| [CLI-004](interfaces/cli/CLI-004-ausgabeformate.md) | Ausgabeformate   | (querschnittlich)                    |
| [CLI-005](interfaces/cli/CLI-005-config.md)         | `rsfdl config`   | CFG-001                              |
| [CLI-006](interfaces/cli/CLI-006-create.md)         | `rsfdl create`   | CR-001 bis CR-006                    |

## App Interface Specs

| ID                                                      | Use Case             | Implementiert                          |
|---------------------------------------------------------|----------------------|----------------------------------------|
| [UI-001](interfaces/app/UI-001-hauptfenster.md)         | Hauptfenster         | SFDL-001, SFDL-003, DL-001             |
| [UI-002](interfaces/app/UI-002-passwort-dialog.md)      | Passwort-Dialog      | SFDL-002                               |
| [UI-003](interfaces/app/UI-003-download-fortschritt.md) | Download-Fortschritt | DL-004, DL-005, DL-006, DL-007, DL-008 |
| [UI-004](interfaces/app/UI-004-download-ergebnis.md)    | Download-Ergebnis    | POST-001, POST-002, POST-003           |
| [UI-005](interfaces/app/UI-005-einstellungen-dialog.md) | Einstellungen-Dialog | CFG-001                                |
| [UI-006](interfaces/app/UI-006-drag-and-drop.md)        | Drag-and-Drop        | SFDL-001                               |
