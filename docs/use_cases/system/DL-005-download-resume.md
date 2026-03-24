# Use Case: Download fortsetzen

## Overview

**Use Case ID:** DL-005
**Use Case Name:** Download fortsetzen
**Primary Actor:** System (automatisch innerhalb DL-004)
**Goal:** Einen teilweise heruntergeladenen Download ab dem letzten Byte fortsetzen.
**Requirements:** FR-06
**Status:** Tested

## Preconditions

- Lokale Datei existiert mit `bytes_downloaded > 0` und `bytes_downloaded < bytes_total`.

## Main Success Scenario

1. System prüft die lokale Dateigrösse.
2. System sendet `REST <bytes_downloaded>` an den FTP-Server.
3. FTP-Server bestätigt den Offset.
4. System sendet `RETR <remote_path>`.
5. FTP-Server liefert Daten ab dem angegebenen Offset.
6. System öffnet die lokale Datei im Append-Modus und schreibt die empfangenen Daten.

## Alternative Flows

### A1: Server unterstützt REST nicht

**Trigger:** FTP-Server antwortet mit Fehler auf `REST` (Schritt 2)
**Flow:**

1. FTP-Server antwortet mit Fehler auf `REST`.
2. System löscht die lokale Datei.
3. System startet den Download von Anfang an (Fallback).

### A2: Lokale Datei grösser als Remote

**Trigger:** `bytes_downloaded >= bytes_total` (Schritt 1)
**Flow:**

1. `bytes_downloaded >= bytes_total`.
2. System löscht die lokale Datei (möglicherweise beschädigt).
3. System startet den Download von Anfang an.

## Postconditions

### Success Postconditions

- Download wird ab letztem Byte fortgesetzt.

### Failure Postconditions

- Datei wird von Anfang an heruntergeladen (Fallback).

## Business Rules

### BR-DL-012: Resume-Erkennung

- Resume wird ausschliesslich über die lokale Dateigrösse erkannt.
- Es gibt keine separate State-Datei für den Download-Fortschritt.
- `bytes_downloaded == bytes_total` → Datei gilt als vollständig (`Skipped`).

## Input

- `local_path`: Pfad zur lokalen Datei
- `bytes_downloaded`: Aktuelle Grösse der lokalen Datei
- `ftp_connection`: Aktive FTP-Verbindung

## Output

- FTP-Stream ab Offset, bereit für Empfang
