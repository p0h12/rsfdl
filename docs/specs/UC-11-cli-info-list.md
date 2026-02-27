# UC-11: CLI Info und List

## Scope

CLI-Befehle zum Anzeigen von SFDL-Container-Informationen ohne Download.
Basis für UC-12 (CLI Download) und für Debugging/Scripting.

## Befehle

### `rsfdl-cli info <datei.sfdl> [-p <password>]`

Zeigt Container-Übersicht:

```
Container: Test.Release.2026.1080p
Uploader:  testuser
Version:   10
Encrypted: yes (decrypted)
Server:    ftp.example.com:21 (Plain FTP, Passive)
Packages:  1
Files:     2
Total:     200.0 MB
```

### `rsfdl-cli list <datei.sfdl> [-p <password>]`

Zeigt Dateiliste:

```
Package: Package1

  /releases/test/movie.part1.rar    100.0 MB  [MD5]
  /releases/test/movie.part2.rar    100.0 MB

2 files, 200.0 MB total
```

Bei BulkFolder-Modus (ohne FTP-Auflösung):

```
Package: Package1 (Bulk Folder Mode)

  [DIR] /releases/test/

1 bulk folder (file listing requires FTP connection)
```

## Akzeptanzkriterien

- AT-07: CLI info zeigt korrekte Container-Daten
- AT-08: CLI list zeigt Dateien mit Grössen
- AT-16: Verschlüsselt ohne Passwort → Fehler
- AT-18: --help zeigt Usage

## Beteiligte Module

- `cli/src/main.rs` — Clap Commands
- `cli/src/commands/info.rs` — Info-Logik
- `cli/src/commands/list.rs` — List-Logik
- Core: `sfdl/parser`, `sfdl/crypto`

## Hilfsfunktion

`format_bytes(bytes: u64) -> String` — Formatiert Bytes human-readable (KB, MB, GB).
