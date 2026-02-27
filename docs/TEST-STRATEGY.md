# Test Strategy: rsfdl

## Ziel

Jede Anforderung aus REQUIREMENTS.md ist durch mindestens einen automatisierten Test abgedeckt.
Tests werden **vor oder parallel** zum Code geschrieben (Test-First / TDD).
Tests schützen das Verhalten bei Code-Regeneration durch AI (AIUP-Prinzip).

---

## Teststufen

### Stufe 1: Unit Tests (Core Crate)

**Scope**: Einzelne Funktionen und Module in `rsfdl-core`
**Framework**: `#[cfg(test)]` + `cargo test`
**Ausführung**: Lokal, schnell, kein Netzwerk, kein Dateisystem

| Modul            | Was wird getestet                                                   |
|------------------|---------------------------------------------------------------------|
| `sfdl/crypto`    | Entschlüsselung mit bekannten Werten, falsches Passwort, Edge Cases |
| `sfdl/parser`    | v3 Parsing, v2 Parsing, ungültiges XML, leere Felder                |
| `sfdl/converter` | v2→v3 Normalisierung                                                |
| `sfdl/models`    | Default-Werte, Serialisierung/Deserialisierung                      |
| `download/item`  | Status-Übergänge, Progress-Berechnung                               |
| `settings`       | Default-Werte, Serialisierung                                       |
| `error`          | Error-Display-Texte                                                 |

**Anforderung**: Keine externen Abhängigkeiten (kein FTP-Server, kein Netzwerk).
Test-Fixtures als `const`-Strings oder eingebettete Dateien (`include_str!`).

### Stufe 2: Integration Tests (Core Crate)

**Scope**: Zusammenspiel mehrerer Module
**Framework**: `tests/` Verzeichnis in `rsfdl-core`
**Ausführung**: Lokal, kann langsamer sein

| Test-Szenario                        | Module beteiligt                |
|--------------------------------------|---------------------------------|
| SFDL öffnen + entschlüsseln          | parser + crypto                 |
| SFDL öffnen + Dateien auflisten      | parser + crypto + models        |
| Parse → Decrypt → Validate Pipeline  | parser + crypto + models        |
| Download-Item aus Container erzeugen | parser + crypto + download/item |

**Fixtures**: Echte `.sfdl`-Testdateien in `tests/fixtures/`:

- `unencrypted_v3.sfdl` — Unverschlüsselter v3-Container
- `encrypted_v3.sfdl` — Verschlüsselter v3-Container (Passwort: "test")
- `unencrypted_v2.sfdl` — Unverschlüsselter v2-Container
- `encrypted_v2.sfdl` — Verschlüsselter v2-Container (Passwort: "test")
- `invalid.sfdl` — Ungültiges XML
- `empty_packages.sfdl` — Container ohne Dateien

### Stufe 3: FTP-Tests (Core Crate, optional)

**Scope**: FTP-Client und Download-Manager gegen echten FTP-Server
**Framework**: `cargo test` mit Feature-Flag `--features ftp-tests`
**Ausführung**: Nur wenn FTP-Server verfügbar, nicht in CI standardmässig

| Test-Szenario         | Was wird getestet                       |
|-----------------------|-----------------------------------------|
| Verbindung herstellen | `ftp/client` Connect + Login            |
| Verzeichnis listen    | `ftp/client` LIST/MLSD                  |
| Datei herunterladen   | `ftp/client` RETR + lokale Datei prüfen |
| Download fortsetzen   | `ftp/client` REST + Partial Download    |
| Rekursives Listing    | `ftp/listing` BulkFolder-Auflösung      |
| Parallele Downloads   | `download/manager` Concurrency          |
| Download abbrechen    | `download/manager` CancellationToken    |

**FTP-Server für Tests**:

- Option A: `test.rebex.net` (öffentlicher Test-FTP)
- Option B: Lokaler FTP-Server via Docker (`docker run -d stilliard/pure-ftpd`)
- Option C: Mock-FTP über trait-basierte Abstraktion

### Stufe 4: CLI E2E Tests

**Scope**: Kompletter Workflow über die CLI-Binary
**Framework**: `assert_cmd` + `predicates` Crates
**Ausführung**: Baut CLI, führt Kommandos aus, prüft Output

| Test-Szenario     | Kommando                                 | Erwartung                               |
|-------------------|------------------------------------------|-----------------------------------------|
| Info anzeigen     | `rsfdl-cli info test.sfdl -p test`       | Beschreibung, Host, Dateizahl im Output |
| Dateien listen    | `rsfdl-cli list test.sfdl -p test`       | Dateinamen und Grössen im Output        |
| Ungültige Datei   | `rsfdl-cli info garbage.txt`             | Exit-Code != 0, Fehlermeldung           |
| Falsches Passwort | `rsfdl-cli info encrypted.sfdl -p wrong` | Exit-Code != 0, "Invalid password"      |
| Ohne Passwort     | `rsfdl-cli info encrypted.sfdl`          | Exit-Code != 0, "Password required"     |
| Hilfe             | `rsfdl-cli --help`                       | Usage-Text                              |

### Stufe 5: GUI Tests (später, Phase 3)

**Scope**: Dioxus Desktop App
**Strategie**: Noch zu definieren — Dioxus hat noch kein etabliertes Test-Framework.
Mögliche Ansätze:

- Component-Tests via Dioxus `VirtualDom` (Headless Rendering)
- Playwright gegen die laufende Desktop-App (Webview-basiert)
- Primär manuelle Acceptance Tests durch den Benutzer

---

## Test-Fixtures Strategie

### SFDL-Testdateien generieren

Da echte SFDL-Dateien Credentials enthalten, erstellen wir synthetische Fixtures:

```
tests/
└── fixtures/
    ├── README.md              # Erklärt die Fixtures
    ├── unencrypted_v3.sfdl    # Generiert mit bekannten Werten
    ├── encrypted_v3.sfdl      # Verschlüsselt mit Passwort "test"
    ├── unencrypted_v2.sfdl
    ├── encrypted_v2.sfdl
    ├── bulk_folder_v3.sfdl    # Container mit BulkFolderMode=true
    ├── invalid.sfdl
    └── empty.sfdl
```

Fixtures werden entweder von Hand erstellt (XML) oder mit einer `generate_fixture`-Hilfsfunktion im Test-Code.

### Crypto-Testvektoren

Bekannte Paare aus Klartext + Passwort → Ciphertext für die Verifikation:

```rust
// Generiert mit der VB.NET SFDL.Container Encrypt-Klasse
const TEST_PASSWORD: &str = "test";
const TEST_PLAINTEXT: &str = "ftp.example.com";
const TEST_CIPHERTEXT_B64: &str = "..."; // aus Referenz-Implementierung
```

---

## Traceability: Requirements → Tests

| Requirement              | Unit Test                | Integration Test  | E2E Test            |
|--------------------------|--------------------------|-------------------|---------------------|
| FR-01: SFDL öffnen       | parser::*                | parse_pipeline::* | cli info/list       |
| FR-02: Entschlüsseln     | crypto::*                | parse_decrypt::*  | cli info -p         |
| FR-03: Inhalt anzeigen   | models::*                | —                 | cli info, cli list  |
| FR-04: Dateien auswählen | item::selection          | —                 | — (GUI)             |
| FR-05: FTP-Download      | —                        | —                 | ftp-tests (Stufe 3) |
| FR-06: Resume            | —                        | —                 | ftp-tests (Stufe 3) |
| FR-07: Abbrechen         | —                        | manager::cancel   | —                   |
| NR-03: Performance       | crypto::bench            | —                 | —                   |
| NR-05: Sicherheit        | parser::input_validation | —                 | cli invalid input   |
| NR-06: Fehlerbehandlung  | error::display           | parse_invalid::*  | cli error cases     |

---

## Ausführung

```bash
# Alle Unit + Integration Tests
cargo test

# Nur Core
cargo test -p rsfdl-core

# Nur CLI E2E
cargo test -p rsfdl-cli

# Mit FTP-Tests (braucht Server)
cargo test -p rsfdl-core --features ftp-tests

# Einzelnen Test
cargo test -p rsfdl-core crypto::tests::decrypt_known_value
```

---

## CI-Pipeline (später)

```yaml
test:
  - cargo test -p rsfdl-core          # Unit + Integration
  - cargo test -p rsfdl-cli           # CLI E2E
  - cargo clippy -- -D warnings        # Linting
  - cargo fmt --check                  # Formatting
```

FTP-Tests laufen **nicht** in CI (brauchen externen Server), nur lokal.
