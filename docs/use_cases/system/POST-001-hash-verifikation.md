# Use Case: Dateien verifizieren

## Overview

**Use Case ID:** POST-001
**Use Case Name:** Dateien verifizieren
**Primary Actor:** System (automatisch nach Download)
**Goal:** Integrität heruntergeladener Dateien anhand von Hash-Werten prüfen.
**Requirements:** FR-09
**Status:** Stable

## Preconditions

- Datei ist vollständig heruntergeladen.
- Container enthält optional Hash-Werte pro Datei.

## Main Success Scenario

1. System prüft, ob der FileEntry einen Hash enthält.
2. **[Hash vorhanden]** System berechnet den lokalen Hash mit dem passenden Algorithmus (MD5, CRC32, SHA1).
3. System vergleicht den berechneten Hash mit dem gespeicherten Hash.
4. Übereinstimmung → HashResult: `Valid`.

## Alternative Flows

### A1: Kein Hash im Container

**Trigger:** FileEntry enthält keinen Hash (Schritt 1)
**Flow:**

1. System prüft, ob der FTP-Server Hash-Fähigkeiten hat (via `FEAT`).
2. **[Server unterstützt Hash]** System fragt den Hash serverseitig ab (XMD5, XSHA1, XCRC).
3. System vergleicht mit lokalem Hash.
4. **[Server unterstützt keinen Hash]** HashResult: `NoHash`.

### A2: Hash stimmt nicht überein

**Trigger:** Berechneter Hash weicht vom gespeicherten Hash ab (Schritt 3)
**Flow:**

1. System markiert die Datei als `Invalid`.
2. System meldet: "Hash-Mismatch für [Dateiname]: erwartet [X], berechnet [Y]."
3. **[Option: mark_failed_on_mismatch=true]** Task-Status → `Failed`.

## Postconditions

### Success Postconditions

- Jede Datei hat ein HashResult (`Valid`, `Invalid` oder `NoHash`).

### Failure Postconditions

- Datei ist als `Invalid` markiert.
- Bei `mark_failed_on_mismatch=true`: Task-Status → `Failed`.

## Business Rules

### BR-POST-001: Hash-Priorität

- Wenn mehrere Hash-Typen verfügbar: SHA1 > MD5 > CRC32 (stärkster zuerst).

### BR-POST-002: Server-Hash-Fallback

- FEAT-Prüfung: `FEAT` → prüfte auf `MD5`, `XMD5`, `XSHA1`, `XCRC`.
- Server-Hash-Abfrage nur wenn kein lokaler Hash im Container und FTP-Verbindung noch aktiv.

## Input

- `file_entry: FileEntry` — mit optionalem Hash
- `local_path: String` — Pfad zur heruntergeladenen Datei
- `ftp_connection: Option<FtpConnection>` — für Server-Hash-Fallback

## Output

- `HashResult { expected, actual, hash_type, result }`
