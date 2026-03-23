# CLI-001: `rsfdl info`

**Interface Spec ID:** CLI-001
**Interface:** CLI (headless)
**Implementiert:** SFDL-001, SFDL-002, SFDL-003

---

## Beschreibung

Zeigt die Metadaten eines SFDL-Containers an, ohne einen Download zu starten. Nützlich für Scripting und schnelle Inspektion.

## Syntax

```
rsfdl info <datei.sfdl> [--password <pw>] [--json]
```

## Parameter

| Parameter         | Pflicht | Beschreibung                                 |
|-------------------|---------|----------------------------------------------|
| `<datei.sfdl>`    | ja      | Pfad zur SFDL-Datei                          |
| `--password <pw>` | nein    | Passwort für verschlüsselte Container        |
| `--json`          | nein    | Ausgabe als JSON statt menschenlesbarem Text |

## Verhalten

1. System öffnet und parst die SFDL-Datei (→ SFDL-001).
2. Falls verschlüsselt und kein `--password`: Auto-Passwort-Liste wird probiert (→ SFDL-002).
3. Falls immer noch verschlüsselt: Passwort-Prompt auf stdin (nur wenn Terminal interaktiv).
4. System zeigt Container-Metadaten an.

## Ausgabe (Standard)

```
Container: Movie.Pack.2024
Uploader:  user123
Host:      ftp.example.com:21 (FTP)
Pakete:    2
Dateien:   47
Grösse:    4.2 GB
Verschlüsselt: ja (entschlüsselt mit Auto-Passwort)
```

## Ausgabe (JSON)

```json
{
    "description": "Movie.Pack.2024",
    "uploader": "user123",
    "host": "ftp.example.com",
    "port": 21,
    "protocol": "FTP",
    "encrypted": true,
    "packages": 2,
    "total_files": 47,
    "total_bytes": 4509715660
}
```

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Erfolg                                   |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungültiges SFDL-Format                   |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
