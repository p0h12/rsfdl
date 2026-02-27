# Acceptance Tests: rsfdl

Abgeleitet aus use-cases.md und requirements.md.
Jeder Test referenziert den Use Case und das Requirement, das er verifiziert.

---

## AT-01: Unverschlüsselte v3-Datei öffnen

**UC**: UC-01 | **FR**: FR-01

**Given**: Eine unverschlüsselte SFDL v3 Datei `unencrypted_v3.sfdl`
**When**: System parst die Datei
**Then**:

- ContainerVersion = 10
- Description, Uploader, Host, Packages sind korrekt gefüllt
- Encrypted = false
- Kein Passwort wird benötigt

---

## AT-02: Unverschlüsselte v2-Datei öffnen

**UC**: UC-01 | **FR**: FR-01

**Given**: Eine unverschlüsselte SFDL v2 Datei `unencrypted_v2.sfdl`
**When**: System parst die Datei
**Then**:

- Datei wird intern auf v3-Modell normalisiert
- Connection, Packages, BulkFolders sind korrekt gemappt
- Alle Felder des v3-Modells sind befüllt

---

## AT-03: Verschlüsselte Datei mit korrektem Passwort

**UC**: UC-02 | **FR**: FR-02

**Given**: Eine verschlüsselte SFDL Datei `encrypted_v3.sfdl` mit Passwort "test"
**When**: System entschlüsselt mit Passwort "test"
**Then**:

- Host enthält einen gültigen Hostnamen (kein Base64-Müll)
- Username, Password sind entschlüsselt
- Alle Dateinamen und Pfade sind lesbar
- Keine CryptoError-Exception

---

## AT-04: Verschlüsselte Datei mit falschem Passwort

**UC**: UC-02 | **FR**: FR-02

**Given**: Eine verschlüsselte SFDL Datei `encrypted_v3.sfdl`
**When**: System versucht Entschlüsselung mit Passwort "wrong"
**Then**:

- System meldet `InvalidPassword` Fehler
- Keine teilweise entschlüsselten Daten werden zurückgegeben
- CLI: Exit-Code != 0

---

## AT-05: Auto-Passwort-Liste

**UC**: UC-02 | **FR**: FR-02

**Given**: Auto-Passwort-Liste enthält ["wrong1", "test", "wrong2"]
**And**: Verschlüsselte Datei mit Passwort "test"
**When**: System probiert Passwörter durch
**Then**:

- Passwort "test" wird gefunden
- Container wird entschlüsselt ohne manuelle Eingabe

---

## AT-06: Ungültige Datei

**UC**: UC-01 (A2) | **FR**: FR-01

**Given**: Eine Datei `invalid.sfdl` mit ungültigem XML
**When**: System versucht zu parsen
**Then**:

- System meldet ParseError mit verständlicher Meldung
- Kein Crash/Panic

---

## AT-07: Container-Info anzeigen (CLI)

**UC**: UC-11 | **FR**: FR-03

**Given**: Geparstes SFDL-File mit Beschreibung "Test.Release", Uploader "user1", Host "ftp.example.com", 2 Dateien
**When**: `rsfdl-cli info test.sfdl -p test`
**Then**: Output enthält:

- "Test.Release"
- "user1"
- "ftp.example.com"
- "2" (Dateizahl)
- Exit-Code = 0

---

## AT-08: Dateiliste anzeigen (CLI)

**UC**: UC-11 | **FR**: FR-03

**Given**: Container mit Dateien "file1.rar" (100MB) und "file2.rar" (200MB)
**When**: `rsfdl-cli list test.sfdl -p test`
**Then**: Output enthält:

- "file1.rar" und "file2.rar"
- Grössen (100 MB, 200 MB)
- Gesamtgrösse (300 MB)
- Exit-Code = 0

---

## AT-09: FTP-Verbindung herstellen

**UC**: UC-05 | **FR**: FR-05

**Given**: Container mit gültigen FTP-Credentials
**When**: System verbindet sich zum FTP-Server
**Then**:

- Verbindung wird hergestellt
- Login erfolgreich
- Passive Mode aktiv

---

## AT-10: Datei herunterladen

**UC**: UC-05 | **FR**: FR-05

**Given**: FTP-Verbindung steht, Datei "test.txt" (bekannter Inhalt) auf Server
**When**: System lädt Datei herunter
**Then**:

- Lokale Datei existiert im Zielverzeichnis
- Dateigrösse stimmt mit Remote überein
- Dateiinhalt ist korrekt

---

## AT-11: Parallele Downloads

**UC**: UC-05 | **FR**: FR-05

**Given**: Container mit 5 Dateien, max_download_threads = 3
**When**: Download wird gestartet
**Then**:

- Maximal 3 gleichzeitige Transfers
- Alle 5 Dateien werden erfolgreich heruntergeladen

---

## AT-12: Fortschrittsanzeige

**UC**: UC-06 | **FR**: FR-05

**Given**: Download einer Datei läuft
**When**: Daten werden übertragen
**Then**:

- ProgressUpdate Events werden gesendet
- bytes_downloaded steigt monoton
- progress_percent geht von 0.0 bis 100.0
- speed_bytes_per_sec > 0 während Übertragung

---

## AT-13: Download abbrechen

**UC**: UC-07 | **FR**: FR-07

**Given**: Download läuft (mind. eine Datei aktiv)
**When**: CancellationToken wird ausgelöst
**Then**:

- Alle aktiven Transfers stoppen innerhalb von 5 Sekunden
- Teilweise geschriebene Dateien bleiben erhalten
- DownloadStatus = Cancelled
- FTP-Verbindungen werden geschlossen

---

## AT-14: Download fortsetzen (Resume)

**UC**: UC-08 | **FR**: FR-06

**Given**: Datei "test.bin" (1000 Bytes) auf FTP-Server
**And**: Lokale Datei "test.bin" existiert mit 500 Bytes
**When**: Download wird gestartet
**Then**:

- System setzt ab Byte 500 fort (FTP REST)
- Lokale Datei ist am Ende 1000 Bytes gross
- Dateiinhalt ist korrekt

---

## AT-15: Bereits vollständige Datei überspringen

**UC**: UC-08 | **FR**: FR-06

**Given**: Datei "test.bin" (1000 Bytes) auf FTP-Server
**And**: Lokale Datei "test.bin" existiert mit 1000 Bytes
**When**: Download wird gestartet
**Then**:

- Datei wird übersprungen
- DownloadStatus = AlreadyDownloaded
- Kein FTP-Transfer findet statt

---

## AT-16: Passwort erforderlich ohne Angabe (CLI)

**UC**: UC-01 | **FR**: FR-02

**Given**: Verschlüsselte SFDL-Datei
**When**: `rsfdl-cli info encrypted.sfdl` (ohne -p)
**Then**:

- Exit-Code != 0
- Fehlermeldung enthält "password" oder "Passwort"

---

## AT-17: BulkFolder-Auflösung

**UC**: UC-03 | **FR**: FR-03

**Given**: Container mit BulkFolderMode=true, BulkFolderPath="/release/"
**And**: FTP-Server hat 3 Dateien in /release/
**When**: System löst BulkFolders auf
**Then**:

- 3 FileItems werden erzeugt
- Jedes FileItem hat korrekten Pfad und Dateinamen
- Dateigrössen sind gesetzt

---

## AT-18: CLI Help

**UC**: — | **NR**: NR-06

**Given**: —
**When**: `rsfdl-cli --help`
**Then**:

- Exit-Code = 0
- Output enthält Subcommands: info, list, download

---

## AT-19: Bandbreitenbegrenzung aktiv

**UC**: UC-13 | **FR**: FR-15

**Given**: Download läuft mit `max_speed = 1024` KB/s, 2 aktive Threads
**When**: Daten werden übertragen
**Then**:

- Jeder Thread überträgt maximal ~512 KB/s
- Gesamtgeschwindigkeit überschreitet 1024 KB/s nicht signifikant (±10% Toleranz)
- Fortschrittsanzeige zeigt gedrosselte Geschwindigkeit

---

## AT-20: Bandbreitenbegrenzung deaktiviert

**UC**: UC-13 | **FR**: FR-15

**Given**: `max_speed = 0` (unbegrenzt)
**When**: Download läuft
**Then**:

- Kein Throttling aktiv
- Geschwindigkeit wird nur durch Netzwerk/Server limitiert

---

## AT-21: Auto-Extraktion RAR Multi-Part

**UC**: UC-14 | **FR**: FR-16

**Given**: Paket mit Dateien `archive.part01.rar`, `archive.part02.rar`, `archive.part03.rar`
**And**: Auto-Extraktion ist aktiviert
**When**: Alle 3 Dateien sind erfolgreich heruntergeladen
**Then**:

- System erkennt `archive.part01.rar` als ersten Teil
- Extraktion startet automatisch
- Extrahierte Dateien befinden sich im Download-Verzeichnis

---

## AT-22: Auto-Extraktion ZIP

**UC**: UC-14 | **FR**: FR-16

**Given**: Heruntergeladene Datei `files.zip`
**And**: Auto-Extraktion ist aktiviert
**When**: Download abgeschlossen
**Then**:

- ZIP wird entpackt
- Extrahierte Dateien befinden sich im Download-Verzeichnis

---

## AT-23: Auto-Extraktion deaktiviert

**UC**: UC-14 | **FR**: FR-16

**Given**: Heruntergeladene Archivdateien
**And**: Auto-Extraktion ist deaktiviert (Standard)
**When**: Download abgeschlossen
**Then**:

- Keine Extraktion findet statt
- Archivdateien bleiben unverändert

---

## AT-24: Datei-Ausschluss per Muster

**UC**: UC-15 | **FR**: FR-17

**Given**: Container mit Dateien `movie.rar`, `info.nfo`, `cover.jpg`, `sample.mkv`
**And**: Ausschluss-Muster: `["*.nfo", "*.jpg", "*sample*"]`
**When**: System wendet Muster auf Dateiliste an
**Then**:

- `info.nfo`, `cover.jpg`, `sample.mkv` werden als "übersprungen" markiert
- Nur `movie.rar` wird heruntergeladen
- Gesamtgrösse enthält nur `movie.rar`

---

## AT-25: Datei-Ausschluss CLI

**UC**: UC-15 | **FR**: FR-17

**Given**: Container mit Dateien `movie.rar`, `info.nfo`
**When**: `rsfdl-cli download test.sfdl -p pw --exclude "*.nfo"`
**Then**:

- `info.nfo` wird übersprungen
- Output enthält "skipped" für `info.nfo`
- Nur `movie.rar` wird heruntergeladen

---

## AT-26: Speedreport generieren

**UC**: UC-16 | **FR**: FR-18

**Given**: Abgeschlossener Download: 3 Dateien, 4.2 GB, 120 Sekunden, Durchschnitt 35 MB/s
**When**: Speedreport wird generiert
**Then**:

- Report enthält Geschwindigkeit, Gesamtzeit, Grösse, Dateianzahl
- Format ist BBCode
- Template-Variablen sind korrekt ersetzt

---

## AT-27: Speedreport CLI

**UC**: UC-16 | **FR**: FR-18

**Given**: Abgeschlossener Download
**When**: `rsfdl-cli download test.sfdl -p pw --speedreport`
**Then**:

- Nach Download-Abschluss wird Speedreport auf stdout ausgegeben
- Report enthält formatierte Zusammenfassung

---

## AT-28: Speicherplatz ausreichend

**UC**: UC-17 | **FR**: FR-19

**Given**: Ausgewählte Dateien: 2 GB, verfügbarer Speicherplatz: 10 GB
**When**: Download wird gestartet
**Then**:

- Keine Warnung
- Download startet normal

---

## AT-29: Speicherplatz unzureichend (GUI)

**UC**: UC-17 | **FR**: FR-19

**Given**: Ausgewählte Dateien: 10 GB, verfügbarer Speicherplatz: 5 GB
**When**: Download wird gestartet
**Then**:

- Warnung wird angezeigt mit benötigtem und verfügbarem Platz
- Benutzer kann trotzdem starten oder abbrechen

---

## AT-30: Speicherplatz unzureichend (CLI strict)

**UC**: UC-17 | **FR**: FR-19

**Given**: Ausgewählte Dateien: 10 GB, verfügbarer Speicherplatz: 5 GB
**When**: `rsfdl-cli download test.sfdl -p pw --strict-disk-check`
**Then**:

- Warnung auf stderr
- Exit-Code != 0
- Kein Download wird gestartet

---

## AT-31: Hash-Verifikation server-seitig

**UC**: UC-05 | **FR**: FR-09

**Given**: Heruntergeladene Datei ohne Hash im Container
**And**: FTP-Server unterstützt XMD5-Kommando (via FEAT)
**When**: System verifiziert Datei
**Then**:

- System fragt Hash via XMD5 vom Server ab
- Lokaler MD5-Hash wird berechnet und verglichen
- Ergebnis: Valid oder Invalid

---

## AT-32: Retry mit granularer Fehlerklassifikation

**UC**: UC-05 | **FR**: FR-10

**Given**: Download einer Datei, max_retries = 3
**When**: FTP-Server antwortet mit 421 (Service not available / Server Full)
**Then**:

- System klassifiziert als retry-fähig
- Download wird nach Wartezeit erneut versucht (bis zu 3 Mal)
- Fehlerstatus zeigt "ServerFull" (nicht generisch "Failed")

---

## AT-33: Kein Retry bei permanentem Fehler

**UC**: UC-05 | **FR**: FR-10

**Given**: Download einer Datei, max_retries = 3
**When**: FTP-Server antwortet mit 550 (File not found)
**Then**:

- System klassifiziert als permanenter Fehler
- Kein Retry-Versuch
- Fehlerstatus zeigt "FileNotFound"

---

## AT-34: v3 XML-Serialisierung Round-Trip

**UC**: UC-18 | **FR**: FR-24

**Given**: Ein geparstes `SfdlContainer`-Objekt aus `unencrypted_v3.sfdl`
**When**: Container wird mit `serialize_v3()` serialisiert und das Ergebnis mit `parse_sfdl()` zurückgeparst
**Then**:

- Alle Felder sind semantisch identisch (container_version, description, uploader, encrypted, max_download_threads)
- Connection-Felder stimmen überein (host, port, username, password, etc.)
- Packages, FileItems und BulkFolders stimmen überein

---

## AT-35: v3 XML-Serialisierung BulkFolder Round-Trip

**UC**: UC-18 | **FR**: FR-24

**Given**: Ein geparstes `SfdlContainer`-Objekt aus `bulkfolder_v3.sfdl`
**When**: Container wird mit `serialize_v3()` serialisiert und das Ergebnis mit `parse_sfdl()` zurückgeparst
**Then**:

- BulkFolderMode = true
- BulkFolderList enthält dieselben Pfade
- FileList ist leer

---

## AT-36: Container verschlüsseln und entschlüsseln (Round-Trip)

**UC**: UC-19 | **FR**: FR-23

**Given**: Ein unverschlüsselter SfdlContainer mit bekannten Werten (host="ftp.example.com", username="user", etc.)
**When**: `encrypt_container(container, "testpassword")` und danach `decrypt_container(container, "testpassword")`
**Then**:

- Alle Felder haben wieder ihre Originalwerte
- `encrypted` ist am Ende `false`
- Zwischenzustand: `encrypted` war `true`, Host war Base64-String

---

## AT-37: Verschlüsselung mit zufälligem IV

**UC**: UC-19 | **FR**: FR-23

**Given**: Klartext "ftp.example.com" und Passwort "test"
**When**: `encrypt_string()` wird zweimal aufgerufen
**Then**:

- Beide Ciphertexte sind verschieden (unterschiedlicher IV)
- Beide können mit `decrypt_string()` zum selben Klartext entschlüsselt werden

---

## AT-38: Vollständiger Create-Pipeline-Test (Encrypt + Serialize + Parse + Decrypt)

**UC**: UC-18, UC-19 | **FR**: FR-20, FR-23, FR-24

**Given**: Ein manuell aufgebauter SfdlContainer mit FileList
**When**: Container wird verschlüsselt → serialisiert → geparst → entschlüsselt
**Then**:

- Alle Felder stimmen mit dem Original überein
- Die Datei ist gültiges SFDL v3 XML

---

## AT-39: CLI config show zeigt Einstellungen an

**UC**: UC-09 | **FR**: FR-11

**Given**: Settings-Datei existiert mit `download_directory = "/tmp/downloads"`, `max_download_threads = 5`
**When**: `rsfdl-cli config show`
**Then**:

- Output enthält den Pfad zur Settings-Datei
- Output enthält `download_directory` mit Wert `/tmp/downloads`
- Output enthält `max_download_threads` mit Wert `5`
- Exit-Code = 0

---

## AT-40: CLI config show mit Defaults (keine Datei)

**UC**: UC-09 | **FR**: FR-11

**Given**: Keine Settings-Datei vorhanden
**When**: `rsfdl-cli config show`
**Then**:

- Output zeigt Default-Werte an (z.B. `max_download_threads = 3`)
- Kein Fehler, kein Crash
- Exit-Code = 0

---

## AT-41: CLI config edit öffnet Editor

**UC**: UC-09 | **FR**: FR-11

**Given**: `$EDITOR` ist gesetzt
**When**: `rsfdl-cli config edit`
**Then**:

- Falls Settings-Datei nicht existiert: wird mit Defaults erstellt
- Editor wird mit dem Pfad zur Settings-Datei gestartet
- Exit-Code = Exit-Code des Editors

---

## AT-42: CLI Download Override ändert Datei nicht

**UC**: UC-09 | **FR**: FR-11

**Given**: Settings-Datei mit `max_download_threads = 3`
**When**: `rsfdl-cli download test.sfdl -p pw --threads 5`
**Then**:

- Download verwendet 5 Threads
- Settings-Datei enthält nach wie vor `max_download_threads = 3` (unverändert)

---

## AT-43: Korrupte Settings-Datei fällt auf Defaults zurück

**UC**: UC-09 | **FR**: FR-11

**Given**: Settings-Datei enthält ungültiges JSON
**When**: System lädt Einstellungen
**Then**:

- Kein Crash/Panic
- Default-Werte werden verwendet
- Warnung wird ausgegeben

---

## Traceability Matrix

| Acceptance Test | Use Case     | Requirement         | Teststufe          |
|-----------------|--------------|---------------------|--------------------|
| AT-01           | UC-01        | FR-01               | Unit + Integration |
| AT-02           | UC-01        | FR-01               | Unit + Integration |
| AT-03           | UC-02        | FR-02               | Unit + Integration |
| AT-04           | UC-02        | FR-02               | Unit               |
| AT-05           | UC-02        | FR-02               | Unit               |
| AT-06           | UC-01        | FR-01               | Unit               |
| AT-07           | UC-11        | FR-03               | CLI E2E            |
| AT-08           | UC-11        | FR-03               | CLI E2E            |
| AT-09           | UC-05        | FR-05               | FTP-Test           |
| AT-10           | UC-05        | FR-05               | FTP-Test           |
| AT-11           | UC-05        | FR-05               | FTP-Test           |
| AT-12           | UC-06        | FR-05               | FTP-Test           |
| AT-13           | UC-07        | FR-07               | FTP-Test           |
| AT-14           | UC-08        | FR-06               | FTP-Test           |
| AT-15           | UC-08        | FR-06               | FTP-Test           |
| AT-16           | UC-01        | FR-02               | CLI E2E            |
| AT-17           | UC-03        | FR-03               | FTP-Test           |
| AT-18           | —            | NR-06               | CLI E2E            |
| AT-19           | UC-13        | FR-15               | Unit + FTP-Test    |
| AT-20           | UC-13        | FR-15               | Unit               |
| AT-21           | UC-14        | FR-16               | Integration        |
| AT-22           | UC-14        | FR-16               | Integration        |
| AT-23           | UC-14        | FR-16               | Unit               |
| AT-24           | UC-15        | FR-17               | Unit               |
| AT-25           | UC-15        | FR-17               | CLI E2E            |
| AT-26           | UC-16        | FR-18               | Unit               |
| AT-27           | UC-16        | FR-18               | CLI E2E            |
| AT-28           | UC-17        | FR-19               | Unit               |
| AT-29           | UC-17        | FR-19               | Unit               |
| AT-30           | UC-17        | FR-19               | CLI E2E            |
| AT-31           | UC-05        | FR-09               | FTP-Test           |
| AT-32           | UC-05        | FR-10               | FTP-Test           |
| AT-33           | UC-05        | FR-10               | Unit               |
| AT-34           | UC-18        | FR-24               | Unit               |
| AT-35           | UC-18        | FR-24               | Unit               |
| AT-36           | UC-19        | FR-23               | Unit               |
| AT-37           | UC-19        | FR-23               | Unit               |
| AT-38           | UC-18, UC-19 | FR-20, FR-23, FR-24 | Integration        |
| AT-39           | UC-09        | FR-11               | CLI E2E            |
| AT-40           | UC-09        | FR-11               | CLI E2E            |
| AT-41           | UC-09        | FR-11               | CLI E2E            |
| AT-42           | UC-09        | FR-11               | CLI E2E + Unit     |
| AT-43           | UC-09        | FR-11               | Unit               |
