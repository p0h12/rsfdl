# Use Case: CLI config

## Overview

**Use Case ID:** CLI-005
**Use Case Name:** CLI config
**Primary Actor:** Benutzer
**Goal:** Einstellungen über die Kommandozeile anzeigen, bearbeiten oder den Dateipfad ermitteln.
**Implementiert:** CFG-001
**Status:** Tested

## Preconditions

- Der Benutzer hat Zugriff auf die Kommandozeile.

## Main Success Scenario

### `rsfdl config show`

1. Benutzer ruft `rsfdl config show` auf.
2. System ermittelt den Konfigurationspfad (-> BR-CLI-014).
3. System lädt die Einstellungen (-> CFG-001, Variante A).
4. System gibt Warnings (korrupte Datei, korrigierte Werte) auf stderr aus.
5. System gibt die Einstellungen im key=value-Format auf stdout aus.
6. Passwörter werden maskiert angezeigt (nur Anzahl).

### `rsfdl config edit`

1. Benutzer ruft `rsfdl config edit` auf.
2. System ermittelt den Konfigurationspfad (-> BR-CLI-014).
3. System erstellt die Konfigurationsdatei mit Standardwerten, falls nicht vorhanden (-> CFG-001, A1).
4. System ermittelt den Editor (-> BR-CLI-015).
5. System öffnet die Datei im Editor.
6. Benutzer bearbeitet die Datei und schliesst den Editor.
7. System validiert die bearbeitete Datei (-> CFG-001, Variante A).
8. System gibt Warnings auf stderr aus.

### `rsfdl config path`

1. Benutzer ruft `rsfdl config path` auf.
2. System ermittelt den Konfigurationspfad (-> BR-CLI-014).
3. System gibt den Pfad auf stdout aus.

## Alternative Flows

### A1: Editor-Prozess schlägt fehl

**Trigger:** Editor beendet sich mit Exit-Code != 0 (config edit, Schritt 6)
**Flow:**

1. System meldet: "Editor '{name}' exited with {code}."
2. Use Case endet mit Exit-Code 1.

### A2: Datei nach Bearbeitung ungültig

**Trigger:** Validierung nach Editor-Schliessung findet Fehler (config edit, Schritt 7)
**Flow:**

1. System gibt Korrektur-Warnings auf stderr aus (-> CFG-001, A3).
2. Use Case endet mit Exit-Code 0 (Warnings sind informativ, keine harten Fehler).

### A3: Konfigurationsdatei nicht schreibbar

**Trigger:** Dateisystem-Fehler beim Erstellen der Default-Datei (config edit, Schritt 3)
**Flow:**

1. System meldet den IO-Fehler auf stderr.
2. Use Case endet mit Exit-Code 1.

## Postconditions

### Success Postconditions

- **config show:** Einstellungen wurden auf stdout ausgegeben.
- **config edit:** Konfigurationsdatei existiert auf Disk (ggf. neu erstellt oder bearbeitet).
- **config path:** Pfad wurde auf stdout ausgegeben.

### Failure Postconditions

- Fehlermeldung auf stderr. Konfigurationsdatei bleibt unverändert.

## Business Rules

### BR-CLI-014: Pfadermittlung

- Konfigurationspfad wird via CFG-002 ermittelt.
- `config path` gibt diesen Pfad auf stdout aus.

### BR-CLI-015: Editor-Ermittlung

- System verwendet `$EDITOR` Umgebungsvariable.
- Fallback: `notepad` (Windows), `vi` (Unix).

### BR-CLI-016: Ausgabeformat

- `config show` gibt key=value-Format auf stdout aus.
- Passwörter werden maskiert: nur `(N entries)` angezeigt (-> CFG-001, BR-CFG-003).
- Warnings und Fehler gehen auf stderr.

Weitere Regeln: -> CLI-CC (Cross-Cutting): Kanaltrennung (BR-CLI-001), Exit-Codes (BR-CLI-007).

## Syntax

| Subcommand    | Optionen | Beschreibung                          |
|---------------|----------|---------------------------------------|
| `config show` | —        | Einstellungen anzeigen                |
| `config edit` | —        | Konfigurationsdatei im Editor öffnen  |
| `config path` | —        | Pfad zur Konfigurationsdatei ausgeben |

## Exit-Codes

| Code | Bedeutung                    |
|------|------------------------------|
| 0    | Erfolg                       |
| 1    | Editor-Fehler oder IO-Fehler |

## Input

- `subcommand`: show | edit | path
- Konfigurationspfad via CFG-002

## Output

- **show:** Formatierte Einstellungen auf stdout
- **edit:** (keine stdout-Ausgabe, Seiteneffekt: Datei bearbeitet)
- **path:** Pfad zur Konfigurationsdatei auf stdout
- **Warnings:** auf stderr
