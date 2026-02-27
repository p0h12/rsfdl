# rsfdl — SFDL Downloader in Rust/Dioxus

## Context

Alternativer SFDL-Downloader als Desktop-App (Dioxus) + CLI, als Ersatz für das Windows-only SFDL.NET.
SFDL-Dateien sind XML-Container mit FTP-Verbindungsdaten und Dateilisten, optional AES-128-CBC verschlüsselt.

## Tech Stack

| Bereich      | Technologie                  |
|--------------|------------------------------|
| GUI          | Dioxus 0.7.x Desktop         |
| Styling      | Tailwind CSS                 |
| FTP          | suppaftp (async, native-tls) |
| SFDL Parsing | quick-xml + serde            |
| Encryption   | aes + cbc + md-5 + base64    |
| CLI          | clap + indicatif             |
| Async        | tokio                        |

## Implementierungs-Schritte

### Phase 1 — MVP (Core + CLI first)

1. Workspace scaffolden
2. Error-Types + Datenmodelle
3. AES-128-CBC Crypto
4. SFDL v3 Parser
5. FTP Client Wrapper
6. Download Manager
7. CLI Commands (download, info, list)
8. CLI testen + stabilisieren
9. GUI Basis-Layout

### Phase 2 — Erweiterungen

- FTPS/TLS Support
- SFDL v2 Parsing + Normalisierung
- Settings-Persistenz (JSON-Datei)
- Auto-Passwort-Liste
- Hash-Verifikation (MD5, CRC, SHA1)

### Phase 3 — Polish

- Retry-Logik
- Drag-and-Drop
- .sfdl Datei-Assoziation
- Disk-Space Check
- Active FTP Mode
- Character Encoding Handling
