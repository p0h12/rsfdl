# CR-002: FTP-Verzeichnis auflisten

**Use Case ID:** CR-002
**Requirements:** FR-21
**Primary Actor:** System
**Trigger:** Wird von CR-001 aufgerufen, wenn `bulk_folder_mode=false`.
**Preconditions:** FTP-Verbindungsdaten und mindestens ein Verzeichnispfad sind vorhanden.
**Postconditions (Erfolg):** Alle Dateien im Verzeichnis sind als FileItems mit file_name, full_path und file_size erfasst.
**Postconditions (Fehlschlag):** Container-Erstellung wird abgebrochen. Fehlermeldung mit Ursache.

---

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

## Alternative Paths

**1a. Server nicht erreichbar:**
1a.1. System kann keine Verbindung zum FTP-Server herstellen.
1a.2. System meldet: „FTP-Server nicht erreichbar: [Fehler]."
1a.3. Use Case endet mit Fehler.

**1b. Authentifizierung fehlgeschlagen:**
1b.1. Server lehnt die Credentials ab.
1b.2. System meldet: „Anmeldung fehlgeschlagen."
1b.3. Use Case endet mit Fehler.

**2a. Verzeichnis existiert nicht:**
2a.1. System kann das angegebene Verzeichnis nicht finden.
2a.2. System meldet: „Verzeichnis nicht gefunden: [Pfad]."
2a.3. Use Case endet mit Fehler.

**3a. Leeres Verzeichnis:**
3a.1. Verzeichnis enthält keine Dateien.
3a.2. System meldet: „Keine Dateien gefunden in [Pfad]."
3a.3. Use Case endet mit Fehler.

## Business Rules

**BR-CR-003: Rekursive Auflistung**

- Unterverzeichnisse werden vollständig rekursiv durchlaufen.
- Verzeichniseinträge selbst werden nicht als FileItems erfasst, nur Dateien.

**BR-CR-004: Verbindungsmanagement**

- FTP-Verbindung wird nach dem Listing sauber geschlossen.
- Passive Mode als Standard.

## Input

- FTP-Verbindungsdaten aus Container
- `paths`: Liste von Verzeichnispfaden

## Output (Erfolg)

- `FileItem[]` mit `file_name`, `full_path`, `file_size` pro Datei
