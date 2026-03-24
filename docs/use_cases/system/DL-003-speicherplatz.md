# Use Case: Speicherplatz pruefen

## Overview

**Use Case ID:** DL-003
**Use Case Name:** Speicherplatz pruefen
**Primary Actor:** System (automatisch vor Download-Start)
**Goal:** Sicherstellen, dass genuegend Speicherplatz fuer den Download vorhanden ist.
**Requirements:** FR-19
**Status:** Stable

## Preconditions

- Eine Selektion mit Dateien und Groessen liegt vor.
- Ein Zielverzeichnis ist konfiguriert.

## Main Success Scenario

1. System ermittelt den verfuegbaren Speicherplatz im Zielverzeichnis.
2. System berechnet den benoetigten Speicherplatz:
    - Fuer neue Dateien: volle Dateigroesse
    - Fuer teilweise vorhandene Dateien (Resume): nur die Restgroesse
3. System vergleicht verfuegbaren mit benoetigtem Speicherplatz.
4. Verfuegbarer Platz >= benoetigter Platz — Use Case endet erfolgreich.

## Alternative Flows

### A1: Speicherplatz unzureichend

**Trigger:** Verfuegbarer Platz < benoetigter Platz (Schritt 4)
**Flow:**

1. System meldet: „Nicht genuegend Speicherplatz. Benoetigt: X MB, Verfuegbar: Y MB."
2. Actor kann den Download trotzdem starten (bestaetigen).
3. Actor kann den Download abbrechen.

### A2: Speicherplatz unzureichend und strict mode

**Trigger:** Verfuegbarer Platz < benoetigter Platz und `strict_disk_check=true` (Schritt 4)
**Flow:**

1. `strict_disk_check=true` (CLI: `--strict-disk-check`)
2. System meldet Fehler und bricht ab. Kein Bestaetigen moeglich.
3. Use Case endet mit Fehler.

### A3: Groessen teilweise unbekannt

**Trigger:** Nicht alle Dateien haben eine bekannte Groesse (Schritt 2)
**Flow:**

1. Nicht alle Dateien haben eine bekannte Groesse.
2. System berechnet mit bekannten Groessen und warnt: „Pruefung basiert auf X von Y Dateien. Tatsaechlicher Bedarf kann hoeher sein."

## Postconditions

### Success Postconditions

- Genuegend Speicherplatz vorhanden, Download kann starten.

### Failure Postconditions

- Speicherplatz knapp, Actor wurde informiert.

## Business Rules

### BR-DL-006: Speicherplatz-Berechnung

- Benoetigter Platz = Sigma(dateigroesse - bereits_heruntergeladen) fuer alle selektierten Dateien
- Ein Sicherheitspuffer von 1% wird addiert (mindestens 10 MB)

## Input

- `selection`: Aktive Selektion mit Dateigroessen
- `target_directory`: Zielverzeichnis
- `strict: bool`: Ob bei Unterschreitung abgebrochen wird

## Output

- `sufficient: bool`
- `available_bytes: int`
- `required_bytes: int`
