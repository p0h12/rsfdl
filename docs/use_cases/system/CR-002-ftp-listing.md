# Use Case: FTP-Verzeichnis auflisten

## Overview

**Use Case ID:** CR-002
**Use Case Name:** FTP-Verzeichnis auflisten
**Primary Actor:** System
**Goal:** Alle Dateien eines FTP-Verzeichnisses rekursiv als FileItems erfassen, um eine FileList für den Container zu erstellen.
**Requirements:** FR-21
**Status:** Tested

## Preconditions

- FTP-Verbindungsdaten und mindestens ein Verzeichnispfad sind vorhanden.
- Wird von CR-001 aufgerufen, wenn `bulk_folder_mode=false`.

## Main Success Scenario

1. System verbindet sich zum FTP-Server mit den Credentials aus dem Container.
2. System navigiert zum angegebenen Verzeichnis.
3. System listet das Verzeichnis rekursiv auf.
4. Für jede Datei erfasst das System:
    - `file_name`: Dateiname
    - `full_path`: Vollständiger Pfad auf dem Server
    - `file_size`: Dateigrösse in Bytes
5. System löst die Verzeichnisstruktur auf (`directory_root`, `directory_path`).
6. System zeigt Fortschritt an (Anzahl gefundene Dateien).
7. System gibt die Dateiliste als FileItems an CR-001 zurück.

## Alternative Flows

### A1: Server nicht erreichbar

**Trigger:** System kann keine Verbindung zum FTP-Server herstellen (Schritt 1)
**Flow:**

1. System kann keine Verbindung zum FTP-Server herstellen.
2. System meldet: "FTP-Server nicht erreichbar: [Fehler]."
3. Use Case endet mit Fehler.

### A2: Authentifizierung fehlgeschlagen

**Trigger:** Server lehnt die Credentials ab (Schritt 1)
**Flow:**

1. Server lehnt die Credentials ab.
2. System meldet: "Anmeldung fehlgeschlagen."
3. Use Case endet mit Fehler.

### A3: Verzeichnis existiert nicht

**Trigger:** System kann das angegebene Verzeichnis nicht finden (Schritt 2)
**Flow:**

1. System kann das angegebene Verzeichnis nicht finden.
2. System meldet: "Verzeichnis nicht gefunden: [Pfad]."
3. Use Case endet mit Fehler.

### A4: Leeres Verzeichnis

**Trigger:** Verzeichnis enthält keine Dateien (Schritt 3)
**Flow:**

1. Verzeichnis enthält keine Dateien.
2. System meldet: "Keine Dateien gefunden in [Pfad]."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Alle Dateien im Verzeichnis sind als FileItems mit file_name, full_path und file_size erfasst.

### Failure Postconditions

- Container-Erstellung wird abgebrochen.
- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-CR-003: Rekursive Auflistung

- Unterverzeichnisse werden vollständig rekursiv durchlaufen.
- Verzeichniseinträge selbst werden nicht als FileItems erfasst, nur Dateien.

### BR-CR-004: Verbindungsmanagement

- FTP-Verbindung wird nach dem Listing sauber geschlossen.
- Passive Mode als Standard.

## Input

- FTP-Verbindungsdaten aus Container
- `paths`: Liste von Verzeichnispfaden

## Output

- `FileItem[]` mit `file_name`, `full_path`, `file_size` pro Datei
