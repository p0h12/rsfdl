# CLI-006: `rsfdl create`

**Interface Spec ID:** CLI-006
**Interface:** CLI (headless)
**Implementiert:** CR-001 bis CR-006

---

## Beschreibung

Erstellt eine neue SFDL-Datei aus FTP-Verbindungsdaten und Pfaden.

## Syntax

```
rsfdl create <output.sfdl> --host <host> --user <user> --pass <pass> --path <pfad> [Optionen]
```

## Parameter

| Parameter              | Pflicht | Beschreibung                                   |
|------------------------|---------|------------------------------------------------|
| `<output.sfdl>`        | ja      | Ziel-Dateipfad für die erstellte SFDL-Datei    |
| `--host <host>`        | ja      | FTP-Hostname                                   |
| `--port <port>`        | nein    | FTP-Port (Standard: 21)                        |
| `--user <user>`        | ja      | FTP-Benutzername                               |
| `--pass <pass>`        | ja      | FTP-Passwort                                   |
| `--path <pfad>`        | ja      | Pfad auf dem FTP-Server (mehrfach angebar)     |
| `--bulk`               | nein    | BulkFolder-Modus (kein FTP-Listing, nur Pfade) |
| `--encrypt <passwort>` | nein    | Container mit Passwort verschlüsseln           |
| `--description <text>` | nein    | Beschreibung / Release-Name                    |
| `--uploader <name>`    | nein    | Uploader-Name (Standard: "rsfdl")              |
| `--threads <n>`        | nein    | Max-Download-Threads (Standard: 3)             |

## Verhalten

1. System validiert Pflichtparameter.
2. System erstellt ein Container-Objekt (→ CR-001).
3. **Ohne `--bulk`:** System verbindet zum FTP-Server und listet Verzeichnisse rekursiv auf (→ CR-002).
4. **Mit `--bulk`:** System speichert nur die Pfade im BulkFolder-Modus (→ CR-003).
5. Falls `--description`, `--uploader` oder `--threads`: Metadaten setzen (→ CR-006).
6. Falls `--encrypt`: Container verschlüsseln (→ CR-004).
7. System serialisiert den Container als SFDL v3 XML (→ CR-005).
8. System schreibt die Datei auf Disk.

## Ausgabe

```
Verbinde mit ftp.example.com:21...
Verzeichnis /release/ wird aufgelistet... 47 Dateien gefunden.
Container erstellt: output.sfdl (4.2 GB, 47 Dateien)
```

## Exit-Codes

| Code | Bedeutung                     |
|------|-------------------------------|
| 0    | Erfolg                        |
| 1    | Pflichtparameter fehlen       |
| 2    | FTP-Verbindung fehlgeschlagen |
| 3    | Zieldatei nicht schreibbar    |
