# Use Case: rsfdl download

## Overview

**Use Case ID:** CLI-003
**Use Case Name:** rsfdl download
**Primary Actor:** Benutzer
**Goal:** SFDL-Container oeffnen und selektierte Dateien herunterladen.
**Implements:** DL-001 bis DL-008, POST-001, POST-002, POST-003
**Status:** Stable

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- Eine SFDL-Datei existiert im Dateisystem.
- Ausreichend Speicherplatz im Zielverzeichnis (wenn `--strict-disk-check`).

## Syntax

`rsfdl download <datei.sfdl> [Optionen]`

## Parameter

| Parameter             | Pflicht | Standard | Beschreibung                                |
|-----------------------|---------|----------|---------------------------------------------|
| `<datei.sfdl>`        | ja      | —        | Pfad zur SFDL-Datei                         |
| `--password <pw>`     | nein    | CFG      | Passwort fuer verschluesselte Container     |
| `--output <dir>`      | nein    | CFG      | Zielverzeichnis                             |
| `--threads <n>`       | nein    | CFG      | Max. parallele Downloads                    |
| `--max-speed <KB/s>`  | nein    | CFG      | Bandbreitenbegrenzung (0 = unbegrenzt)      |
| `--exclude <pattern>` | nein    | CFG      | Zusaetzliches Ausschlussmuster (mehrfach)   |
| `--no-exclude`        | nein    | false    | Alle Ausschlussmuster deaktivieren          |
| `--retries <n>`       | nein    | CFG      | Max. Retry-Versuche                         |
| `--retry-delay <s>`   | nein    | CFG      | Wartezeit zwischen Retries                  |
| `--strict-disk-check` | nein    | CFG      | Bei zu wenig Speicherplatz abbrechen        |
| `--extract`           | nein    | CFG      | Auto-Extraktion nach Download               |
| `--delete-archives`   | nein    | CFG      | Archive nach Extraktion loeschen            |
| `--verify`            | nein    | false    | Hash-Verifikation nach Download (geplant)   |
| `--speedreport`       | nein    | false    | Speed-Report auf stdout (geplant)           |
| `--json`              | nein    | false    | JSON-Ausgabe (geplant, -> CLI-CC)           |
| `--quiet`             | nein    | false    | Keine Fortschrittsanzeige, nur Ergebnis     |

## Main Success Scenario

1. Benutzer ruft `rsfdl download <datei.sfdl>` auf.
2. System oeffnet und parst die SFDL-Datei (-> SFDL-001).
3. Falls verschluesselt: System entschluesselt den Container (-> SFDL-002).
4. System loest den Inhalt auf (-> SFDL-003).
5. System wendet Ausschlussmuster an (-> DL-002).
6. System erstellt die Selektion (-> DL-001): Alle nicht-ausgeschlossenen Dateien.
7. System prueft den Speicherplatz (-> DL-003).
8. System startet den Download (-> DL-004) mit Fortschrittsanzeige auf stderr (-> CLI-CC).
9. Nach Abschluss: optionale Verifikation (-> POST-001), Extraktion (-> POST-002).
10. System gibt Ergebnis-Zusammenfassung auf stdout aus.
11. Optional: Speed-Report auf stdout (-> POST-003).

## Alternative Flows

### A1: Passwort erforderlich (nicht-interaktiv)

**Trigger:** Container ist verschluesselt, kein Passwort passt, kein interaktives Terminal (Schritt 2)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 3.

### A2: Falsches Passwort

**Trigger:** `--password` angegeben, aber falsches Passwort (Schritt 2)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 4.

### A3: Datei nicht gefunden

**Trigger:** SFDL-Datei existiert nicht (Schritt 2)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 1.

### A4: Ungueltiges SFDL-Format

**Trigger:** Datei kann nicht geparst werden (Schritt 2)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 2.

### A5: Interaktiver Passwort-Prompt

**Trigger:** Container ist verschluesselt, kein Passwort passt, stderr ist ein Terminal (Schritt 2)
**Flow:**

1. System zeigt Passwort-Prompt auf stdin.
2. Benutzer gibt Passwort ein.
3. System entschluesselt den Container.
4. Use Case faehrt mit Schritt 3 fort.

### A6: Nicht genug Speicherplatz

**Trigger:** `--strict-disk-check` aktiv und zu wenig Platz (Schritt 6)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 6.

### A7: Teilweise fehlgeschlagen

**Trigger:** Einige Dateien schlagen nach Retries fehl (Schritt 7)
**Flow:**

1. System listet fehlgeschlagene Dateien in der Zusammenfassung.
2. Exit-Code 10.

### A8: Alle Downloads fehlgeschlagen

**Trigger:** Keine Datei konnte heruntergeladen werden (Schritt 7)
**Flow:**

1. System listet alle Fehler in der Zusammenfassung.
2. Exit-Code 11.

### A9: Abbruch durch Signal

**Trigger:** Benutzer sendet SIGINT (Ctrl+C) oder SIGTERM (Schritt 7)
**Flow:**

1. Einmaliges SIGINT: Graceful Shutdown (-> DL-006), dann Ergebnis ausgeben.
2. SIGINT innerhalb 2s erneut: Sofortiger Abbruch.
3. Exit-Code 12.

### A10: CLI-Parameter ueberschreiben Settings

**Trigger:** Benutzer gibt `--threads`, `--retries`, etc. an (Schritt 1)
**Flow:**

1. CLI-Parameter ueberschreiben die konfigurierten Werte fuer diese Ausfuehrung (-> CLI-CC, BR-CLI-006).
2. Werte werden nicht in die Datei zurueckgeschrieben.

## Postconditions

### Success Postconditions

- Alle selektierten Dateien sind heruntergeladen.
- Optionale Post-Processing-Schritte sind abgeschlossen.
- Ergebnis wurde auf stdout ausgegeben.

### Failure Postconditions

- Fehlgeschlagene Dateien bleiben als partielle Downloads auf Disk (fuer Resume).
- Ergebnis mit Fehlerliste auf stdout.

## Business Rules

### BR-CLI-012: Fortschrittsanzeige

- Fortschritt auf stderr mit Progress-Bars (wenn Terminal).
- `--quiet`: Keine Fortschrittsanzeige.
- `--json`: JSON-Lines auf stderr (-> CLI-CC).

### BR-CLI-013: Signal-Handling

- SIGINT einmal: Graceful Shutdown.
- SIGINT zweimal (< 2s): Sofortiger Abbruch.
- SIGTERM: Wie einmaliges SIGINT.

Weitere Regeln: -> CLI-CC (Cross-Cutting): Parameter-Prioritaet (BR-CLI-006), Exit-Codes (BR-CLI-007), Quiet-Modus (BR-CLI-008).

## Fortschrittsanzeige (stderr)

Zeigt pro aktiver Datei: Zaehler, Dateiname, Fortschrittsbalken, Prozent, Geschwindigkeit, ETA. Darunter eine Gesamtzeile mit aggregiertem Fortschritt, Durchschnittsgeschwindigkeit und Gesamt-ETA. Updates per `\r` wenn Terminal, sonst eine Zeile pro Datei-Abschluss.

## Ergebnis-Ausgabe (stdout)

Zusammenfassung mit Anzahl OK/fehlgeschlagen/uebersprungen, Dauer, Gesamtgroesse, Durchschnittsgeschwindigkeit. Bei Fehlern: Liste der fehlgeschlagenen Dateien mit Fehlertyp und Retry-Anzahl.

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Alle Dateien erfolgreich                 |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungueltiges SFDL-Format                  |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
| 5    | FTP-Fehler bei BulkFolder-Aufloesung     |
| 6    | Nicht genug Speicherplatz (strict mode)  |
| 10   | Teilweise fehlgeschlagen                 |
| 11   | Alle Downloads fehlgeschlagen            |
| 12   | Abbruch durch Signal (SIGINT/SIGTERM)    |
