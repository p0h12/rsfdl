# rsfdl

A cross-platform SFDL downloader written in Rust. Alternative to [SFDL.NET](https://github.com/n0ix/SFDL.NET) with both a desktop GUI and a CLI.

## What is SFDL?

SFDL files are XML containers with FTP connection details and file lists, optionally encrypted with AES-128-CBC. rsfdl is a cross-platform alternative to the Windows-only SFDL.NET.

## Features

- **Cross-platform**: macOS, Linux, Windows
- **SFDL v2 & v3** container parsing
- **AES-128-CBC** decryption (UTF-8 + Latin-1 fallback)
- **Async FTP** downloads with configurable parallelism
- **Resume support**, retry logic, and hash verification
- **Real-time progress** display (progress bars in CLI, live UI updates in GUI)
- **Desktop GUI** (Dioxus) with file selection, password dialog, and settings
- **CLI** for headless/scripted usage and automation

## Project Structure

```
crates/
  core/   — Shared library: SFDL parsing, crypto, FTP client, download manager
  cli/    — Command-line interface (clap + indicatif)
  gui/    — Desktop application (Dioxus + Tailwind CSS)
```

## Build

```bash
# All crates
cargo build --release

# Individual crate
cargo build --release -p rsfdl-cli

# GUI release build
dx build -p rsfdl-gui --release --platform desktop
```

## Usage

### CLI

```bash
# Show container metadata
rsfdl info <file.sfdl> [-p <password>]

# List all files in container
rsfdl list <file.sfdl> [-p <password>]

# Download files
rsfdl download <file.sfdl> -p <password> [-d <dest>] [-t <threads>]
```

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --password` | Decryption password | — |
| `-d, --dest` | Download directory | current dir |
| `-t, --threads` | Max concurrent downloads (1–3) | 3 |

### GUI

```bash
dx serve --platform desktop
```

## Tech Stack

| Component | Library |
|-----------|---------|
| Async runtime | Tokio |
| XML parsing | quick-xml |
| Encryption | aes + cbc + md-5 |
| FTP | suppaftp (async-native-tls) |
| GUI | Dioxus 0.7 Desktop |
| CLI | clap 4 + indicatif |

## Testing

```bash
cargo test -p rsfdl-core
```

## License

MIT
