# Use Case: Dateien verifizieren

## Overview

**Use Case ID:** POST-001
**Use Case Name:** Dateien verifizieren
**Primary Actor:** System (automatisch nach Download)
**Goal:** Integritaet heruntergeladener Dateien anhand von Hash-Werten pruefen.
**Requirements:** FR-09
**Status:** Stable

## Preconditions

- Datei ist vollstaendig heruntergeladen.
- Container enthaelt optional Hash-Werte pro Datei.

## Main Success Scenario

1. System prueft, ob der FileEntry einen Hash enthaelt.
2. **[Hash vorhanden]** System berechnet den lokalen Hash mit dem passenden Algorithmus (MD5, CRC32, SHA1).
3. System vergleicht den berechneten Hash mit dem gespeicherten Hash.
4. Uebereinstimmung → HashResult: `Valid`.

## Alternative Flows

### A1: Kein Hash im Container

**Trigger:** FileEntry enthaelt keinen Hash (Schritt 1)
**Flow:**

1. System prueft, ob der FTP-Server Hash-Faehigkeiten hat (via `FEAT`).
2. **[Server unterstuetzt Hash]** System fragt den Hash serverseitig ab (XMD5, XSHA1, XCRC).
3. System vergleicht mit lokalem Hash.
4. **[Server unterstuetzt keinen Hash]** HashResult: `NoHash`.

### A2: Hash stimmt nicht ueberein

**Trigger:** Berechneter Hash weicht vom gespeicherten Hash ab (Schritt 3)
**Flow:**

1. System markiert die Datei als `Invalid`.
2. System meldet: „Hash-Mismatch fuer [Dateiname]: erwartet [X], berechnet [Y]."
3. **[Option: mark_failed_on_mismatch=true]** Task-Status → `Failed`.

## Postconditions

### Success Postconditions

- Jede Datei hat ein HashResult (`Valid`, `Invalid` oder `NoHash`).

### Failure Postconditions

- Datei ist als `Invalid` markiert.
- Bei `mark_failed_on_mismatch=true`: Task-Status → `Failed`.

## Business Rules

### BR-POST-001: Hash-Prioritaet

- Wenn mehrere Hash-Typen verfuegbar: SHA1 > MD5 > CRC32 (staerkster zuerst).

### BR-POST-002: Server-Hash-Fallback

- FEAT-Pruefung: `FEAT` → pruefte auf `MD5`, `XMD5`, `XSHA1`, `XCRC`.
- Server-Hash-Abfrage nur wenn kein lokaler Hash im Container und FTP-Verbindung noch aktiv.

## Input

- `file_entry: FileEntry` — mit optionalem Hash
- `local_path: String` — Pfad zur heruntergeladenen Datei
- `ftp_connection: Option<FtpConnection>` — fuer Server-Hash-Fallback

## Output

- `HashResult { expected, actual, hash_type, result }`
