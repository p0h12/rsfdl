# Use Case: Metadaten setzen

## Overview

**Use Case ID:** CR-006
**Use Case Name:** Metadaten setzen
**Primary Actor:** Benutzer
**Goal:** Optionale Metadaten (Beschreibung, Uploader, Max-Threads) in einem Container setzen.
**Requirements:** FR-25
**Status:** Stable

## Preconditions

- Ein Container-Objekt wird gerade erstellt.
- Wird von CR-001 aufgerufen, wenn Metadaten angegeben werden.

## Main Success Scenario

1. Actor gibt optionale Metadaten an:
    - Beschreibung (z.B. Release-Name)
    - Uploader-Name
    - Max-Download-Threads
2. System validiert die Werte (-> BR-CR-013).
3. System setzt die Metadaten im Container-Objekt.
4. System gibt den Container an CR-001 zurück.

## Alternative Flows

### A1: Keine Metadaten angegeben

**Trigger:** Actor gibt keine Metadaten an (Schritt 1)
**Flow:**

1. System verwendet die Standardwerte (-> BR-CR-013).
2. Use Case endet erfolgreich.

### A2: Ungültiger Wert für MaxDownloadThreads

**Trigger:** Actor gibt einen Wert ausserhalb des gültigen Bereichs an (Schritt 2)
**Flow:**

1. Actor gibt einen Wert < 1 oder > 10 an.
2. System meldet: "MaxDownloadThreads muss zwischen 1 und 10 liegen."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Container enthält die gesetzten Metadaten.

### Failure Postconditions

- Standardwerte werden verwendet.

## Business Rules

### BR-CR-013: Standardwerte

- `Description`: leer
- `Uploader`: "rsfdl"
- `MaxDownloadThreads`: 3

### BR-CR-014: Validierung

- `MaxDownloadThreads` muss ein positiver Integer sein (1-10).
- `Description` und `Uploader` sind Freitextfelder ohne Längenbeschränkung.

## Input

- `description`: Optional -- Beschreibung des Containers
- `uploader`: Optional -- Uploader-Name (Standard: "rsfdl")
- `max_threads`: Optional -- Maximale Download-Threads (Standard: 3)

## Output

- Container mit gesetzten Metadaten
