# Reference Project Analysis

## Overview

| Project                                                                   | Language    | Type            | Key Features                                              |
|---------------------------------------------------------------------------|-------------|-----------------|-----------------------------------------------------------|
| [SFDL.NET 3](https://github.com/n0ix/SFDL.NET)                            | VB.NET/WPF  | Desktop GUI     | Official Windows client, v2+v3, SmartThreadPool           |
| [SFDL.Container](https://github.com/n0ix/SFDL.Container)                  | VB.NET      | Library         | Canonical data models, AES encryption                     |
| [SFDLSaugerPro](https://github.com/DoctorW00/SFDLSaugerPro)               | C++/Qt      | Desktop GUI     | Best for macOS, IRC chat, RAR extraction                  |
| [SFDLSaugerCLI](https://github.com/DoctorW00/SFDLSaugerCLI)               | C++/Qt      | CLI + Web       | Custom AES, advanced SFDL fields, ANSI UI                 |
| [goSFDLSauger](https://github.com/DoctorW00/goSFDLSauger)                 | Go          | CLI + Web       | YAML config, MQTT, SOCKS5 proxy                           |
| [pySFDLSauger](https://github.com/DoctorW00/pySFDLSauger)                 | Python      | CLI + Web       | Latin-1 crypto, file watcher, Flask GUI                   |
| [jSFDLSauger](https://github.com/DoctorW00/jSFDLSauger)                   | Java        | CLI             | Java 7+, cross-platform                                   |
| [SFDL BASH Core Loader](https://github.com/DoctorW00/SFDLBashCore)        | Shell       | CLI             | Very simple CLI loader                                    |
| [sfdl-medialoader](https://github.com/efc5c264/sfdl-medialoader)          | Python      | Web (Flask)     | TMDB integration, Playwright, auto-organize               |
| [SFDL BASH - Loader](https://github.com/raz3r-code/sfdl-bash-loader)      | Shell       | CLI             | Best for Linux, v3.16                                     |
| SFDL BASH - Webinterface                                                  | PHP         | Web             | Addon: browser control for BASH Loader                    |
| [Docker SFDL BASH](https://hub.docker.com/r/n0ix/docker-sfdl-bash-loader) | Shell       | Docker          | Dockerized BASH Loader                                    |
| SFDL Bat-Loader                                                           | Batch       | CLI             | Loads multiple SFDLs with wget                            |
| SFDL Bat-Zilla                                                            | Batch       | CLI             | Loads multiple SFDLs with FileZilla                       |
| [jsfdlloader](https://gitlab.com/jsfdl/jsfdlloader)                       | Java/Native | CLI + WebSocket | Java 17+, rolling release                                 |
| [JSFDLSwing](https://gitlab.com/jsfdl/JSFDLSwing)                         | Java        | Desktop GUI     | Swing GUI, multi-threaded downloads, bulk mode, CI builds |
| pySFDL                                                                    | Python      | CLI             | Discontinued, limited SFDL support                        |
| [JSFDL Library](https://gitlab.com/jsfdl/JSFDL)                           | Java        | Library         | Core library, also lanterna (ncurses) TUI, open source    |

---

## Encryption Encoding Differences (Critical)

The single biggest interop pitfall: implementations disagree on password encoding for MD5 key derivation.

| Implementation         | Key Encoding                       | IV Handling                                        |
|------------------------|------------------------------------|----------------------------------------------------|
| SFDL.NET (VB.NET)      | UTF-8 only                         | First 16 bytes of base64-decoded ciphertext        |
| goSFDLSauger (Go)      | UTF-8 (Go default)                 | First 16 bytes of base64-decoded ciphertext        |
| pySFDLSauger (Python)  | **Latin-1** explicitly             | First 16 bytes of base64-decoded ciphertext        |
| SFDLSaugerCLI (C++/Qt) | `toLocal8Bit()` (locale-dependent) | **MD5(first 16 chars of b64 string)** — likely bug |
| rsfdl (Rust)           | UTF-8 first, Latin-1 fallback      | First 16 bytes of base64-decoded ciphertext        |

**Takeaway**: For ASCII-only passwords (the common case) all encodings produce identical bytes. The difference only matters for passwords with characters >127. rsfdl's UTF-8-first-then-Latin-1 strategy covers both camps. The SFDLSaugerCLI IV handling appears to be a bug (its files likely only work with its own encrypted containers).

---

## [SFDL.NET 3](https://github.com/n0ix/SFDL.NET) (n0ix/SFDL.NET) — The Reference

### Key Files

- `Modules/SFDLFileHelper.vb` — Version detection (lines 55-91), decryption mapping (lines 141-188), recursive FTP listing (lines 270-506), container fingerprinting (lines 93-139), UnRAR chain detection (lines 571-658)
- `Modules/FTPHelper.vb` — FTP client setup (lines 7-131), LIST parser with 5 platform formats (lines 229-260)
- `Classes/DownloadHelper.vb` — FTP download with resume, progress, throttling (985 lines total)
- `Classes/Models/ContainerSession.vb` — Session tracking with GUID + fingerprint
- `Classes/Models/DownloadItem.vb` — Per-file status, progress, retry state (450 lines)
- `Classes/SFDL Converter/Converter.vb` — v2→v3 normalization

### Architecture

- MVVM pattern (WPF)
- SmartThreadPool for concurrent downloads (configurable max threads, default 3)
- WorkItemsGroup per container session
- FTP client: ArxOne.Ftp (forked C# library, `SFDL.FTP.csproj`)
- Session persistence: XML files in `%APPDATA%\SFDL.NET 3\Sessions\`
- NuGet: MahApps.Metro (UI), NLog (logging), SmartThreadPool, ControlzEx, Hardcodet.NotifyIcon.Wpf

### Version Detection

- Reads XML with `Text.Encoding.Default` (system default)
- Tries `ContainerVersion` tag first (v3 format)
- Falls back to `SFDLFileVersion` tag (v2 format)
- Returns 0 if version unknown

### Download Workflow

1. Open SFDL → detect version → decrypt if needed
2. Create ContainerSession with unique GUID + fingerprint
3. Generate DownloadItems from FileItems/BulkFolders
4. Apply blacklist filters (regex patterns downloaded from GitHub)
5. Create FTP client (cached by `base64(host+port+user+pass)`, with `SyncLock`)
6. Queue downloads in SmartThreadPool
7. Per file: check disk space → get size → check resume → RETR stream → 8KB buffer writes → speed throttle
8. Post: hash verification → RAR extraction (both queued via separate SmartThreadPool)

### FTP Connection & Session Management

- **Connection pooling**: Dictionary keyed by `base64(host+port+user+pass)`, thread-safe with `SyncLock`
- **Session modes**: Single session (one FTP session reused) vs multi session (new session per download item)
- **SSL/TLS mapping**: `FtpES` (explicit TLS), `FtpS` (implicit SSL), plain `Ftp`
- **UTF8 negotiation**: Sends `OPTS UTF8 ON` if server advertises `UTF8` feature
- **Anonymous fallback**: Uses `anonymous` / `Password` when `AuthRequired = False`
- **Basic availability test**: Ping + port test before first connection

### FTP Listing Strategy (Fallback Chain)

1. **Vendor detection**: Special handling for certain FTP servers (CWD + LIST)
2. **MLSD** (preferred): Machine-readable listing if server supports it
3. **CWD + LIST**: Change to directory, then LIST with no argument
4. **LIST(path)**: Direct LIST with path parameter
5. **LIST parser** (`TryParseLine`): Tries 5 platform parsers in order — Unix, UnixAlt, Windows, WindowsFileZilla, Generic

### FTP Error Handling

- 421: Server Full → retry
- 530/430: Auth Error → retry
- 425/426: Connection Error → retry
- 434: Server Down → no retry
- 450-452/501/550: File Not Found → no retry
- String-based: `"maximum login limit has been reached."` → retry

### Hash Verification (Post-Download)

- Queued via separate SmartThreadPool after download completes
- **Server-side hashing**: Checks FTP `FEAT` for `MD5`/`XMD5`/`XSHA1`/`XCRC` commands, sends command and parses reply code 250/251
- **Client-side verification**: Uses RHash library (external C# project) for MD5, SHA1, CRC32
- CRC32 hashes padded to 8 chars with leading zeros before comparison

### Speed Throttling

- Per-thread bandwidth limiting: `max_bytes_per_second / active_download_count`
- `ThrottleByteTransfer()` called in the read loop after each buffer write

### Additional Features

- **Container fingerprinting**: Base64-encoded concatenation of metadata (threads + host + port + user + packages + name + uploader + filename) for deduplication
- **UnRAR chain detection**: Regex `^((?!\.part(?!0*1\.rar$)\d+\.rar$).)*\.(?:rar|r?0*1)$` — detects first-part RAR archives for "Instant Video" streaming (download first 2 parts, start playback)
- **Blacklist system**: Downloads regex patterns from GitHub to filter unwanted files (e.g., `.exe` in video archives)
- **Disk space check**: `GetFreeDiskSpaceForPath()` before each download
- **Long path support**: Checks Windows registry `SYSTEM\CurrentControlSet\Control\FileSystem\LongPathsEnabled`, enables `\\?\` prefix for paths >260 chars

### Settings

- Download directory, max threads, max retries, retry wait time
- File handling: resume vs overwrite
- Package subfolder creation
- Auto-password list (`%APPDATA%\SFDL.NET 3\sfdl_passwords.def`)
- Max download speed (global, divided across threads)

---

## [SFDL.Container](https://github.com/n0ix/SFDL.Container) (n0ix/SFDL.Container) — Data Models

### Models (with defaults)

- `Container`: Description="", Uploader="", ContainerVersion=0, Encrypted=False, Connection, Packages=[], **MaxDownloadThreads=3**
- `Connection`: Host="", **Port=21**, Username="", Password="", AuthRequired=False, DataConnectionType=Passive, DataType=Binary, CharacterEncoding=Standard, SSLProtocol=None, **ConnectTimeout=10**, **CommandTimeout=10**
- `Package`: Name="", BulkFolderMode=False, FileList=[], BulkFolderList=[]
- `FileItem`: FileName="", DirectoryRoot="", DirectoryPath="", FullPath="", FileSize=0, HashType=None, FileHash="", PackageName=""
- `BulkFolder`: BulkFolderPath="", PackageName=""

### Enums

- `HashType`: MD5, CRC, SHA1, None (XmlEnum: `"default"` → None, `""` → None)
- `FTPDataType`: ASCII, Binary (XmlEnum: `"default"` → Binary, `""` → Binary)
- `FTPDataConnectionType`: Passive, Active, ExtendedPassive (XmlEnum: `"default"` → Passive, `""` → Passive)
- `CharacterEncoding`: Standard, UTF8, UTF7, ASCII (XmlEnum: `"default"` → UTF8, `""` → UTF8)
- `SslProtocols`: None, SSL2, SSL3, TLS, TLS11, TLS12 (uses .NET `System.Security.Authentication.SslProtocols`)
- `FTPListOption`: (exists in codebase, not widely used)

### Encryption

- AES-Rijndael, 128-bit, CBC mode
- Key: `MD5(Encoding.UTF8.GetBytes(password))`
- IV: Random on encrypt, first 16 bytes of base64-decoded ciphertext on decrypt
- Ciphertext layout: `Base64(IV[16] || AES-CBC(plaintext_with_PKCS7))`
- Error: Throws `FalsePasswordException` on decryption failure

---

## [goSFDLSauger](https://github.com/DoctorW00/goSFDLSauger) (DoctorW00/goSFDLSauger) — Clean Go Reference

### Key Files

- `sfdl.go` (lines 138-194): Decryption with AES-CBC, MD5 key, PKCS5 unpadding
- `ftp.go` (lines 1-621): Recursive listing, download queue, resume, progress
- `config.go`: YAML configuration
- `unpacker.go`: Post-download RAR/ZIP extraction
- `webserver.go`: Embedded HTTP server with WebSocket for live progress
- `mqtt.go`: MQTT status publishing

### SFDL Parsing

- `encoding/xml` with struct tags, no explicit v2/v3 branching
- Default password: hardcoded string (line 17)
- Global variables for parsed server data

### Encryption

- Key: `MD5(password)` — Go strings are UTF-8 by default
- IV: First 16 bytes of base64-decoded ciphertext
- PKCS5 unpad implementation (lines 185-194)
- Base64 validation before decryption attempt

### FTP Client

- **Library**: `jlaffaye/ftp`
- **Buffer size**: 32KB (`make([]byte, 32*1024)`)
- **EPSV disabled**: `ftp.DialWithDisabledEPSV(true)`
- **SOCKS5 proxy**: Custom dialer via `golang.org/x/net/proxy`, with authentication
- **Context-based cancellation**: `context.CancelFunc` per connection
- **Connection pooling**: `map[string]context.CancelFunc` with `sync.Mutex`
- **Resume**: `RetrFrom(filename, offset)`, file opened with `os.O_APPEND|os.O_WRONLY`
- **Timeout**: 30 seconds (configurable)

### Download Manager

- **Threading**: Goroutines + channel-based work queue (`downloadQueue chan string`)
- **Concurrency**: Configurable `MaxConcurrentDownloads` (default 3), semaphore pattern
- **Progress**: `mpb/v8` library — per-file bars + global progress bar
- **Retry**: 553/530/421 errors trigger retry
- **File list format**: Internal encoding `path;;;size`

### Additional Features

- YAML config file
- MQTT notifications (download status)
- Web server on :8080 with WebSocket for live progress
- UnRAR/UnZIP post-download extraction

---

## [pySFDLSauger](https://github.com/DoctorW00/pySFDLSauger) (DoctorW00/pySFDLSauger) — Python Reference

### Key Files

- `pySFDLSauger.py` (1,448 lines — single-file implementation)

### SFDL Parsing

- `xml.etree.ElementTree` for XML parsing
- Extracts: Encrypted, Description, Uploader, Host, Port, Username, Password, BulkFolderPath
- Reads `SFDLFileVersion` but doesn't branch logic

### Encryption

- **Key derivation**: `hashlib.md5(password.encode('latin-1')).digest()` — **explicitly Latin-1!**
- IV: First 16 bytes of base64-decoded ciphertext
- Library: `cryptography.hazmat` (fallback to `Crypto`/`Cryptodome`)
- PKCS7 unpad: Manual `padding_length = decrypted_message[-1]`

### FTP Client

- **Library**: Built-in `ftplib.FTP`
- **Listing fallback**: MLSD (with `facts=["type", "size"]`) → NLST + per-file `SIZE`
- **Binary mode**: `TYPE I` command
- **PASV mode**: `ftp.set_pasv(True)`
- **Encoding**: `ftp.encoding = "utf-8"`
- **SOCKS5 proxy**: Custom `Proxy` class using `socks.socksocket()`

### Download Manager

- **Threading**: `ThreadPoolExecutor(max_workers=10)` + `threading.Semaphore`
- **Per-thread FTP**: Separate FTP connection per thread stored in `ftp_sessions` dict
- **Progress**: `tqdm` library with custom bar format
- **Retry**: `time.sleep(10)` then reconnect on 553/530/421 errors
- **Resume**: Checks local file size, appends if partial

### Additional Features

- **File watcher mode**: Monitors directory for new `.sfdl` files, auto-downloads
- **Flask web GUI**: Embedded web server with WebSocket for browser control
- **UnRAR**: Calls `RarExtractor` class post-download
- **File exclusion**: Configurable patterns (`.nfo`, `sample`, etc.)
- **Update checker**: Checks GitHub for newer version

---

## [SFDLSaugerCLI](https://github.com/DoctorW00/SFDLSaugerCLI) (DoctorW00/SFDLSaugerCLI) — C++/Qt Reference

### Key Files

- `sfdl.cpp` (416 lines): SFDL parsing + decryption
- `ftpdownload.cpp` (260 lines): FTP download with resume
- `sauger.cpp` (785 lines): Download orchestration, console UI
- `qaesencryption.cpp` (465 lines): Full AES implementation from scratch

### SFDL Parsing

- `QDomDocument` for XML parsing
- **Parses additional fields** not in other implementations: `EncryptionMode`, `ListMethod`, `SpecialServerMode`, `ForceSingleConnection`, `DataStaleDetection`, `DefaultPath`
- Returns key-value pairs as pipe-separated `QStringList`

### Encryption (Caution: Divergent IV Handling)

- **Key derivation**: `QCryptographicHash::hash(password.toLocal8Bit(), QCryptographicHash::Md5)` — locale-dependent, often Latin-1
- **IV handling**: `MD5(first 16 chars of base64 string)` — **differs from all other implementations** which use the raw first 16 bytes of the decoded ciphertext. Likely a bug; containers encrypted by SFDLSaugerCLI may not be decryptable by other tools.
- **Custom AES**: Full AES-128/192/256 implementation in `qaesencryption.cpp` (SubBytes, ShiftRows, MixColumns, key expansion) with PKCS7/ISO/ZERO padding
- Skips first 16 bytes of decrypted output (`.mid(16)`)

### FTP Client

- **Library**: Custom/legacy `QFtp` (not in modern Qt)
- **Resume**: Sets `ftp->m_fileSize = file->size()` before `ftp->get()`
- **Error codes**: QFtp enums — NoError(0), UnknownError(1), HostNotFound(2), ConnectionRefused(3), NotConnected(4)
- **SOCKS5 proxy**: Code present but commented out
- **Timeout**: QTimer-based

### Download Manager

- Qt `QThread` with `moveToThread` pattern
- Signal/slot architecture for progress updates
- Rich ANSI terminal output with colors, ASCII art, custom progress bars

---

## [sfdl-medialoader](https://github.com/efc5c264/sfdl-medialoader) (efc5c264/sfdl-medialoader) — Media Intelligence

### Key Files

- `main.py`: Entry point
- `src/downloader.py` (2,313 lines): Download + media organization logic
- `src/download_sfdl.py` (203 lines): Forum automation + SFDL extraction

### Forum Automation (Playwright)

- Headless Chromium browser for forum login
- Clicks "Thanks" button to reveal download links
- SFDL URL extraction via regex: `r'https?://(?:download\.)?sfdl\.net/enc/[^"\s<>]+'`
- Password list support (`passwords.txt`)

### Encryption

- Uses `Crypto.Cipher.AES` / `Cryptodome.Cipher.AES`
- Same AES-128-CBC pattern as pySFDLSauger

### Media Organization

- **TMDB API**: Movie/series/documentary detection from filenames
- **Organization rules**:
    - Single video → `/movies/` with metadata cleanup
    - Series → `/serien/Show Name/Season XX/`
    - Documentary → `/docus/` (flattened)
    - Also detects: software, games, eBooks
- **Cleanup**: Removes `.nfo`, `.jpg`, `.sub`, `.idx`, sample files, proof folders
- **Archive handling**: Multi-part RAR (`.r00`, `.r01`, `.part1.rar`), TAR; removes archives after extraction

---

## Common Patterns Across All Implementations

### SFDL Parsing

All use standard XML parsing → same field structure → optional decryption. Version detection via `ContainerVersion` (v3) or `SFDLFileVersion` (v2).

### Encryption

All implement AES-128-CBC with MD5 key derivation. **Encoding differs** (see table above) — only matters for non-ASCII passwords. IV is consistently the first 16 bytes of the decoded ciphertext (except SFDLSaugerCLI — likely bug).

### FTP Operations

1. Connect with host:port + credentials (anonymous fallback if no auth required)
2. **Listing fallback**: MLSD (preferred) → LIST (various forms) → NLST + SIZE
3. RETR for downloads with optional REST for resume
4. Recursive directory walk for BulkFolder mode
5. **Buffer sizes**: 8KB (SFDL.NET), 32KB (Go) — Go is more modern default

### FTP Error Codes (Retry Logic)

All implementations retry on these codes:

- **421**: Server full / too many connections
- **530**: Authentication error (retry with same or different credentials)
- **553**: File transfer timeout

SFDL.NET additionally handles: 430 (auth), 425/426 (connection), 434 (server down, no retry), 450-452/501/550 (file not found, no retry).

### Hash Verification

- SFDL.NET: Full support — server-side (MD5/XMD5/XSHA1/XCRC FTP commands) + client-side (RHash library)
- Others: Minimal or no hash verification

### Progress Tracking

All track per-file progress (bytes downloaded / total bytes) and emit updates to UI:

- SFDL.NET: WPF data binding via DownloadItem properties
- Go: `mpb` progress bar library
- Python: `tqdm` library
- C++/Qt: Custom ANSI terminal progress bars

### Configuration

- Download directory, max threads, retry settings, password list
- Stored as: XML (SFDL.NET), YAML (Go), .env/args (Python), QSettings (Qt)
