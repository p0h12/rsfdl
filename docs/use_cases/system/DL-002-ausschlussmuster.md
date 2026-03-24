# Use Case: Ausschlussmuster anwenden

## Overview

**Use Case ID:** DL-002
**Use Case Name:** Ausschlussmuster anwenden
**Primary Actor:** System (automatisch)
**Goal:** FileEntries anhand konfigurierter Glob-Muster als ausgeschlossen markieren.
**Requirements:** FR-17
**Status:** Stable

## Preconditions

- Eine Liste von FileEntries liegt vor.
- Einstellungen mit Ausschlussmustern sind geladen (CFG-001).

## Main Success Scenario

1. System laedt die Ausschlussmuster aus den Einstellungen (CFG-001).
2. Fuer jeden FileEntry prueft das System den Dateinamen gegen alle Muster:
    - System fuehrt einen case-insensitiven Glob-Match des Dateinamens gegen jedes Muster durch.
    - Wenn mindestens ein Muster passt: FileEntry wird als `excluded=true` markiert.
3. System gibt die markierte Liste zurueck.

## Alternative Flows

### A1: Keine Muster konfiguriert

**Trigger:** Ausschlussmuster-Liste ist leer (Schritt 1)
**Flow:**

1. Ausschlussmuster-Liste ist leer.
2. Alle FileEntries bleiben `excluded=false`.
3. Use Case endet.

## Postconditions

### Success Postconditions

- FileEntries sind mit `excluded=true/false` markiert.

### Failure Postconditions

- Keine — der Use Case kann nicht fehlschlagen.

## Business Rules

### BR-DL-003: Glob-Syntax

- Unterstuetzte Wildcards: `*` (beliebig viele Zeichen), `?` (ein Zeichen)
- Matching ist case-insensitiv
- Muster werden nur auf den Dateinamen angewendet, nicht auf den Pfad

### BR-DL-004: Standard-Blacklist

- Default-Muster (bei Erstinstallation): `*.nfo`, `*.jpg`, `*.png`, `*.txt`, `*sample*`
- Da Matching case-insensitiv ist, deckt `*sample*` auch `*Sample*` ab.
- Actor kann Muster hinzufuegen und entfernen (CFG-001).

### BR-DL-005: CLI-Ueberschreibung

- CLI-Parameter `--exclude <pattern>` fuegt Muster zusaetzlich zu den gespeicherten Mustern hinzu
- CLI-Parameter `--no-exclude` deaktiviert alle Ausschlussmuster (auch gespeicherte)

## Input

- `file_entries[]`: Liste von FileEntries
- `patterns[]`: Ausschlussmuster aus Einstellungen + CLI

## Output

- `file_entries[]`: Gleiche Liste mit aktualisiertem `excluded`-Flag
- `excluded_count: int`: Anzahl ausgeschlossener Dateien
