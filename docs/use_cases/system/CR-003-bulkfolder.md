# Use Case: BulkFolder-Modus

## Overview

**Use Case ID:** CR-003
**Use Case Name:** BulkFolder-Modus
**Primary Actor:** System
**Goal:** Einen Container mit BulkFolder-Einträgen erstellen, ohne eine FTP-Verbindung herzustellen.
**Requirements:** FR-22
**Status:** Stable

## Preconditions

- Mindestens ein Verzeichnispfad ist angegeben.
- Wird von CR-001 aufgerufen, wenn `bulk_folder_mode=true`.

## Main Success Scenario

1. System setzt `BulkFolderMode=true` im Container.
2. System erstellt für jeden angegebenen Pfad einen BulkFolder-Eintrag.
3. System fügt die BulkFolderList dem Package hinzu (statt FileList).
4. System gibt das Package an CR-001 zurück.

## Alternative Flows

### A1: Keine Pfade angegeben

**Trigger:** System erkennt, dass keine Pfade vorhanden sind (Schritt 1)
**Flow:**

1. System erkennt, dass keine Pfade vorhanden sind.
2. System meldet: "Mindestens ein Pfad ist erforderlich."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Container enthält BulkFolderList mit den angegebenen Pfaden.
- Keine FTP-Verbindung wurde hergestellt.

### Failure Postconditions

- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-CR-005: BulkFolder vs. FileList

- Im BulkFolder-Modus enthält das Package ausschliesslich eine BulkFolderList, keine FileList.
- Die Dateiauflösung erfolgt erst beim Download (SFDL-003).
- Es wird keine FTP-Verbindung benötigt -- nur die Pfade werden gespeichert.

### BR-CR-006: Kompatibilität

- Der generierte BulkFolder-Container muss kompatibel sein mit der BulkFolder-Auflösung beim Download (SFDL-003).

## Input

- `paths`: Liste von Verzeichnispfaden auf dem FTP-Server

## Output

- Package mit `BulkFolderList`
