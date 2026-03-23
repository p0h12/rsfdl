# Use Case: rsfdl list

## Overview

**Use Case ID:** CLI-002
**Use Case Name:** rsfdl list
**Primary Actor:** Benutzer
**Goal:** Alle Dateien eines SFDL-Containers auflisten, inklusive Ausschluss-Markierungen.
**Implements:** SFDL-001, SFDL-002, SFDL-003, DL-002
**Interface:** CLI (headless)
**Status:** Stable

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- Eine SFDL-Datei existiert im Dateisystem.

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

## Main Success Scenario

1. Benutzer ruft `rsfdl list <datei.sfdl>` auf.
2. System öffnet, parst und entschlüsselt die SFDL-Datei (-> SFDL-001, SFDL-002).
3. System löst den Container-Inhalt auf (-> SFDL-003).
4. System wendet Ausschlussmuster an (-> DL-002).
5. System gibt die Dateiliste auf stdout aus.

## Alternative Flows

### A1: Passwort erforderlich

**Trigger:** Container verschlüsselt, kein Passwort passt (Schritt 2)
**Flow:** Wie CLI-001 A1/A5.

### A2: BulkFolder-Auflösung fehlgeschlagen

**Trigger:** FTP-Verbindung für BulkFolder schlägt fehl (Schritt 3)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 5.

### A3: Ausschlussmuster überschreiben

**Trigger:** `--exclude` oder `--no-exclude` angegeben (Schritt 4)
**Flow:**

1. `--no-exclude`: Keine Muster werden angewendet.
2. `--exclude <pattern>`: Zusätzliche Muster werden zu den konfigurierten hinzugefügt.
3. Use Case fährt mit Schritt 5 fort.

## Postconditions

### Success Postconditions

- Dateiliste wurde auf stdout ausgegeben.

### Failure Postconditions

- Fehlermeldung auf stderr. Kein Output auf stdout.

## Business Rules

### BR-CLI-002-001: Ausschluss-Anzeige

- Ohne `--show-excluded`: Ausgeschlossene Dateien werden nicht aufgelistet.
- Mit `--show-excluded`: Ausgeschlossene Dateien werden mit `[excluded]` markiert.

### BR-CLI-002-002: Zusammenfassung

Die letzte Zeile zeigt: „N Dateien (X GB), M ausgeschlossen".

## Ausgabe (Standard)

```
Paket: Movie.Pack.2024
  movie.part01.rar          1.5 GB
  movie.part02.rar          1.5 GB
  movie.nfo                    2 KB  [excluded]

47 Dateien (4.2 GB), 5 ausgeschlossen
```

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
