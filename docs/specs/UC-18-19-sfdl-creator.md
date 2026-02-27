# UC-18/19: SFDL-Container erstellen und verschlüsseln

## Scope

Implementiert die Erstellung neuer SFDL-Dateien — das Gegenstück zu UC-01/02 (Öffnen/Entschlüsseln):

- SfdlContainer-Objekt aus Benutzereingaben aufbauen
- Packages im BulkFolder- oder FileList-Modus erstellen
- Alle Felder mit AES-128-CBC verschlüsseln (optional)
- Container als SFDL v3 XML serialisieren
- GUI Creator View mit Formular, FTP-Listing und Speicherdialog

## Beteiligte Module

- `sfdl/builder.rs` — Package-Erzeugung (BulkFolder)
- `sfdl/parser.rs` — `serialize_v3()`, `to_raw_v3()` (XML-Serialisierung)
- `sfdl/crypto.rs` — `encrypt_string()`, `encrypt_container()` (Verschlüsselung)
- `ftp/listing.rs` — `resolve_container_bulk_folders()` (FTP-Listing für FileList-Modus)
- `error.rs` — `SerializeError` Variante
- `gui/views/creator_view.rs` — Creator-Formular und async Erstellungslogik

## Akzeptanzkriterien (aus ACCEPTANCE-TESTS.md)

- AT-34: v3 XML-Serialisierung Round-Trip (FileList)
- AT-35: v3 XML-Serialisierung Round-Trip (BulkFolder)
- AT-36: Container verschlüsseln und entschlüsseln (Round-Trip)
- AT-37: Verschlüsselung mit zufälligem IV
- AT-38: Vollständiger Create-Pipeline-Test (Encrypt → Serialize → Parse → Decrypt)

## API Design

```rust
// --- Builder ---

/// Erzeugt ein Package im BulkFolder-Modus (kein FTP-Connect nötig).
/// Speichert nur den Remote-Pfad — Dateien werden erst beim Download aufgelöst.
pub fn build_bulk_package(path: &str, package_name: &str) -> Package;

// --- Serialisierung ---

/// Serialisiert einen SfdlContainer als SFDL v3 XML-String.
/// Gibt XML mit Header (<?xml version="1.0" encoding="utf-8"?>) zurück.
pub fn serialize_v3(container: &SfdlContainer) -> Result<String, SfdlError>;

/// Konvertiert SfdlContainer → RawContainerV3 (internes XML-Mapping).
fn to_raw_v3(c: &SfdlContainer) -> RawContainerV3;

// --- Verschlüsselung ---

/// Verschlüsselt einen einzelnen String mit AES-128-CBC.
/// Rückgabe: base64(random_IV || ciphertext).
pub fn encrypt_string(plaintext: &str, password: &str) -> String;

/// Verschlüsselt alle sensitiven Felder eines Containers in-place.
/// Spiegelt decrypt_container() — gleiche Felder, gleicher Algorithmus.
pub fn encrypt_container(container: &mut SfdlContainer, password: &str);
```

## Implementierungsdetails

### Builder: BulkFolder-Paket

```
1. Package erstellen mit bulk_folder_mode = true
2. file_list = leer (Vec::new())
3. bulk_folder_list = [BulkFolder { path, package_name }]
4. Paketname = letztes Pfadsegment des Remote-Pfads
```

### Serialisierung: SfdlContainer → XML

```
1. SfdlContainer → RawContainerV3 konvertieren (to_raw_v3)
   - Leere file_list → FileList = None (wird nicht serialisiert)
   - Leere bulk_folder_list → BulkFolderList = None
2. quick_xml::se::to_string(&raw) → XML-Body
3. XML-Header voranstellen: <?xml version="1.0" encoding="utf-8"?>
4. Fehler → SfdlError::SerializeError
```

### Serialisierung: Feld-Mapping (Rust → XML)

Die Raw v3 Structs nutzen `#[serde(rename = "...")]` bidirektional:

| Rust Feld (RawContainerV3) | XML Element            |
|----------------------------|------------------------|
| `container_version`        | `<ContainerVersion>`   |
| `description`              | `<Description>`        |
| `uploader`                 | `<Uploader>`           |
| `encrypted`                | `<Encrypted>`          |
| `max_download_threads`     | `<MaxDownloadThreads>` |
| `connection`               | `<Connection>`         |
| `packages`                 | `<Packages>`           |

Optional-Wrapper für leere Listen:

- `Option<RawFileListV3>` → `<FileList>` nur wenn Dateien vorhanden
- `Option<RawBulkFolderListV3>` → `<BulkFolderList>` nur wenn BulkFolders vorhanden

### Verschlüsselung: Algorithmus (encrypt_string)

```
1. Wenn plaintext leer → leeren String zurückgeben
2. key = MD5(password.as_bytes())          // UTF-8, 16 Bytes
3. iv = rand::random::<[u8; 16]>()         // Kryptographisch zufällig
4. ciphertext = AES-128-CBC-encrypt(plaintext.as_bytes(), key, iv, PKCS7)
5. result = base64(iv || ciphertext)
6. return result
```

### Verschlüsselung: Betroffene Felder (encrypt_container)

Identisch mit `decrypt_container()` — symmetrisch:

| Ebene      | Felder                                                                       |
|------------|------------------------------------------------------------------------------|
| Container  | `description`, `uploader`                                                    |
| Connection | `host`, `username`, `password`                                               |
| Package    | `name`                                                                       |
| FileItem   | `file_name`, `directory_root`, `directory_path`, `full_path`, `package_name` |
| BulkFolder | `bulk_folder_path`, `package_name`                                           |

Nicht verschlüsselt: `port`, `file_size`, `hash_type`, `container_version`, `encrypted`, `max_download_threads`,
`auth_required`, `bulk_folder_mode`, Connection-Enums.

Nach Verschlüsselung: `container.encrypted = true`.
Idempotent: Wenn bereits `encrypted == true` → kein Effekt.

### GUI: Creator View Ablauf

```
1. Benutzer navigiert über Header-Button "Create" → AppView::Creator
2. Formular mit vier Sektionen:
   a. FTP Connection: Host, Port, Username, Password
   b. Content: Remote Path, BulkFolder/FileList Radio-Toggle
   c. Metadata: Description, Threads (1-10), Uploader
   d. Encryption: Password (optional)
3. Benutzer klickt "Create SFDL":
   a. Validierung: Host und Remote Path sind Pflichtfelder
   b. Paketname = letztes Segment des Remote-Pfads
   c. BulkFolder-Modus: build_bulk_package() direkt
   d. FileList-Modus: FTP-Connect → resolve_container_bulk_folders()
   e. SfdlContainer zusammenbauen
   f. Optional: encrypt_container()
   g. serialize_v3()
   h. Native Save-Dialog (rfd::AsyncFileDialog, Filter: *.sfdl)
   i. tokio::fs::write()
4. Fehler werden als Error-Banner angezeigt (AppState.error_message)
5. "Back"-Button → AppView::Main
```

### GUI: Zustandsverwaltung

Alle Formularfelder als `use_signal`:

- `host: Signal<String>`, `port: Signal<u16>`, `username: Signal<String>`, `password: Signal<String>`
- `remote_path: Signal<String>`, `bulk_folder_mode: Signal<bool>`
- `description: Signal<String>`, `uploader: Signal<String>` (Default: "rsfdl"), `threads: Signal<u32>` (Default: 3)
- `encrypt_password: Signal<String>`
- `busy: Signal<bool>` — deaktiviert den Create-Button während der Erstellung

### Fehlerbehandlung

| Situation                      | Reaktion                                        |
|--------------------------------|-------------------------------------------------|
| Host leer                      | Error-Banner: "Host is required"                |
| Remote Path leer               | Error-Banner: "Remote path is required"         |
| FTP-Listing fehlgeschlagen     | Error-Banner: "FTP listing failed: {details}"   |
| Serialisierung fehlgeschlagen  | Error-Banner: "Serialization failed: {details}" |
| Datei schreiben fehlgeschlagen | Error-Banner: "Failed to write file: {details}" |
| Save-Dialog abgebrochen        | Keine Aktion (stille Rückkehr)                  |
