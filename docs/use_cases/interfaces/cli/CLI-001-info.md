# Use Case: rsfdl info

## Overview

**Use Case ID:** CLI-001
**Use Case Name:** rsfdl info
**Primary Actor:** Benutzer
**Goal:** Metadaten eines SFDL-Containers anzeigen, ohne einen Download zu starten.
**Implements:** SFDL-001, SFDL-002, SFDL-003
**Interface:** CLI (headless)
**Status:** Stable

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.
- Eine SFDL-Datei existiert im Dateisystem.

## Syntax

`rsfdl info <datei.sfdl> [--password <pw>] [--json]`

## Parameter

| Parameter         | Pflicht | Beschreibung                                 |
|-------------------|---------|----------------------------------------------|
| `<datei.sfdl>`    | ja      | Pfad zur SFDL-Datei                          |
| `--password <pw>` | nein    | Passwort für verschlüsselte Container        |
| `--json`          | nein    | Ausgabe als JSON statt menschenlesbarem Text |

## Main Success Scenario

1. Benutzer ruft `rsfdl info <datei.sfdl>` auf.
2. System öffnet und parst die SFDL-Datei (-> SFDL-001).
3. Falls verschlüsselt und `--password` angegeben: System entschlüsselt mit dem Passwort (-> SFDL-002).
4. Falls verschlüsselt und kein `--password`: Auto-Passwort-Liste wird probiert (-> SFDL-002).
5. System gibt Container-Metadaten auf stdout aus.

## Alternative Flows

### A1: Passwort erforderlich (nicht-interaktiv)

**Trigger:** Container ist verschlüsselt, kein Passwort passt, kein interaktives Terminal (Schritt 4)
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

### A4: Ungültiges SFDL-Format

**Trigger:** Datei kann nicht geparst werden (Schritt 2)
**Flow:**

1. System gibt Fehlermeldung auf stderr aus.
2. Exit-Code 2.

### A5: Interaktiver Passwort-Prompt

**Trigger:** Container ist verschlüsselt, kein Passwort passt, stderr ist ein Terminal (Schritt 4)
**Flow:**

1. System zeigt Passwort-Prompt auf stdin.
2. Benutzer gibt Passwort ein.
3. System entschlüsselt den Container.
4. Use Case fährt mit Schritt 5 fort.

## Postconditions

### Success Postconditions

- Container-Metadaten wurden auf stdout ausgegeben.

### Failure Postconditions

- Fehlermeldung auf stderr. Kein Output auf stdout.

## Business Rules

### BR-CLI-001-001: Ausgabeformat

Standard-Ausgabe als key-value-Text. Mit `--json`: JSON-Objekt auf stdout.

## Ausgabe (Standard)

Key-value-Paare auf stdout, eines pro Zeile:

| Feld          | Beschreibung                                                |
|---------------|-------------------------------------------------------------|
| Container     | Beschreibung / Release-Name                                 |
| Uploader      | Uploader-Name                                               |
| Host          | Hostname:Port (Protokoll)                                   |
| Pakete        | Anzahl Pakete                                               |
| Dateien       | Anzahl Dateien                                              |
| Grösse        | Gesamtgrösse (menschenlesbar)                               |
| Verschlüsselt | ja/nein, ggf. mit Hinweis auf Auto-Passwort-Entschlüsselung |

## Ausgabe (JSON)

JSON-Objekt auf stdout mit folgenden Feldern:

| Feld        | Typ     | Beschreibung                       |
|-------------|---------|------------------------------------|
| description | string  | Beschreibung / Release-Name        |
| uploader    | string  | Uploader-Name                      |
| host        | string  | FTP-Hostname                       |
| port        | number  | FTP-Port                           |
| protocol    | string  | Protokoll (z.B. "FTP")             |
| encrypted   | boolean | Ob der Container verschlüsselt war |
| packages    | number  | Anzahl Pakete                      |
| total_files | number  | Anzahl Dateien                     |
| total_bytes | number  | Gesamtgrösse in Bytes              |

## Exit-Codes

| Code | Bedeutung                                |
|------|------------------------------------------|
| 0    | Erfolg                                   |
| 1    | Datei nicht gefunden / nicht lesbar      |
| 2    | Ungültiges SFDL-Format                   |
| 3    | Passwort erforderlich (nicht-interaktiv) |
| 4    | Falsches Passwort                        |
