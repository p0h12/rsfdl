# Business Requirements Catalog: rsfdl

## Legende

- **Prio**: MUST / SHOULD / COULD
- **Phase**: MVP (Phase 1) / P2 / P3

---

## FR — Funktionale Anforderungen

### FR-01: SFDL-Datei öffnen

**Prio**: MUST | **Phase**: MVP

Der Benutzer kann eine `.sfdl`-Datei vom Dateisystem öffnen.
Das System erkennt automatisch die SFDL-Version (v2 oder v3) und parst den XML-Inhalt.

**Akzeptanzkriterien:**

- v3-Dateien (ContainerVersion=10) werden korrekt geparst
- v2-Dateien (SFDLFileVersion) werden korrekt geparst und intern auf v3 normalisiert
- Ungültige Dateien erzeugen eine verständliche Fehlermeldung

---

### FR-02: Verschlüsselte SFDL-Dateien entschlüsseln

**Prio**: MUST | **Phase**: MVP

Wenn eine SFDL-Datei verschlüsselt ist (`Encrypted=true`), fordert das System ein Passwort an.
Alle verschlüsselten Felder werden mit AES-128-CBC entschlüsselt.

**Akzeptanzkriterien:**

- Korrektes Passwort entschlüsselt alle Felder (Host, Username, Password, Dateinamen, Pfade)
- Falsches Passwort wird erkannt und gemeldet (kein stiller Fehlschlag)
- Auto-Passwort-Liste: System probiert gespeicherte Passwörter automatisch durch

---

### FR-03: Container-Inhalt anzeigen

**Prio**: MUST | **Phase**: MVP

Nach dem Öffnen/Entschlüsseln zeigt das System die Container-Informationen an:

- Beschreibung, Uploader
- Verbindungsdaten (Host, Port, Protokoll)
- Paketliste mit Dateien/Ordnern und Gesamtgrösse

**Akzeptanzkriterien:**

- Alle Pakete mit ihren Dateien/BulkFolders werden aufgelistet
- Dateigrössen werden angezeigt (sofern im Container enthalten)
- Bei BulkFolder-Modus werden die Verzeichnisse aufgelöst (via FTP-Listing)

---

### FR-04: Dateien auswählen

**Prio**: SHOULD | **Phase**: MVP

Der Benutzer kann einzelne Dateien oder ganze Pakete für den Download an-/abwählen.

**Akzeptanzkriterien:**

- Checkbox pro Datei und pro Paket (Alle auswählen/abwählen)
- Gesamtgrösse der Auswahl wird angezeigt
- Standard: Alle Dateien sind vorausgewählt

---

### FR-05: FTP-Download

**Prio**: MUST | **Phase**: MVP

Das System lädt ausgewählte Dateien via FTP vom Server herunter.

**Akzeptanzkriterien:**

- Verbindung mit Host:Port und Credentials aus dem Container
- Passive Mode als Standard
- Parallele Downloads (konfigurierbare Anzahl, Standard: 3)
- Fortschrittsanzeige pro Datei (Prozent, Geschwindigkeit)
- Globaler Fortschritt (Gesamtbytes, Dateien erledigt/total)
- Download in konfigurierbares Zielverzeichnis

---

### FR-06: Download fortsetzen (Resume)

**Prio**: MUST | **Phase**: MVP

Abgebrochene Downloads können fortgesetzt werden.

**Akzeptanzkriterien:**

- Lokale Dateigrösse wird geprüft
- Download wird ab letztem Byte fortgesetzt (FTP REST command)
- Vollständig heruntergeladene Dateien werden übersprungen

---

### FR-07: Download abbrechen

**Prio**: MUST | **Phase**: MVP

Der Benutzer kann laufende Downloads stoppen.

**Akzeptanzkriterien:**

- Einzelne Dateien oder alle Downloads auf einmal stoppbar
- Bereits heruntergeladene Bytes bleiben erhalten (für Resume)
- FTP-Verbindungen werden sauber geschlossen

---

### FR-08: FTPS/TLS-Unterstützung

**Prio**: SHOULD | **Phase**: P2

Downloads über verschlüsselte FTP-Verbindungen (FTPS).

**Akzeptanzkriterien:**

- TLS 1.2 und TLS 1.3 werden unterstützt
- SSL-Protokoll-Einstellung aus dem SFDL-Container wird respektiert
- Implicit und Explicit FTPS

---

### FR-09: Hash-Verifikation

**Prio**: SHOULD | **Phase**: P2

Nach dem Download werden Dateien gegen den Hash im Container geprüft.

**Akzeptanzkriterien:**

- MD5, CRC32, SHA1 werden unterstützt
- Ergebnis wird pro Datei angezeigt (Valid / Invalid / No Hash)
- Option: Download bei Hash-Mismatch als fehlgeschlagen markieren
- Fallback: Wenn kein Hash im Container vorhanden, wird der FTP-Server via FEAT-Kommando nach Hash-Fähigkeiten geprüft (MD5, XMD5, XSHA1, XCRC) und der Hash serverseitig abgefragt

---

### FR-10: Retry-Logik

**Prio**: SHOULD | **Phase**: P2

Fehlgeschlagene Downloads werden automatisch wiederholt.

**Akzeptanzkriterien:**

- Konfigurierbare Anzahl Retries (Standard: 3)
- Konfigurierbare Wartezeit zwischen Retries (Standard: 10s)
- Granulare Fehlerklassifikation mit Retry-Entscheidung:
  - Retry-fähig: ServerFull (421), AuthError (530/430), ConnectionError (425/426), Timeout
  - Permanent (kein Retry): ServerDown (434), FileNotFound (450-452/501/550), IOError
- Fehlerstatus pro Datei wird mit spezifischem Fehlertyp angezeigt (nicht nur generisch «Fehlgeschlagen»)

---

### FR-11: Einstellungen persistieren

**Prio**: SHOULD | **Phase**: P2

App-Einstellungen werden gespeichert und beim nächsten Start geladen.

**Akzeptanzkriterien:**

- Download-Verzeichnis, Max-Threads, Retry-Einstellungen, Passwort-Liste, FTP-Timeout, Datei-Ausschluss-Muster
- Gespeichert als JSON-Datei im plattformspezifischen Config-Verzeichnis
- Fehlende oder korrupte Datei führt zu Default-Werten (kein Absturz)
- GUI: Einstellungs-Dialog
- CLI: Einstellungen anzeigen (`config show`) und im Editor bearbeiten (`config edit`, öffnet `$EDITOR`)
- CLI: Einstellungen pro Aufruf per Flag überschreiben (ohne die Datei zu ändern)

---

### FR-13: Drag-and-Drop

**Prio**: COULD | **Phase**: P3

SFDL-Dateien können per Drag-and-Drop auf das App-Fenster gezogen werden.

---

### FR-14: .sfdl Datei-Assoziation

**Prio**: COULD | **Phase**: P3

Das Betriebssystem öffnet `.sfdl`-Dateien automatisch mit rsfdl.

---

### FR-15: Bandbreitenbegrenzung

**Prio**: SHOULD | **Phase**: P2

Der Benutzer kann die maximale Download-Geschwindigkeit begrenzen (KB/s oder MB/s).
Die Begrenzung wird gleichmässig auf alle aktiven Download-Threads verteilt.

**Akzeptanzkriterien:**

- Globale Bandbreitenbegrenzung konfigurierbar in den Einstellungen (0 = unbegrenzt)
- Begrenzung wird pro Thread aufgeteilt: `max_bytes_per_second / aktive_threads`
- Throttling im Read-Loop nach jedem Buffer-Write
- Aktuelle Geschwindigkeit in der Fortschrittsanzeige sichtbar
- CLI: Parameter `--max-speed <KB/s>`

---

### FR-16: Auto-Extraktion

**Prio**: SHOULD | **Phase**: P2

Nach Abschluss eines Downloads werden Archive automatisch entpackt.
Unterstützt werden RAR-Archive (auch Multi-Part) und ZIP-Dateien.

**Akzeptanzkriterien:**

- RAR-Erkennung: Erster Teil eines Multi-Part-Archivs wird erkannt (`.rar`, `.part01.rar`, `.r01`)
- ZIP-Dateien werden entpackt
- Extraktion startet automatisch nach erfolgreichem Download aller Dateien eines Pakets
- Option: Archive nach erfolgreicher Extraktion löschen (Standard: aus)
- Fehlgeschlagene Extraktion wird gemeldet, ohne den Gesamterfolg zu beeinflussen
- Fortschrittsanzeige für Extraktionsvorgang
- Feature ist in den Einstellungen deaktivierbar (Standard: aus)

---

### FR-17: Datei-Ausschluss-Muster

**Prio**: SHOULD | **Phase**: P2

Der Benutzer kann Dateien vom Download ausschliessen, basierend auf konfigurierbaren Mustern.

**Akzeptanzkriterien:**

- Glob-basierte Blacklist (z.B. `*.nfo`, `*.jpg`, `*sample*`)
- Muster werden in den Einstellungen gespeichert
- Ausgeschlossene Dateien werden in der Dateiliste als übersprungen markiert
- Standard-Blacklist enthält typische unerwünschte Dateien (`.nfo`, `.jpg`, `.txt`, Samples)
- Muster können hinzugefügt und entfernt werden
- CLI: Parameter `--exclude <pattern>` (zusätzlich zu gespeicherten Mustern)

---

### FR-18: Speedreport

**Prio**: COULD | **Phase**: P2

Nach Abschluss eines Downloads kann ein formatierter Report generiert werden.
Der Report ist als BBCode-Text für die Verwendung in Foren vorgesehen.

**Akzeptanzkriterien:**

- BBCode-Template mit Variablen: Benutzername, Durchschnittsgeschwindigkeit, Gesamtzeit, Gesamtgrösse, Dateianzahl
- Template ist in den Einstellungen konfigurierbar
- Report wird nach Abschluss aller Downloads eines Containers angeboten
- GUI: Report wird angezeigt mit Kopieren-Button
- CLI: Report wird auf stdout ausgegeben (optional via `--speedreport`)

---

### FR-19: Speicherplatz-Prüfung

**Prio**: SHOULD | **Phase**: P2

Vor dem Start eines Downloads prüft das System, ob genügend Speicherplatz im Zielverzeichnis vorhanden ist.

**Akzeptanzkriterien:**

- Gesamtgrösse der ausgewählten Dateien wird mit verfügbarem Speicherplatz verglichen
- Bei unzureichendem Speicherplatz wird der Benutzer gewarnt, bevor der Download beginnt
- Benutzer kann den Download trotz Warnung starten
- Prüfung berücksichtigt bereits teilweise heruntergeladene Dateien (nur Restgrösse)
- CLI: Warnung auf stderr, Abbruch mit `--strict-disk-check`

---

### FR-20: SFDL-Container erstellen

**Prio**: MUST

Als Benutzer möchte ich eine neue `.sfdl`-Datei aus FTP-Verbindungsdaten und Dateipfaden erstellen, damit ich
SFDL-Container für andere Benutzer bereitstellen kann.

**Akzeptanzkriterien:**

- Benutzer gibt FTP-Verbindungsdaten an (Host, Port, Username, Password)
- Benutzer gibt einen oder mehrere Pfade auf dem FTP-Server an
- System generiert einen gültigen SFDL v3 Container (ContainerVersion=10)
- Die generierte `.sfdl`-Datei kann von rsfdl und SFDL.NET gelesen werden (Round-Trip)

---

### FR-21: FTP-Verzeichnis für Container auflisten

**Prio**: MUST

Als Benutzer möchte ich ein FTP-Verzeichnis rekursiv auflisten lassen, damit der Creator die Dateiliste mit Grössen
automatisch in den Container aufnimmt.

**Akzeptanzkriterien:**

- System verbindet sich zum FTP-Server und listet das angegebene Verzeichnis rekursiv
- Jede Datei wird als FileItem mit file_name, full_path und file_size erfasst
- Verzeichnisstruktur (directory_root, directory_path) wird korrekt aufgelöst
- Fortschrittsanzeige während des Listens (Anzahl gefundene Dateien)
- Fehler bei nicht erreichbarem Server werden verständlich gemeldet

---

### FR-22: BulkFolder-Modus beim Erstellen

**Prio**: MUST

Als Benutzer möchte ich einen Container im BulkFolder-Modus erstellen können (ohne FTP-Verbindung), damit ich nur den
Verzeichnispfad speichere und die Dateiauflösung dem Downloader überlasse.

**Akzeptanzkriterien:**

- Container kann mit BulkFolderMode=true erstellt werden
- Kein FTP-Connect nötig — nur der Pfad wird im Container gespeichert
- Package enthält BulkFolderList statt FileList
- Generierter Container ist kompatibel mit der BulkFolder-Auflösung beim Download (UC-03)

---

### FR-23: SFDL-Container verschlüsseln

**Prio**: MUST

Als Benutzer möchte ich einen erstellten Container optional mit einem Passwort verschlüsseln, damit nur autorisierte
Benutzer die FTP-Zugangsdaten lesen können.

**Akzeptanzkriterien:**

- Verschlüsselung ist optional (Standard: unverschlüsselt)
- Verwendet AES-128-CBC mit MD5-Key-Derivation (identisch zum Entschlüsselungsalgorithmus)
- Kryptographisch zufälliger IV pro verschlüsseltem Feld
- Alle sensitiven Felder werden verschlüsselt (Host, Username, Password, Dateinamen, Pfade — selbe Felder wie bei der
  Entschlüsselung)
- Port, FileSize und HashType bleiben unverschlüsselt
- Container-Flag `Encrypted` wird auf `true` gesetzt
- Verschlüsselter Container kann mit demselben Passwort entschlüsselt werden (Round-Trip mit FR-02)

---

### FR-24: SFDL v3 XML-Serialisierung

**Prio**: MUST

Als System muss ich einen SfdlContainer in gültiges SFDL v3 XML serialisieren können, damit die erstellte `.sfdl`-Datei
dem Standardformat entspricht.

**Akzeptanzkriterien:**

- Ausgabe folgt dem v3 XML-Schema mit Root-Element `<Container>`
- XML-Header: `<?xml version="1.0" encoding="utf-8"?>`
- Alle XML-Elementnamen entsprechen der v3-Spezifikation (PascalCase: `ContainerVersion`, `MaxDownloadThreads`, etc.)
- Enum-Werte werden korrekt serialisiert (z.B. `UTF8`, `Binary`, `Passive`)
- Leere Listen erzeugen leere XML-Elemente (nicht weggelassen)
- Round-Trip: `parse_sfdl(serialize_v3(container))` ergibt semantisch identischen Container

---

### FR-25: Container-Metadaten setzen

**Prio**: SHOULD

Als Benutzer möchte ich Metadaten beim Erstellen eines Containers angeben, damit der Container aussagekräftige
Informationen enthält.

**Akzeptanzkriterien:**

- Beschreibung (z.B. Release-Name) kann gesetzt werden
- Uploader-Name kann gesetzt werden (Standard: "rsfdl")
- Max-Download-Threads kann gesetzt werden (Standard: 3)

---

## NR — Nicht-funktionale Anforderungen

### NR-01: Plattformübergreifend

**Prio**: MUST | **Phase**: MVP

Die App läuft auf macOS, Linux und Windows.

---

### NR-02: Dual-Interface

**Prio**: MUST | **Phase**: MVP

Derselbe Core wird von GUI (Dioxus Desktop) und CLI (headless) genutzt.

---

### NR-03: Performance

**Prio**: SHOULD | **Phase**: MVP

- App startet in unter 2 Sekunden
- SFDL-Parsing und Entschlüsselung in unter 100ms
- Download-Geschwindigkeit wird nicht durch die App limitiert

---

### NR-04: Speicherverbrauch

**Prio**: SHOULD | **Phase**: MVP

- Downloads werden gestreamt (nicht im RAM gepuffert)
- Idle-Speicherverbrauch unter 50 MB

---

### NR-05: Sicherheit

**Prio**: MUST | **Phase**: MVP

- Passwörter werden nicht im Klartext auf der Festplatte gespeichert
- FTP-Credentials existieren nur zur Laufzeit im Speicher
- Keine Remote-Code-Execution durch manipulierte SFDL-Dateien (Input-Validation)

---

### NR-06: Fehlerbehandlung

**Prio**: MUST | **Phase**: MVP

- Netzwerkfehler führen nicht zum App-Absturz
- Alle Fehler werden dem Benutzer verständlich angezeigt
- Logging für Debugging (konfigurierbar: info/debug/trace)

---

### NR-07: Testbarkeit

**Prio**: MUST | **Phase**: MVP

- Core-Logik ist unabhängig von UI testbar
- Unit-Tests für Parsing, Crypto, Download-Logik
- Integration-Tests für CLI-Workflows
