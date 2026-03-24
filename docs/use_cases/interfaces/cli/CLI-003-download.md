# Use Case: rsfdl download

## Overview

**Use Case ID:** CLI-003
**Use Case Name:** rsfdl download
**Primary Actor:** Benutzer
**Goal:** SFDL-Container öffnen und selektierte Dateien herunterladen.
**Implements:** DL-001 bis DL-008, POST-001, POST-002, POST-003
**Interface:** CLI (headless)
**Status:** Stable

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- Eine SFDL-Datei existiert im Dateisystem.
- Ausreichend Speicherplatz im Zielverzeichnis (wenn `--strict-disk-check`).

## Syntax

`rsfdl download <datei.sfdl> [Optionen]`

## Parameter

| Parameter             | Pflicht | Standard          | Beschreibung                                |
|-----------------------|---------|-------------------|---------------------------------------------|
| `<datei.sfdl>`        | ja      | —                 | Pfad zur SFDL-Datei                         |
| `--password <pw>`     | nein    | Einstellung (CFG) | Passwort fuer verschluesselte Container     |
| `--output <dir>`      | nein    | Einstellung (CFG) | Zielverzeichnis                             |
| `--threads <n>`       | nein    | Einstellung (CFG) | Max. parallele Downloads                    |
| `--max-speed <KB/s>`  | nein    | Einstellung (CFG) | Bandbreitenbegrenzung (0 = unbegrenzt)      |
| `--exclude <pattern>` | nein    | Einstellung (CFG) | Zusaetzliches Ausschlussmuster (mehrfach)   |
| `--no-exclude`        | nein    | false             | Alle Ausschlussmuster deaktivieren          |
| `--retries <n>`       | nein    | Einstellung (CFG) | Max. Retry-Versuche                         |
| `--retry-delay <s>`   | nein    | Einstellung (CFG) | Wartezeit zwischen Retries                  |
| `--strict-disk-check` | nein    | Einstellung (CFG) | Bei zu wenig Speicherplatz abbrechen        |
| `--extract`           | nein    | Einstellung (CFG) | Auto-Extraktion nach Download               |
| `--delete-archives`   | nein    | Einstellung (CFG) | Archive nach Extraktion loeschen            |
| `--verify`            | nein    | false             | Hash-Verifikation nach Download (geplant)   |
| `--speedreport`       | nein    | false             | Speed-Report auf stdout (geplant)           |
| `--json`              | nein    | false             | JSON-Ausgabe (geplant, -> CLI-004)          |
| `--quiet`             | nein    | false             | Keine Fortschrittsanzeige, nur Ergebnis     |

## Main Success Scenario

1. Benutzer ruft `rsfdl download <datei.sfdl>` auf.
2. System öffnet und entschlüsselt den Container (-> SFDL-001, SFDL-002).
3. System löst den Inhalt auf (-> SFDL-003).
4. System wendet Ausschlussmuster an (-> DL-002).
5. System erstellt die Selektion (-> DL-001): Alle nicht-ausgeschlossenen Dateien.
6. System prüft den Speicherplatz (-> DL-003).
7. System startet den Download (-> DL-004) mit Fortschrittsanzeige auf stderr (-> CLI-004).
8. Nach Abschluss: optionale Verifikation (-> POST-001), Extraktion (-> POST-002).
9. System gibt Ergebnis-Zusammenfassung auf stdout aus.
10. Optional: Speed-Report auf stdout (-> POST-003).

## Alternative Flows

### A1: Passwort erforderlich

**Trigger:** Container verschlüsselt, kein Passwort passt (Schritt 2)
**Flow:** Wie CLI-001 A1/A5.

### A2: Nicht genug Speicherplatz

**Trigger:** `--strict-disk-check` aktiv und zu wenig Platz (Schritt 6)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 6.

### A3: Teilweise fehlgeschlagen

**Trigger:** Einige Dateien schlagen nach Retries fehl (Schritt 7)
**Flow:**

1. System listet fehlgeschlagene Dateien in der Zusammenfassung.
2. Exit-Code 10.

### A4: Alle Downloads fehlgeschlagen

**Trigger:** Keine Datei konnte heruntergeladen werden (Schritt 7)
**Flow:**

1. System listet alle Fehler in der Zusammenfassung.
2. Exit-Code 11.

### A5: Abbruch durch Signal

**Trigger:** Benutzer sendet SIGINT (Ctrl+C) oder SIGTERM (Schritt 7)
**Flow:**

1. Einmaliges SIGINT: Graceful Shutdown (-> DL-006), dann Ergebnis ausgeben.
2. SIGINT innerhalb 2s erneut: Sofortiger Abbruch.
3. Exit-Code 12.

### A6: CLI-Parameter überschreiben Settings

**Trigger:** Benutzer gibt `--threads`, `--retries`, etc. an (Schritt 1)
**Flow:**

1. CLI-Parameter ueberschreiben die konfigurierten Werte fuer diese Ausfuehrung (-> CLI-004, BR-CLI-004-006).
2. Werte werden nicht in die Datei zurueckgeschrieben.

## Postconditions

### Success Postconditions

- Alle selektierten Dateien sind heruntergeladen.
- Optionale Post-Processing-Schritte sind abgeschlossen.
- Ergebnis wurde auf stdout ausgegeben.

### Failure Postconditions

- Fehlgeschlagene Dateien bleiben als partielle Downloads auf Disk (für Resume).
- Ergebnis mit Fehlerliste auf stdout.

## Business Rules

### BR-CLI-003-001: Fortschrittsanzeige

- Fortschritt auf stderr mit Progress-Bars (wenn Terminal).
- `--quiet`: Keine Fortschrittsanzeige.
- `--json`: JSON-Lines auf stderr (-> CLI-004).

### BR-CLI-003-002: Signal-Handling

- SIGINT einmal: Graceful Shutdown.
- SIGINT zweimal (< 2s): Sofortiger Abbruch.
- SIGTERM: Wie einmaliges SIGINT.

Weitere Regeln: -> CLI-004 (Cross-Cutting): Parameter-Prioritaet (BR-CLI-004-006), Exit-Codes (BR-CLI-004-007), Quiet-Modus (BR-CLI-004-008).

## Fortschrittsanzeige (stderr)

Zeigt pro aktiver Datei: Zähler, Dateiname, Fortschrittsbalken, Prozent, Geschwindigkeit, ETA. Darunter eine Gesamtzeile mit aggregiertem Fortschritt, Durchschnittsgeschwindigkeit und Gesamt-ETA. Updates per `\r` wenn Terminal, sonst eine Zeile pro Datei-Abschluss.

## Ergebnis-Ausgabe (stdout)

Zusammenfassung mit Anzahl OK/fehlgeschlagen/übersprungen, Dauer, Gesamtgrösse, Durchschnittsgeschwindigkeit. Bei Fehlern: Liste der fehlgeschlagenen Dateien mit Fehlertyp und Retry-Anzahl.

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Alle Dateien erfolgreich                 |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungültiges SFDL-Format                   |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
| 5    | FTP-Fehler bei BulkFolder-Auflösung      |
| 6    | Nicht genug Speicherplatz (strict mode)  |
| 10   | Teilweise fehlgeschlagen                 |
| 11   | Alle Downloads fehlgeschlagen            |
| 12   | Abbruch durch Signal (SIGINT/SIGTERM)    |
