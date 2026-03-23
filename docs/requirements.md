# Anforderungskatalog: rsfdl

## Legende

| Priority | Beschreibung                                     |
|----------|--------------------------------------------------|
| High     | Muss für den Release vorhanden sein              |
| Medium   | Sollte enthalten sein, Release nicht blockierend |
| Low      | Wünschenswert, kann zurückgestellt werden        |

| Status      | Beschreibung               |
|-------------|----------------------------|
| Open        | Noch nicht begonnen        |
| In Progress | In Bearbeitung             |
| Done        | Implementiert und getestet |

---

## FR — Funktionale Anforderungen

| ID    | Titel                      | User Story                                                                                                                                     | Priority | Status |
|-------|----------------------------|------------------------------------------------------------------------------------------------------------------------------------------------|----------|--------|
| FR-01 | SFDL-Datei öffnen          | Als Benutzer möchte ich eine `.sfdl`-Datei öffnen, damit das System den Inhalt parst (v2/v3) und ich den Container verarbeiten kann.           | High     | Open   |
| FR-02 | Container entschlüsseln    | Als Benutzer möchte ich verschlüsselte SFDL-Dateien entschlüsseln, damit ich geschützte Container verarbeiten kann.                            | High     | Open   |
| FR-03 | Container-Inhalt anzeigen  | Als Benutzer möchte ich den Container-Inhalt (Pakete, Dateien, Verbindungsdaten) sehen, damit ich den Download-Umfang überblicken kann.        | High     | Open   |
| FR-04 | Dateien auswählen          | Als Benutzer möchte ich einzelne Dateien oder Pakete an-/abwählen, damit ich nur relevante Inhalte herunterlade.                               | Medium   | Open   |
| FR-05 | FTP-Download               | Als Benutzer möchte ich Dateien via FTP parallel herunterladen, damit ich die im Container referenzierten Inhalte erhalte.                     | High     | Open   |
| FR-06 | Download fortsetzen        | Als Benutzer möchte ich abgebrochene Downloads fortsetzen, damit ich bei Unterbrüchen nicht von vorne beginnen muss.                           | High     | Open   |
| FR-07 | Download abbrechen         | Als Benutzer möchte ich laufende Downloads stoppen, damit ich die Kontrolle über den Prozess behalte.                                          | High     | Open   |
| FR-08 | FTPS/TLS                   | Als Benutzer möchte ich FTPS-verschlüsselte Verbindungen nutzen, damit meine Datenübertragung geschützt ist.                                   | Medium   | Open   |
| FR-09 | Hash-Verifikation          | Als Benutzer möchte ich heruntergeladene Dateien gegen Hashes verifizieren, damit ich Integrität und Vollständigkeit sicherstelle.             | Medium   | Open   |
| FR-10 | Retry-Logik                | Als Benutzer möchte ich, dass fehlgeschlagene Downloads automatisch wiederholt werden, damit temporäre Fehler keine Intervention nötig machen. | Medium   | Open   |
| FR-11 | Einstellungen persistieren | Als Benutzer möchte ich meine Einstellungen als TOML-Datei speichern, damit ich sie nicht bei jedem Start neu konfigurieren muss.              | Medium   | Open   |
| FR-13 | Drag-and-Drop              | Als Benutzer möchte ich SFDL-Dateien per Drag-and-Drop öffnen, damit ich Dateien schnell und intuitiv laden kann.                              | Low      | Open   |
| FR-14 | Datei-Assoziation          | Als Benutzer möchte ich `.sfdl`-Dateien per Doppelklick öffnen, damit rsfdl nahtlos ins OS integriert ist.                                     | Low      | Open   |
| FR-15 | Bandbreitenbegrenzung      | Als Benutzer möchte ich die Download-Geschwindigkeit begrenzen, damit ich Bandbreite mit anderen Anwendungen teilen kann.                      | Medium   | Open   |
| FR-16 | Auto-Extraktion            | Als Benutzer möchte ich, dass Archive nach dem Download automatisch entpackt werden, damit ich die Inhalte sofort nutzen kann.                 | Medium   | Open   |
| FR-17 | Datei-Ausschluss-Muster    | Als Benutzer möchte ich Dateien per Glob-Muster vom Download ausschliessen, damit unerwünschte Dateien übersprungen werden.                    | Medium   | Open   |
| FR-18 | Speedreport                | Als Benutzer möchte ich einen Speed-Report als BBCode generieren, damit ich Download-Ergebnisse in Foren teilen kann.                          | Low      | Open   |
| FR-19 | Speicherplatz-Prüfung      | Als Benutzer möchte ich vor dem Download über unzureichenden Speicherplatz gewarnt werden, damit ich rechtzeitig reagieren kann.               | Medium   | Open   |
| FR-20 | Container erstellen        | Als Benutzer möchte ich SFDL-Container aus FTP-Verbindungsdaten erstellen, damit ich Container für andere Benutzer bereitstellen kann.         | High     | Open   |
| FR-21 | FTP-Verzeichnis auflisten  | Als Benutzer möchte ich ein FTP-Verzeichnis rekursiv auflisten, damit die Dateiliste automatisch in den Container aufgenommen wird.            | High     | Open   |
| FR-22 | BulkFolder-Modus           | Als Benutzer möchte ich Container im BulkFolder-Modus erstellen, damit ich ohne FTP-Verbindung nur Verzeichnispfade speichern kann.            | High     | Open   |
| FR-23 | Container verschlüsseln    | Als Benutzer möchte ich Container optional verschlüsseln, damit nur autorisierte Benutzer die FTP-Zugangsdaten lesen können.                   | High     | Open   |
| FR-24 | SFDL v3 Serialisierung     | Als System muss ich Container in gültiges SFDL v3 XML serialisieren, damit erstellte Dateien dem Standardformat entsprechen.                   | High     | Open   |
| FR-25 | Container-Metadaten        | Als Benutzer möchte ich Metadaten (Beschreibung, Uploader) beim Erstellen angeben, damit der Container aussagekräftig ist.                     | Medium   | Open   |

---

## NFR — Nicht-funktionale Anforderungen

| ID     | Titel                   | Anforderung                                                         | Kategorie       | Priority | Status |
|--------|-------------------------|---------------------------------------------------------------------|-----------------|----------|--------|
| NFR-01 | Startzeit               | App startet in unter 2 Sekunden.                                    | Performance     | Medium   | Open   |
| NFR-02 | Parsing-Geschwindigkeit | SFDL-Parsing und Entschlüsselung in unter 100 ms.                   | Performance     | Medium   | Open   |
| NFR-03 | Download-Durchsatz      | Download-Geschwindigkeit wird nicht durch die App limitiert.        | Performance     | Medium   | Open   |
| NFR-04 | Streaming-Downloads     | Downloads werden gestreamt, nicht im RAM gepuffert.                 | Ressourcen      | Medium   | Open   |
| NFR-05 | Speicherverbrauch       | Idle-Speicherverbrauch unter 50 MB.                                 | Ressourcen      | Medium   | Open   |
| NFR-06 | Passwort-Sicherheit     | Passwörter werden nicht im Klartext auf der Festplatte gespeichert. | Sicherheit      | High     | Open   |
| NFR-07 | Credential-Schutz       | FTP-Credentials existieren nur zur Laufzeit im Speicher.            | Sicherheit      | High     | Open   |
| NFR-08 | Input-Validierung       | Keine Remote-Code-Execution durch manipulierte SFDL-Dateien.        | Sicherheit      | High     | Open   |
| NFR-09 | Fehlertoleranz          | Netzwerkfehler führen nicht zum App-Absturz.                        | Zuverlässigkeit | High     | Open   |
| NFR-10 | Fehlermeldungen         | Alle Fehler werden dem Benutzer verständlich angezeigt.             | Benutzbarkeit   | High     | Open   |
| NFR-11 | Logging                 | Konfigurierbares Logging mit Stufen info/debug/trace.               | Wartbarkeit     | High     | Open   |
| NFR-12 | Testbare Architektur    | Core-Logik ist unabhängig von UI testbar.                           | Testbarkeit     | High     | Open   |
| NFR-13 | Unit-Tests              | Unit-Tests für Parsing, Crypto und Download-Logik.                  | Testbarkeit     | High     | Open   |
| NFR-14 | Integration-Tests       | Integration-Tests für CLI-Workflows.                                | Testbarkeit     | High     | Open   |

---

## C — Constraints

| ID   | Titel          | Constraint                                                                                   | Kategorie   | Priority | Status |
|------|----------------|----------------------------------------------------------------------------------------------|-------------|----------|--------|
| C-01 | Plattformen    | App läuft auf macOS, Linux und Windows.                                                      | Technisch   | High     | Open   |
| C-02 | Dual-Interface | Derselbe Core wird von App (Dioxus Desktop/Mobile) und CLI (headless) genutzt.               | Architektur | High     | Open   |
| C-03 | Config-Format  | Einstellungen werden als TOML gespeichert; Dateipfad wird von der UI-Schicht bereitgestellt. | Technisch   | High     | Open   |
