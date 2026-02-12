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
- Download-Verzeichnis, Max-Threads, Retry-Einstellungen, Passwort-Liste
- Gespeichert als JSON-Datei im Config-Verzeichnis (`~/.config/rsfdl/settings.json`)
- GUI: Einstellungs-Dialog
- CLI: Konfigurationsdatei oder Kommandozeilen-Parameter

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
