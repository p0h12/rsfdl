# Use Case: Speicherplatz prüfen

## Overview

**Use Case ID:** DL-003
**Use Case Name:** Speicherplatz prüfen
**Primary Actor:** System (automatisch vor Download-Start)
**Goal:** Sicherstellen, dass genügend Speicherplatz für den Download vorhanden ist.
**Requirements:** FR-19
**Status:** Tested

## Preconditions

- Eine Selektion mit Dateien und Grössen liegt vor.
- Ein Zielverzeichnis ist konfiguriert.

## Main Success Scenario

1. System ermittelt den verfügbaren Speicherplatz im Zielverzeichnis.
2. System berechnet den benötigten Speicherplatz:
    - Für neue Dateien: volle Dateigrösse
    - Für teilweise vorhandene Dateien (Resume): nur die Restgrösse
3. System vergleicht verfügbaren mit benötigtem Speicherplatz.
4. Verfügbarer Platz >= benötigter Platz — Use Case endet erfolgreich.

## Alternative Flows

### A1: Speicherplatz unzureichend

**Trigger:** Verfügbarer Platz < benötigter Platz (Schritt 4)
**Flow:**

1. System meldet: "Nicht genügend Speicherplatz. Benötigt: X MB, Verfügbar: Y MB."
2. Actor kann den Download trotzdem starten (bestätigen).
3. Actor kann den Download abbrechen.

### A2: Speicherplatz unzureichend und strict mode

**Trigger:** Verfügbarer Platz < benötigter Platz und `strict_disk_check=true` (Schritt 4)
**Flow:**

1. `strict_disk_check=true` (CLI: `--strict-disk-check`)
2. System meldet Fehler und bricht ab. Kein Bestätigen möglich.
3. Use Case endet mit Fehler.

### A3: Grössen teilweise unbekannt

**Trigger:** Nicht alle Dateien haben eine bekannte Grösse (Schritt 2)
**Flow:**

1. Nicht alle Dateien haben eine bekannte Grösse.
2. System berechnet mit bekannten Grössen und warnt: "Prüfung basiert auf X von Y Dateien. Tatsächlicher Bedarf kann höher sein."

## Postconditions

### Success Postconditions

- Genügend Speicherplatz vorhanden, Download kann starten.

### Failure Postconditions

- Speicherplatz knapp, Actor wurde informiert.

## Business Rules

### BR-DL-006: Speicherplatz-Berechnung

- Benötigter Platz = Sigma(dateigrösse - bereits_heruntergeladen) für alle selektierten Dateien
- Ein Sicherheitspuffer von 1% wird addiert (mindestens 10 MB)

## Input

- `selection`: Aktive Selektion mit Dateigrössen
- `target_directory`: Zielverzeichnis
- `strict: bool`: Ob bei Unterschreitung abgebrochen wird

## Output

- `sufficient: bool`
- `available_bytes: int`
- `required_bytes: int`
