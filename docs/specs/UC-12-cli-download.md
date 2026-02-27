# UC-12: CLI Download ausführen

## Scope

CLI-Befehl zum Herunterladen aller Dateien eines SFDL-Containers:

- Kommandozeilen-Argumente (Passwort, Zielverzeichnis, Threads)
- Passwort-File-Support (eine pro Zeile)
- indicatif Multi-Progress-Bars (global + per-File)
- Ctrl+C Cancellation
- Exit-Codes

## Akzeptanzkriterien (aus acceptance-tests.md)

- AT-18: `--help` zeigt Subcommands inkl. `download`

## Beteiligte Module

- `cli/src/main.rs` — `Commands::Download` Variant, `load_password_file()`
- `cli/src/commands/download.rs` — `run()`, Progress-Event-Loop, `truncate_name()`

## Befehl

```
rsfdl-cli download <datei.sfdl> [-p <password>] [--password-file <file>] [-d <dest>] [-t <threads>]
```

| Argument          | Kurz | Default    | Beschreibung                           |
|-------------------|------|------------|----------------------------------------|
| `file`            | —    | (required) | Pfad zur .sfdl-Datei                   |
| `--password`      | `-p` | —          | Entschlüsselungs-Passwort              |
| `--password-file` | —    | —          | Datei mit Passwörtern (eine pro Zeile) |
| `--dest`          | `-d` | `cwd`      | Zielverzeichnis                        |
| `--threads`       | `-t` | `3`        | Max. parallele Downloads               |

## API Design

```rust
// --- CLI Args (clap) ---

#[derive(Subcommand)]
enum Commands {
    Download {
        file: String,
        #[arg(short, long)]
        password: Option<String>,
        #[arg(long)]
        password_file: Option<String>,
        #[arg(short, long)]
        dest: Option<String>,
        #[arg(short, long, default_value = "3")]
        threads: u32,
    },
}

// --- Download-Logik ---

/// Führt den kompletten Download-Workflow aus.
pub async fn run(
    file: &str,
    password: Option<&str>,
    password_list: &[String],
    dest: Option<&str>,
    threads: u32,
);

/// Kürzt Dateinamen für Progress-Bar-Anzeige.
fn truncate_name(name: &str, max: usize) -> String;

/// Liest Passwort-Datei, eine pro Zeile, leere Zeilen ignoriert.
fn load_password_file(path: Option<&str>) -> Vec<String>;
```

## Implementierungsdetails

### Ablauf

```
1. SFDL-Datei lesen und parsen (parse_sfdl)
2. Settings laden (JSON-Datei, Default-Pfad oder --config-file)
3. Passwort-Listen mergen: [--password-file Einträge] + [settings.auto_password_list]
4. Entschlüsseln wenn nötig:
   a. -p angegeben → validate_password → decrypt_container
   b. Kein -p, aber merged password list → try_passwords → decrypt_container
   c. Kein Passwort → Fehler "File is encrypted. Provide a password with -p"
5. CLI-Flags auf Settings anwenden (--dest, --threads, --exclude)
6. File/BulkFolder-Counts und Gesamtgrösse berechnen und ausgeben
7. DownloadManager::new(container, &settings) → (manager, cancel_token, _)
8. Ctrl+C Handler spawnen → cancel_token.cancel()
9. Progress-Channel erstellen
10. Manager in Background-Task spawnen
11. Progress-Event-Loop mit indicatif
12. Exit-Code: 0 bei Erfolg, 1 bei Fehlern
```

### Progress-Anzeige (indicatif)

**Global Bar** (oben, bleibt immer sichtbar):

```
[2/5 files] [=========>----------] 2.4/4.2 GB 21.0 MiB/s ETA 01:23
```

- Style: `{prefix:.bold} [{bar:40.cyan/dim}] {bytes}/{total_bytes} {binary_bytes_per_sec} ETA {eta}`
- Prefix: `[files_done/files_total files]`

**Per-File Bars** (darunter, dynamisch):

```
  ...movie.part1.rar [============>-------] 800.0/1000.0 MB 12.3 MB/s
```

- Style: `  {prefix:.cyan} [{bar:30.green/dim}] {bytes}/{total_bytes} {bytes_per_sec}`
- Prefix: Dateiname, gekürzt auf 30 Zeichen

### Event-Verarbeitung

| ProgressEvent  | Aktion                                                                        |
|----------------|-------------------------------------------------------------------------------|
| `Started`      | files_total++, global_total_bytes += total_bytes, neue Per-File Bar erstellen |
| `BytesWritten` | Per-File Bar + Global Bar Position aktualisieren                              |
| `Completed`    | Per-File Bar `finish()`, files_done++                                         |
| `Skipped`      | files_total++, files_done++, `[SKIP] filename` auf stderr                     |
| `Failed`       | Per-File Bar `abandon_with_message("FAILED: error")`, files_done++            |
| `Cancelled`    | Per-File Bar `abandon_with_message("cancelled")`, files_done++                |
| `AllDone`      | Global Bar clearen, Summary auf stderr ausgeben, Loop beenden                 |

### Dateinamen-Kürzung

```rust
fn truncate_name(name: &str, max: usize) -> String {
    if name.len() <= max { name }
    else { format!("...{}", &name[name.len() - (max - 3)..]) }
}
```

### Exit-Codes

| Bedingung                                  | Exit-Code |
|--------------------------------------------|-----------|
| Alle Dateien erfolgreich oder übersprungen | `0`       |
| Mindestens eine Datei fehlgeschlagen       | `1`       |
| Parse/Decrypt-Fehler                       | `1`       |
| DownloadError (z.B. BulkFolder-Auflösung)  | `1`       |

### Unterschiede zur GUI

- Default-Zielverzeichnis: `cwd` (nicht `dirs::download_dir()`)
- Keine File-Selection (alle Dateien werden heruntergeladen)
- Kein Per-File Cancel (nur globaler Ctrl+C)
- Settings werden aus JSON geladen, CLI-Flags überschreiben einzelne Werte ohne die Datei zu ändern
