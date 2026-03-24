# Use Case: SFDL-Container erstellen

## Overview

**Use Case ID:** CR-001
**Use Case Name:** SFDL-Container erstellen
**Primary Actor:** Benutzer
**Goal:** Aus FTP-Verbindungsdaten und Pfaden eine gültige `.sfdl`-Datei erstellen.
**Requirements:** FR-20
**Status:** Tested

## Preconditions

- FTP-Verbindungsdaten (Host, Port, Username, Password) und mindestens ein Pfad auf dem FTP-Server sind bekannt.

## Main Success Scenario

1. Actor übergibt FTP-Verbindungsdaten und einen oder mehrere Pfade.
2. System erstellt ein leeres Container-Objekt mit den Verbindungsdaten.
3. System prüft den Modus:
    - BulkFolder-Modus -> **extend** CR-003
    - FTP-Listing-Modus -> **extend** CR-002
4. **[Optional]** Actor setzt Metadaten -> **extend** CR-006
5. **[Optional]** Actor gibt ein Verschlüsselungspasswort an -> **extend** CR-004
6. -> **include** CR-005 (Container serialisieren)
7. System schreibt die serialisierte XML-Datei auf das Dateisystem.
8. System gibt den Pfad zur erstellten `.sfdl`-Datei zurück.

## Alternative Flows

### A1: Pflichtfelder fehlen

**Trigger:** System erkennt fehlende Pflichtfelder beim Aufbau des Containers (Schritt 1)
**Flow:**

1. System erkennt fehlende Pflichtfelder (z.B. kein Host oder kein Pfad).
2. System meldet: "Container-Erstellung unvollständig: [fehlende Felder]."
3. Use Case endet.

### A2: Datei kann nicht geschrieben werden

**Trigger:** System kann die Zieldatei nicht schreiben (Schritt 7)
**Flow:**

1. System kann die Zieldatei nicht schreiben (Berechtigung, Speicherplatz).
2. System meldet: "Datei konnte nicht geschrieben werden: [OS-Fehler]."
3. Use Case endet.

## Postconditions

### Success Postconditions

- Eine gültige `.sfdl`-Datei (v3, ContainerVersion=10) wurde auf dem Dateisystem geschrieben.

### Failure Postconditions

- Keine Datei wird geschrieben.
- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-CR-001: Round-Trip-Kompatibilität

- Jeder erstellte Container muss von rsfdl und SFDL.NET gelesen werden können.
- Validierung: `parse_sfdl(serialize_v3(container))` ergibt semantisch identischen Container.

### BR-CR-002: Standard-Werte

- `ContainerVersion`: 10
- `MaxDownloadThreads`: 3
- `Uploader`: "rsfdl" (falls nicht explizit gesetzt)

## Input

- `host`: FTP-Hostname
- `port`: FTP-Port (Standard: 21)
- `username`: FTP-Benutzername
- `password`: FTP-Passwort
- `paths`: Liste von Pfaden auf dem FTP-Server
- `bulk_folder_mode`: bool (Standard: false)
- `encryption_password`: Optional -- Passwort für Verschlüsselung
- `output_path`: Ziel-Dateipfad für die `.sfdl`-Datei

## Output

- Dateipfad zur erstellten `.sfdl`-Datei
