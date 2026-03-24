# Use Case: rsfdl create

## Overview

**Use Case ID:** CLI-006
**Use Case Name:** rsfdl create
**Primary Actor:** Benutzer
**Goal:** Eine neue SFDL-Datei aus FTP-Verbindungsdaten und Pfaden erstellen.
**Implements:** CR-001 bis CR-006
**Status:** Draft

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- FTP-Server ist erreichbar (ausser bei `--bulk`).

## Syntax

`rsfdl create <output.sfdl> --host <host> --user <user> --pass <pass> --path <pfad> [Optionen]`

## Parameter

| Parameter              | Pflicht | Beschreibung                                   |
|------------------------|---------|------------------------------------------------|
| `<output.sfdl>`        | ja      | Ziel-Dateipfad fuer die erstellte SFDL-Datei   |
| `--host <host>`        | ja      | FTP-Hostname                                   |
| `--port <port>`        | nein    | FTP-Port (Standard: 21)                        |
| `--user <user>`        | ja      | FTP-Benutzername                               |
| `--pass <pass>`        | ja      | FTP-Passwort                                   |
| `--path <pfad>`        | ja      | Pfad auf dem FTP-Server (mehrfach angebbar)    |
| `--bulk`               | nein    | BulkFolder-Modus (kein FTP-Listing, nur Pfade) |
| `--encrypt <passwort>` | nein    | Container mit Passwort verschluesseln          |
| `--description <text>` | nein    | Beschreibung / Release-Name                    |
| `--uploader <name>`    | nein    | Uploader-Name (Standard: "rsfdl")              |
| `--threads <n>`        | nein    | Max-Download-Threads (Standard: 3)             |

## Main Success Scenario

1. Benutzer ruft `rsfdl create <output.sfdl> --host ... --user ... --pass ... --path ...` auf.
2. System validiert Pflichtparameter.
3. System erstellt ein Container-Objekt (-> CR-001).
4. System verbindet zum FTP-Server und listet Verzeichnisse rekursiv auf (-> CR-002).
5. System setzt Metadaten (-> CR-006), falls `--description`, `--uploader` oder `--threads`.
6. System serialisiert den Container als SFDL v3 XML (-> CR-005).
7. System schreibt die Datei auf Disk.
8. System gibt Zusammenfassung auf stdout aus.

## Alternative Flows

### A1: BulkFolder-Modus

**Trigger:** `--bulk` angegeben (statt Schritt 4)
**Flow:**

1. System speichert nur die Pfade im BulkFolder-Modus (-> CR-003), ohne FTP-Verbindung.
2. Use Case faehrt mit Schritt 5 fort.

### A2: Verschluesselung

**Trigger:** `--encrypt <passwort>` angegeben (nach Schritt 5)
**Flow:**

1. System verschluesselt den Container mit dem angegebenen Passwort (-> CR-004).
2. Use Case faehrt mit Schritt 6 fort.

### A3: FTP-Verbindung fehlgeschlagen

**Trigger:** FTP-Server nicht erreichbar (Schritt 4)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 2.

### A4: Pflichtparameter fehlen

**Trigger:** Pflichtparameter nicht angegeben (Schritt 2)
**Flow:**

1. System gibt Usage-Hilfe auf stderr aus.
2. Exit-Code 1.

### A5: Zieldatei nicht schreibbar

**Trigger:** Dateisystem-Fehler beim Schreiben (Schritt 7)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 3.

## Postconditions

### Success Postconditions

- SFDL-Datei wurde auf Disk geschrieben.
- Zusammenfassung auf stdout.

### Failure Postconditions

- Keine Datei geschrieben. Fehlermeldung auf stderr.

## Business Rules

### BR-CLI-017: Standard-Metadaten

- Uploader: "rsfdl" wenn nicht angegeben.
- Threads: 3 wenn nicht angegeben.
- Format: SFDL v3.

Weitere Regeln: -> CLI-CC (Cross-Cutting): Kanaltrennung (BR-CLI-001), Exit-Codes (BR-CLI-007).

## Ausgabe

Fortschrittsmeldungen auf stderr (Verbindungsaufbau, Verzeichnis-Listing). Abschlussmeldung auf stdout mit Dateiname, Gesamtgroesse und Dateianzahl.

## Exit-Codes

| Code | Bedeutung                     |
|------|-------------------------------|
| 0    | Erfolg                        |
| 1    | Pflichtparameter fehlen       |
| 2    | FTP-Verbindung fehlgeschlagen |
| 3    | Zieldatei nicht schreibbar    |
