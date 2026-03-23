# CR-003: BulkFolder-Modus

**Use Case ID:** CR-003
**Requirements:** FR-22
**Primary Actor:** System
**Trigger:** Wird von CR-001 aufgerufen, wenn `bulk_folder_mode=true`.
**Preconditions:** Mindestens ein Verzeichnispfad ist angegeben.
**Postconditions (Erfolg):** Container enthält BulkFolderList mit den angegebenen Pfaden. Keine FTP-Verbindung wurde hergestellt.
**Postconditions (Fehlschlag):** Fehlermeldung mit Ursache.

---

## Main Success Scenario

1. System setzt `BulkFolderMode=true` im Container.
2. System erstellt für jeden angegebenen Pfad einen BulkFolder-Eintrag.
3. System fügt die BulkFolderList dem Package hinzu (statt FileList).
4. System gibt das Package an CR-001 zurück.

## Alternative Paths

**1a. Keine Pfade angegeben:**
1a.1. System erkennt, dass keine Pfade vorhanden sind.
1a.2. System meldet: „Mindestens ein Pfad ist erforderlich."
1a.3. Use Case endet mit Fehler.

## Business Rules

**BR-CR-005: BulkFolder vs. FileList**

- Im BulkFolder-Modus enthält das Package ausschliesslich eine BulkFolderList, keine FileList.
- Die Dateiauflösung erfolgt erst beim Download (SFDL-003).
- Es wird keine FTP-Verbindung benötigt — nur die Pfade werden gespeichert.

**BR-CR-006: Kompatibilität**

- Der generierte BulkFolder-Container muss kompatibel sein mit der BulkFolder-Auflösung beim Download (SFDL-003).

## Input

- `paths`: Liste von Verzeichnispfaden auf dem FTP-Server

## Output (Erfolg)

- Package mit `BulkFolderList`
