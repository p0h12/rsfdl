# Use Case: rsfdl info

## Overview

**Use Case ID:** CLI-001
**Use Case Name:** rsfdl info
**Primary Actor:** Benutzer
**Goal:** Metadaten eines SFDL-Containers anzeigen, ohne einen Download zu starten.
**Implements:** SFDL-001, SFDL-002, SFDL-003
**Status:** Stable

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- Eine SFDL-Datei existiert im Dateisystem.

## Syntax

`rsfdl info <datei.sfdl> [--password <pw>] [--json]`

## Parameter

| Parameter         | Pflicht | Standard | Beschreibung                                 |
|-------------------|---------|----------|----------------------------------------------|
| `<datei.sfdl>`    | ja      | —        | Pfad zur SFDL-Datei                          |
| `--password <pw>` | nein    | CFG      | Passwort fuer verschluesselte Container      |
| `--json`          | nein    | false    | Ausgabe als JSON statt menschenlesbarem Text |

## Main Success Scenario

1. Benutzer ruft `rsfdl info <datei.sfdl>` auf.
2. System oeffnet und parst die SFDL-Datei (-> SFDL-001).
3. Falls verschluesselt: System entschluesselt den Container (-> SFDL-002).
4. System gibt Container-Metadaten auf stdout aus.

## Alternative Flows

### A1: Passwort erforderlich (nicht-interaktiv)

**Trigger:** Container ist verschluesselt, kein Passwort passt, kein interaktives Terminal (Schritt 4)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 3.

### A2: Falsches Passwort

**Trigger:** `--password` angegeben, aber falsches Passwort (Schritt 3)
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

**Trigger:** Container ist verschluesselt, kein Passwort passt, stderr ist ein Terminal (Schritt 4)
**Flow:**

1. System zeigt Passwort-Prompt auf stdin.
2. Benutzer gibt Passwort ein.
3. System entschluesselt den Container.
4. Use Case faehrt mit Schritt 5 fort.

## Postconditions

### Success Postconditions

- Container-Metadaten wurden auf stdout ausgegeben.

### Failure Postconditions

- Fehlermeldung auf stderr. Kein Output auf stdout.

## Business Rules

### BR-CLI-009: Ausgabeformat

Standard-Ausgabe als key-value-Text. Mit `--json`: JSON-Objekt auf stdout.

Weitere Regeln: -> CLI-CC (Cross-Cutting): Kanaltrennung (BR-CLI-001), Parameter-Prioritaet (BR-CLI-006), Exit-Codes (BR-CLI-007).

## Ausgabe (Standard)

Key-value-Paare auf stdout, eines pro Zeile:

| Feld       | Beschreibung                                                  |
|------------|---------------------------------------------------------------|
| Container  | Beschreibung / Release-Name                                   |
| Uploader   | Uploader-Name                                                 |
| Host       | Hostname:Port (Protokoll)                                     |
| Version    | v2 oder v3                                                    |
| Encrypted  | no / yes (auto-decrypted) / yes (not decrypted)               |
| Packages   | Anzahl Pakete                                                 |
| Files      | Anzahl Dateien                                                |
| Size       | Gesamtgroesse (menschenlesbar)                                |

## Ausgabe (JSON)

JSON-Objekt auf stdout mit folgenden Feldern:

| Feld        | Typ     | Beschreibung                       |
|-------------|---------|------------------------------------|
| description | string  | Beschreibung / Release-Name        |
| uploader    | string  | Uploader-Name                      |
| host        | string  | FTP-Hostname                       |
| port        | number  | FTP-Port                           |
| protocol    | string  | Protokoll (z.B. "FTP")             |
| encrypted   | boolean | Ob der Container verschluesselt war |
| packages    | number  | Anzahl Pakete                      |
| total_files | number  | Anzahl Dateien                     |
| total_bytes | number  | Gesamtgroesse in Bytes             |

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Erfolg                                   |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungueltiges SFDL-Format                  |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
