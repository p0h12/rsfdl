# DL-005: Download fortsetzen

**Use Case ID:** DL-005
**Requirements:** FR-06
**Primary Actor:** System (automatisch innerhalb DL-004)
**Trigger:** Wird als `<<extend>>` von DL-004 aktiviert, wenn eine lokale Datei teilweise vorhanden ist.
**Preconditions:** Lokale Datei existiert mit `bytes_downloaded > 0` und `bytes_downloaded < bytes_total`.
**Postconditions (Erfolg):** Download wird ab letztem Byte fortgesetzt.
**Postconditions (Fehlschlag):** Datei wird von Anfang an heruntergeladen (Fallback).

---

## Main Success Scenario

1. System prüft die lokale Dateigrösse.
2. System sendet `REST <bytes_downloaded>` an den FTP-Server.
3. FTP-Server bestätigt den Offset.
4. System sendet `RETR <remote_path>`.
5. FTP-Server liefert Daten ab dem angegebenen Offset.
6. System öffnet die lokale Datei im Append-Modus und schreibt die empfangenen Daten.

## Alternative Paths

**2a. Server unterstützt REST nicht:**
2a.1. FTP-Server antwortet mit Fehler auf `REST`.
2a.2. System löscht die lokale Datei.
2a.3. System startet den Download von Anfang an (Fallback).

**1a. Lokale Datei grösser als Remote:**
1a.1. `bytes_downloaded >= bytes_total`.
1a.2. System löscht die lokale Datei (möglicherweise beschädigt).
1a.3. System startet den Download von Anfang an.

## Business Rules

**BR-DL-012: Resume-Erkennung**

- Resume wird ausschliesslich über die lokale Dateigrösse erkannt.
- Es gibt keine separate State-Datei für den Download-Fortschritt.
- `bytes_downloaded == bytes_total` → Datei gilt als vollständig (`Skipped`).

## Input

- `local_path`: Pfad zur lokalen Datei
- `bytes_downloaded`: Aktuelle Grösse der lokalen Datei
- `ftp_connection`: Aktive FTP-Verbindung

## Output

- FTP-Stream ab Offset, bereit für Empfang
