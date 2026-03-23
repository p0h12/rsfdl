# CR-001: SFDL-Container erstellen

**Use Case ID:** CR-001
**Requirements:** FR-20
**Primary Actor:** Benutzer
**Secondary Actors:** —
**Preconditions:** FTP-Verbindungsdaten (Host, Port, Username, Password) und mindestens ein Pfad auf dem FTP-Server sind bekannt.
**Postconditions (Erfolg):** Eine gültige `.sfdl`-Datei (v3, ContainerVersion=10) wurde auf dem Dateisystem geschrieben.
**Postconditions (Fehlschlag):** Keine Datei wird geschrieben. Fehlermeldung mit Ursache.

---

## Main Success Scenario

1. Actor übergibt FTP-Verbindungsdaten und einen oder mehrere Pfade.
2. System erstellt ein leeres Container-Objekt mit den Verbindungsdaten.
3. System prüft den Modus:
    - BulkFolder-Modus → **extend** CR-003
    - FTP-Listing-Modus → **extend** CR-002
4. **[Optional]** Actor setzt Metadaten → **extend** CR-006
5. **[Optional]** Actor gibt ein Verschlüsselungspasswort an → **extend** CR-004
6. → **include** CR-005 (Container serialisieren)
7. System schreibt die serialisierte XML-Datei auf das Dateisystem.
8. System gibt den Pfad zur erstellten `.sfdl`-Datei zurück.

## Alternative Paths

**1a. Pflichtfelder fehlen:**
1a.1. System erkennt fehlende Pflichtfelder (z.B. kein Host oder kein Pfad).
1a.2. System meldet: „Container-Erstellung unvollständig: [fehlende Felder]."
1a.3. Use Case endet.

**7a. Datei kann nicht geschrieben werden:**
7a.1. System kann die Zieldatei nicht schreiben (Berechtigung, Speicherplatz).
7a.2. System meldet: „Datei konnte nicht geschrieben werden: [OS-Fehler]."
7a.3. Use Case endet.

## Business Rules

**BR-CR-001: Round-Trip-Kompatibilität**

- Jeder erstellte Container muss von rsfdl und SFDL.NET gelesen werden können.
- Validierung: `parse_sfdl(serialize_v3(container))` ergibt semantisch identischen Container.

**BR-CR-002: Standard-Werte**

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
- `encryption_password`: Optional — Passwort für Verschlüsselung
- `output_path`: Ziel-Dateipfad für die `.sfdl`-Datei

## Output (Erfolg)

- Dateipfad zur erstellten `.sfdl`-Datei
