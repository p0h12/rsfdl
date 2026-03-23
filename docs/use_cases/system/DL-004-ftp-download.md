# DL-004: FTP-Download durchführen

**Use Case ID:** DL-004
**Requirements:** FR-05, FR-08
**Primary Actor:** Benutzer
**Preconditions:** Ein aufgelöster Container, eine bestätigte Selektion und ein Zielverzeichnis liegen vor.
**Postconditions (Erfolg):** Alle selektierten Dateien sind im Zielverzeichnis gespeichert. DownloadSession hat Status `Completed`.
**Postconditions (Teilweise):** Einige Dateien heruntergeladen, andere fehlgeschlagen. DownloadSession hat Status `Failed` mit Detail pro Datei.
**Postconditions (Abbruch):** DownloadSession hat Status `Cancelled`. Teilweise heruntergeladene Dateien bleiben erhalten.

---

## Main Success Scenario

1. System erstellt eine DownloadSession mit den Parametern:
    - `target_directory` aus Einstellungen oder Parameter
    - `max_threads` aus Einstellungen oder Parameter (Standard: 3)
    - `max_speed_kbps` aus Einstellungen oder Parameter (Standard: 0 = unbegrenzt)
2. → **include** DL-003 (Speicherplatz prüfen)
3. System erstellt für jede selektierte Datei eine DownloadTask im Status `Pending`.
4. Für jede Datei prüft das System, ob sie lokal bereits existiert:
    - Vollständig vorhanden (Grösse stimmt) → Status `Skipped`
    - Teilweise vorhanden → Status `Pending`, mit `bytes_downloaded` gesetzt (→ DL-005)
    - Nicht vorhanden → Status `Pending`, `bytes_downloaded=0`
5. System startet den Thread-Pool mit `max_threads` parallelen Workern.
6. Jeder Worker nimmt die nächste `Pending`-Task und:
   6a. Öffnet eine FTP-Verbindung mit ConnectionInfo aus dem Container.
   6b. **[FTPS konfiguriert]** → Baut TLS-Handshake auf (→ BR-DL-009).
   6c. Wechselt in Passive Mode.
   6d. **[Resume]** → Sendet `REST <bytes_downloaded>` (→ DL-005).
   6e. Sendet `RETR <remote_path>`.
   6f. Empfängt Daten in Blöcken und schreibt sie in die lokale Datei.
   6g. **[Bandbreite konfiguriert]** → Throttling nach jedem Block (→ DL-008).
   6h. Aktualisiert `bytes_downloaded` und `current_speed_bps` nach jedem Block.
   6i. System emittiert ein Progress-Event (→ Interface Specs).
7. Worker schliesst die FTP-Verbindung nach Abschluss der Datei.
8. Task-Status wechselt zu `Completed`.
9. Wenn alle Tasks abgeschlossen: DownloadSession-Status → `Completed`.
10. **[Auto-Verifikation]** → **extend** POST-001 (Hash-Verifikation)
11. **[Auto-Extraktion]** → **extend** POST-002 (Archive extrahieren)

## Alternative Paths

**6a-alt. FTP-Verbindung fehlgeschlagen:**
6a-alt.1. Worker kann keine Verbindung herstellen.
6a-alt.2. Task-Status → `Failed` mit `error_type=ConnectionError`.
6a-alt.3. → **extend** DL-007 (Retry) falls Retries verfügbar.
6a-alt.4. Worker nimmt nächste Task.

**6e-alt. Datei nicht gefunden:**
6e-alt.1. FTP-Server meldet 550 (File not found).
6e-alt.2. Task-Status → `Failed` mit `error_type=FileNotFound`.
6e-alt.3. Kein Retry (permanent failure, → BR-DL-010).
6e-alt.4. Worker nimmt nächste Task.

**6f-alt. Verbindung unterbrochen:**
6f-alt.1. Verbindung bricht während des Transfers ab.
6f-alt.2. Bereits geschriebene Bytes bleiben erhalten.
6f-alt.3. Task-Status → `Failed` mit `error_type=ConnectionError`.
6f-alt.4. → **extend** DL-007 (Retry).

**6a-alt2. Server voll (421):**
6a-alt2.1. FTP-Server meldet 421 (Too many connections).
6a-alt2.2. Task-Status → `Failed` mit `error_type=ServerFull`.
6a-alt2.3. → **extend** DL-007 (Retry mit Backoff).

**Cancel-Path:** → DL-006 (Download abbrechen) kann jederzeit eintreten.

## Business Rules

**BR-DL-007: Parallele Downloads**

- Jeder Worker verwendet eine eigene FTP-Verbindung.
- `max_threads` bestimmt die maximale Anzahl gleichzeitiger Verbindungen.
- Bei `max_threads=1` werden Dateien sequentiell heruntergeladen.

**BR-DL-008: Streaming**

- Daten werden blockweise gelesen und geschrieben (Buffer: 64 KB).
- Es werden nie ganze Dateien im RAM gehalten (NFR-04).
- Lokale Dateien werden im Append-Modus geöffnet (für Resume-Kompatibilität).

**BR-DL-009: FTPS/TLS**

- Explicit FTPS: `AUTH TLS` nach Verbindungsaufbau, dann `PBSZ 0` + `PROT P`.
- Implicit FTPS: TLS-Verbindung direkt auf Port 990 (oder konfiguriertem Port).
- TLS 1.2 und 1.3 werden unterstützt. Ältere Versionen werden abgelehnt.
- Die SSL-Einstellung aus dem SFDL-Container hat Vorrang vor den App-Einstellungen.

**BR-DL-010: Fehlerklassifikation**

- Retry-fähig: `ServerFull (421)`, `AuthError (530/430)`, `ConnectionError (425/426)`, `Timeout`
- Permanent: `ServerDown (434)`, `FileNotFound (450-452/501/550)`, `IOError`
- Detaillierter Fehlerstatus pro Task (nicht nur „Fehlgeschlagen").

**BR-DL-011: Verzeichnisstruktur**

- Lokale Verzeichnisstruktur spiegelt die Remote-Pfade.
- Fehlende Verzeichnisse werden automatisch erstellt.

## Input

- `container`: Aufgelöster Container
- `selection`: Bestätigte Selektion
- `target_directory`: Zielverzeichnis
- `max_threads: int`
- `max_speed_kbps: int`

## Output

- `DownloadSession` mit `DownloadTask[]` und finalem Status
- Progress-Events (Callback/Channel): `{ task_id, bytes_downloaded, bytes_total, speed_bps }`
