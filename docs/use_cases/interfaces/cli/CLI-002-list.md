# Use Case: rsfdl list

## Overview

**Use Case ID:** CLI-002
**Use Case Name:** rsfdl list
**Primary Actor:** Benutzer
**Goal:** Alle Dateien eines SFDL-Containers auflisten, inklusive Ausschluss-Markierungen.
**Implements:** SFDL-001, SFDL-002, SFDL-003, DL-002
**Status:** Stable

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- Eine SFDL-Datei existiert im Dateisystem.

## Syntax

`rsfdl list <datei.sfdl> [--password <pw>] [--json] [--exclude <pattern>] [--no-exclude] [--show-excluded]`

## Parameter

| Parameter             | Pflicht | Standard | Beschreibung                                        |
|-----------------------|---------|----------|-----------------------------------------------------|
| `<datei.sfdl>`        | ja      | —        | Pfad zur SFDL-Datei                                 |
| `--password <pw>`     | nein    | CFG      | Passwort fuer verschluesselte Container             |
| `--json`              | nein    | false    | Ausgabe als JSON                                    |
| `--exclude <pattern>` | nein    | CFG      | Zusaetzliches Ausschlussmuster (mehrfach verwendbar) |
| `--no-exclude`        | nein    | false    | Alle Ausschlussmuster deaktivieren                  |
| `--show-excluded`     | nein    | false    | Ausgeschlossene Dateien mit anzeigen (markiert)     |

## Main Success Scenario

1. Benutzer ruft `rsfdl list <datei.sfdl>` auf.
2. System oeffnet und parst die SFDL-Datei (-> SFDL-001).
3. Falls verschluesselt: System entschluesselt den Container (-> SFDL-002, via --password, Auto-Passwort-Liste oder interaktiver Prompt).
4. System loest den Container-Inhalt auf (-> SFDL-003).
5. System wendet Ausschlussmuster an (-> DL-002).
6. System gibt die Dateiliste auf stdout aus.

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

### A6: BulkFolder-Aufloesung fehlgeschlagen

**Trigger:** FTP-Verbindung fuer BulkFolder schlaegt fehl (Schritt 3)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 5.

### A7: Ausschlussmuster ueberschreiben

**Trigger:** `--exclude` oder `--no-exclude` angegeben (Schritt 4)
**Flow:**

1. `--no-exclude`: Keine Muster werden angewendet.
2. `--exclude <pattern>`: Zusaetzliche Muster werden zu den konfigurierten hinzugefuegt.
3. Use Case faehrt mit Schritt 5 fort.

## Postconditions

### Success Postconditions

- Dateiliste wurde auf stdout ausgegeben.

### Failure Postconditions

- Fehlermeldung auf stderr. Kein Output auf stdout.

## Business Rules

### BR-CLI-010: Ausschluss-Anzeige

- Ohne `--show-excluded`: Ausgeschlossene Dateien werden nicht aufgelistet.
- Mit `--show-excluded`: Ausgeschlossene Dateien werden mit `[excluded]` markiert.

### BR-CLI-011: Zusammenfassung

Die letzte Zeile zeigt: "N Dateien (X GB), M ausgeschlossen".

Weitere Regeln: -> CLI-CC (Cross-Cutting): Kanaltrennung (BR-CLI-001), Parameter-Prioritaet (BR-CLI-006), Exit-Codes (BR-CLI-007).

## Ausgabe (Standard)

Gruppiert nach Paketen, pro Datei eine Zeile mit Dateiname und Groesse. Bei `--show-excluded` werden ausgeschlossene Dateien mit `[excluded]` markiert. Abschlusszeile mit Zusammenfassung (Dateien, Groesse, Ausgeschlossene).

## Ausgabe (JSON)

JSON-Objekt auf stdout:

| Feld                     | Typ     | Beschreibung                              |
|--------------------------|---------|-------------------------------------------|
| packages[].name          | string  | Paketname                                 |
| packages[].files[]       | array   | Datei-Eintraege                           |
| .files[].filename        | string  | Dateiname                                 |
| .files[].size_bytes      | number  | Dateigroesse in Bytes                     |
| .files[].excluded        | boolean | Ob durch Ausschlussmuster ausgeschlossen  |
| .files[].exclude_pattern | string  | Das passende Muster (nur wenn excluded)   |
| summary.total_files      | number  | Gesamtanzahl Dateien                      |
| summary.selected_files   | number  | Anzahl nicht-ausgeschlossene Dateien      |
| summary.excluded_files   | number  | Anzahl ausgeschlossene Dateien            |
| summary.total_bytes      | number  | Gesamtgroesse in Bytes                    |
| summary.selected_bytes   | number  | Groesse der nicht-ausgeschlossenen Dateien |

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Erfolg                                   |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungueltiges SFDL-Format                  |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
| 5    | FTP-Fehler bei BulkFolder-Aufloesung     |
