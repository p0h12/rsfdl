# Use Case: FTP-Download durchfuehren

## Overview

**Use Case ID:** DL-004
**Use Case Name:** FTP-Download durchfuehren
**Primary Actor:** Benutzer
**Goal:** Alle selektierten Dateien parallel ueber FTP herunterladen und im Zielverzeichnis speichern.
**Requirements:** FR-05, FR-08
**Status:** Stable

## Preconditions

- Ein aufgeloester Container, eine bestaetigte Selektion und ein Zielverzeichnis liegen vor.

## Main Success Scenario

1. System erstellt eine DownloadSession mit den Parametern:
    - `target_directory` aus Einstellungen oder Parameter
    - `max_threads` aus Einstellungen oder Parameter (Standard: 3)
    - `max_speed_kbps` aus Einstellungen oder Parameter (Standard: 0 = unbegrenzt)
2. → **include** DL-003 (Speicherplatz pruefen)
3. System erstellt fuer jede selektierte Datei eine DownloadTask im Status `Pending`.
4. Fuer jede Datei prueft das System, ob sie lokal bereits existiert:
    - Vollstaendig vorhanden (Groesse stimmt) → Status `Skipped`
    - Teilweise vorhanden → Status `Pending`, mit `bytes_downloaded` gesetzt (→ DL-005)
    - Nicht vorhanden → Status `Pending`, `bytes_downloaded=0`
5. System startet den Thread-Pool mit `max_threads` parallelen Workern.
6. Jeder Worker nimmt die naechste `Pending`-Task und:
    - Oeffnet eine FTP-Verbindung mit ConnectionInfo aus dem Container.
    - **[FTPS konfiguriert]** Baut TLS-Handshake auf (→ BR-DL-009).
    - Wechselt in Passive Mode.
    - **[Resume]** Sendet `REST <bytes_downloaded>` (→ DL-005).
    - Sendet `RETR <remote_path>`.
    - Empfaengt Daten in Bloecken und schreibt sie in die lokale Datei.
    - **[Bandbreite konfiguriert]** Throttling nach jedem Block (→ DL-008).
    - Aktualisiert `bytes_downloaded` und `current_speed_bps` nach jedem Block.
    - System emittiert ein Progress-Event (→ Interface Specs).
7. Worker schliesst die FTP-Verbindung nach Abschluss der Datei.
8. Task-Status wechselt zu `Completed`.
9. Wenn alle Tasks abgeschlossen: DownloadSession-Status → `Completed`.
10. **[Auto-Verifikation]** → **extend** POST-001 (Hash-Verifikation)
11. **[Auto-Extraktion]** → **extend** POST-002 (Archive extrahieren)

## Alternative Flows

### A1: FTP-Verbindung fehlgeschlagen

**Trigger:** Worker kann keine Verbindung herstellen (Schritt 6)
**Flow:**

1. Worker kann keine Verbindung herstellen.
2. Task-Status → `Failed` mit `error_type=ConnectionError`.
3. → **extend** DL-007 (Retry) falls Retries verfuegbar.
4. Worker nimmt naechste Task.

### A2: Datei nicht gefunden

**Trigger:** FTP-Server meldet 550 (Schritt 6)
**Flow:**

1. FTP-Server meldet 550 (File not found).
2. Task-Status → `Failed` mit `error_type=FileNotFound`.
3. Kein Retry (permanent failure, → BR-DL-010).
4. Worker nimmt naechste Task.

### A3: Verbindung unterbrochen

**Trigger:** Verbindung bricht waehrend des Transfers ab (Schritt 6)
**Flow:**

1. Verbindung bricht waehrend des Transfers ab.
2. Bereits geschriebene Bytes bleiben erhalten.
3. Task-Status → `Failed` mit `error_type=ConnectionError`.
4. → **extend** DL-007 (Retry).

### A4: Server voll (421)

**Trigger:** FTP-Server meldet 421 (Schritt 6)
**Flow:**

1. FTP-Server meldet 421 (Too many connections).
2. Task-Status → `Failed` mit `error_type=ServerFull`.
3. → **extend** DL-007 (Retry mit Backoff).

### A5: Download abbrechen

**Trigger:** Actor bricht den Download ab (jederzeit)
**Flow:**

1. → DL-006 (Download abbrechen).

## Postconditions

### Success Postconditions

- Alle selektierten Dateien sind im Zielverzeichnis gespeichert.
- DownloadSession hat Status `Completed`.

### Failure Postconditions

- Einige Dateien heruntergeladen, andere fehlgeschlagen: DownloadSession hat Status `Failed` mit Detail pro Datei.
- DownloadSession hat Status `Cancelled` (bei Abbruch): Teilweise heruntergeladene Dateien bleiben erhalten.

## Business Rules

### BR-DL-007: Parallele Downloads

- Jeder Worker verwendet eine eigene FTP-Verbindung.
- `max_threads` bestimmt die maximale Anzahl gleichzeitiger Verbindungen.
- Bei `max_threads=1` werden Dateien sequentiell heruntergeladen.

### BR-DL-008: Streaming

- Daten werden blockweise gelesen und geschrieben (Buffer: 32 KB).
- Es werden nie ganze Dateien im RAM gehalten (NFR-04).
- Lokale Dateien werden im Append-Modus geoeffnet (fuer Resume-Kompatibilitaet).

### BR-DL-009: FTPS/TLS (geplant)

- Aktuell: Nur unverschluesseltes FTP unterstuetzt.
- Geplant: Explicit FTPS (`AUTH TLS`, `PBSZ 0`, `PROT P`) und Implicit FTPS (Port 990).
- Die SSL-Einstellung aus dem SFDL-Container soll Vorrang vor den App-Einstellungen haben.

### BR-DL-010: Fehlerklassifikation

- Retry-faehig: `ServerFull (421)`, `AuthError (530/430)`, `ConnectionError (425/426)`, `Timeout`
- Permanent: `ServerDown (434)`, `FileNotFound (450-452/501/550)`, `IOError`
- Detaillierter Fehlerstatus pro Task (nicht nur "Fehlgeschlagen").

### BR-DL-011: Verzeichnisstruktur

- Lokale Verzeichnisstruktur spiegelt die Remote-Pfade.
- Fehlende Verzeichnisse werden automatisch erstellt.

## Input

- `container`: Aufgeloester Container
- `selection`: Bestaetigte Selektion
- `target_directory`: Zielverzeichnis
- `max_threads: int`
- `max_speed_kbps: int`

## Output

- `DownloadSession` mit `DownloadTask[]` und finalem Status
- Progress-Events (Callback/Channel): `{ task_id, bytes_downloaded, bytes_total, speed_bps }`
