# UC-01/02: SFDL-Datei öffnen und entschlüsseln

## Scope

Implementiert die Grundlage für alle weiteren Use Cases:

- SFDL-Datei lesen und Version erkennen
- XML in Rust-Structs parsen (v3 und v2)
- Verschlüsselte Felder mit AES-128-CBC entschlüsseln
- Passwort validieren

## Beteiligte Module

- `sfdl/models.rs` — Vollständige Datenmodelle mit Enums
- `sfdl/parser.rs` — XML-Parsing für v3 und v2, Version-Detection
- `sfdl/crypto.rs` — AES-128-CBC Entschlüsselung
- `error.rs` — Fehlertypen

## Akzeptanzkriterien (aus acceptance-tests.md)

- AT-01: Unverschlüsselte v3-Datei parsen
- AT-02: Unverschlüsselte v2-Datei parsen und normalisieren
- AT-03: Verschlüsselte Datei mit korrektem Passwort entschlüsseln
- AT-04: Falsches Passwort erkennen
- AT-05: Auto-Passwort-Liste durchprobieren
- AT-06: Ungültiges XML erkennen

## API Design

```rust
// --- Haupteintrittspunkt ---

/// Liest und parst eine SFDL-Datei.
/// Gibt verschlüsselten Container zurück (Felder noch Base64).
pub fn parse_sfdl(xml: &str) -> Result<SfdlContainer, SfdlError>;

/// Erkennt die SFDL-Version aus dem XML.
pub fn detect_version(xml: &str) -> Result<SfdlVersion, SfdlError>;

// --- Crypto ---

/// Entschlüsselt einen einzelnen Base64-String.
pub fn decrypt_string(ciphertext_b64: &str, password: &str) -> Result<String, CryptoError>;

/// Entschlüsselt alle verschlüsselten Felder eines Containers in-place.
pub fn decrypt_container(container: &mut SfdlContainer, password: &str) -> Result<(), CryptoError>;

/// Prüft ob ein Passwort korrekt ist (entschlüsselt Host, prüft auf gültigen Hostnamen).
pub fn validate_password(container: &SfdlContainer, password: &str) -> bool;

/// Probiert eine Liste von Passwörtern durch, gibt das erste gültige zurück.
pub fn try_passwords(container: &SfdlContainer, passwords: &[String]) -> Option<String>;
```

## Implementierungsdetails

### Parser: Version-Detection

```
Wenn XML enthält "<ContainerVersion>" → v3
Wenn XML enthält "<SFDLFileVersion>" → v2
Sonst → SfdlError::ParseError
```

### Parser: v3 XML-Mapping

| XML Element                    | Rust Feld                                                |
|--------------------------------|----------------------------------------------------------|
| `<Container>`                  | `SfdlContainer`                                          |
| `<ContainerVersion>`           | `container_version: u32`                                 |
| `<Description>`                | `description: String`                                    |
| `<Uploader>`                   | `uploader: String`                                       |
| `<Encrypted>`                  | `encrypted: bool`                                        |
| `<MaxDownloadThreads>`         | `max_download_threads: u32`                              |
| `<Connection>`                 | `connection: Connection`                                 |
| `<Host>`                       | `connection.host: String`                                |
| `<Port>`                       | `connection.port: u16`                                   |
| `<Username>`                   | `connection.username: String`                            |
| `<Password>`                   | `connection.password: String`                            |
| `<AuthRequired>`               | `connection.auth_required: bool`                         |
| `<DataConnectionType>`         | `connection.data_connection_type: FtpDataConnectionType` |
| `<DataType>`                   | `connection.data_type: FtpDataType`                      |
| `<CharacterEncoding>`          | `connection.character_encoding: CharacterEncoding`       |
| `<SSLProtocol>`                | `connection.ssl_protocol: SslProtocol`                   |
| `<ConnectTimeout>`             | `connection.connect_timeout: u32`                        |
| `<CommandTimeout>`             | `connection.command_timeout: u32`                        |
| `<Packages><Package>`          | `packages: Vec<Package>`                                 |
| `<Name>`                       | `package.name: String`                                   |
| `<BulkFolderMode>`             | `package.bulk_folder_mode: bool`                         |
| `<FileList><FileItem>`         | `package.file_list: Vec<FileItem>`                       |
| `<FileName>`                   | `file_item.file_name: String`                            |
| `<DirectoryRoot>`              | `file_item.directory_root: String`                       |
| `<DirectoryPath>`              | `file_item.directory_path: String`                       |
| `<FullPath>`                   | `file_item.full_path: String`                            |
| `<FileSize>`                   | `file_item.file_size: u64`                               |
| `<HashType>`                   | `file_item.hash_type: HashType`                          |
| `<FileHash>`                   | `file_item.file_hash: String`                            |
| `<PackageName>`                | `file_item.package_name: String`                         |
| `<BulkFolderList><BulkFolder>` | `package.bulk_folder_list: Vec<BulkFolder>`              |
| `<BulkFolderPath>`             | `bulk_folder.bulk_folder_path: String`                   |
| `<PackageName>`                | `bulk_folder.package_name: String`                       |

### Parser: v2 → v3 Normalisierung

| v2 Element          | v3 Mapping                |
|---------------------|---------------------------|
| `<SFDLFile>`        | → `SfdlContainer`         |
| `<SFDLFileVersion>` | → `container_version = 2` |
| `<ConnectionInfo>`  | → `Connection`            |
| `<SFDLPackage>`     | → `Package`               |

### Crypto: Entschlüsselungsalgorithmus

```
1. key = MD5(password.as_bytes())          // UTF-8, 16 Bytes
2. decoded = base64_decode(ciphertext)
3. iv = decoded[0..16]
4. ciphertext_bytes = decoded[16..]
5. plaintext = AES-128-CBC-decrypt(ciphertext_bytes, key, iv, PKCS7)
6. return String::from_utf8(plaintext)
```

Fallback bei Fehler: Schritt 1 mit Latin-1 Encoding wiederholen.

### Crypto: Zu entschlüsselnde Felder

Wenn `container.encrypted == true`:

- `container.description`
- `container.uploader`
- `connection.host`
- `connection.username`
- `connection.password`
- Für jedes Package: `package.name`
- Für jedes FileItem: `file_name`, `directory_root`, `directory_path`, `full_path`, `package_name`
- Für jeden BulkFolder: `bulk_folder_path`, `package_name`
