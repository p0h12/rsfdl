# Use Case: Archive extrahieren

## Overview

**Use Case ID:** POST-002
**Use Case Name:** Archive extrahieren
**Primary Actor:** System (automatisch nach Download)
**Goal:** Heruntergeladene Archive automatisch im Zielverzeichnis entpacken.
**Requirements:** FR-16
**Status:** Stable

## Preconditions

- Alle Dateien eines Pakets sind vollstaendig heruntergeladen.
- Auto-Extraktion ist aktiviert (`auto_extract=true` in den Einstellungen).

## Main Success Scenario

1. System scannt die heruntergeladenen Dateien des Pakets nach Archiven (→ BR-POST-003).
2. Fuer jedes erkannte Archiv erstellt das System eine ExtractionTask.
3. System extrahiert das Archiv in das Zielverzeichnis:
   - **[ZIP]** System entpackt mit Standard-ZIP-Bibliothek.
   - **[RAR]** System entpackt mit `unrar`-Bibliothek.
   - **[Multi-Part RAR]** System erkennt den ersten Teil und entpackt das gesamte Archiv.
4. System emittiert Fortschritts-Events waehrend der Extraktion.
5. ExtractionTask-Status → `Completed`.
6. **[delete_archives_after_extract=true]** System loescht die Archiv-Dateien.

## Alternative Flows

### A1: Keine Archive gefunden

**Trigger:** Keine Dateien passen auf Archiv-Muster (Schritt 1)
**Flow:**

1. Keine Dateien passen auf Archiv-Muster.
2. Use Case endet ohne Aktion.

### A2: Extraktion fehlgeschlagen

**Trigger:** Archiv ist beschaedigt oder passwortgeschuetzt (Schritt 3)
**Flow:**

1. Archiv ist beschaedigt oder passwortgeschuetzt.
2. ExtractionTask-Status → `Failed` mit Fehlermeldung.
3. Archiv-Dateien bleiben erhalten.
4. System meldet: „Extraktion fehlgeschlagen fuer [Archiv]: [Fehler]."
5. Der Download-Gesamterfolg wird nicht beeinflusst.

### A3: Multi-Part unvollstaendig

**Trigger:** Nicht alle Teile eines Multi-Part-RAR sind vorhanden (Schritt 3)
**Flow:**

1. Nicht alle Teile eines Multi-Part-RAR sind vorhanden.
2. System meldet: „Archiv unvollstaendig: Teil X von Y fehlt."
3. ExtractionTask-Status → `Failed`.

## Postconditions

### Success Postconditions

- Archive sind entpackt.
- Optional: Archiv-Dateien geloescht (wenn `delete_archives_after_extract=true`).

### Failure Postconditions

- Fehlermeldung vorhanden. Archiv-Dateien bleiben erhalten.
- Download-Erfolg wird nicht beeinflusst.

## Business Rules

### BR-POST-003: Archiv-Erkennung

- ZIP: Dateiendung `.zip`
- RAR: Dateiendung `.rar`
- Multi-Part RAR: `.part01.rar`, `.part001.rar`, `.r01`, `.r001`
- Bei Multi-Part: Nur der erste Teil loest die Extraktion aus.
- Erkennung ist case-insensitiv.

### BR-POST-004: Extraktions-Ziel

- Archive werden in das gleiche Verzeichnis entpackt, in dem sie liegen.
- Enthaelt das Archiv einen Root-Ordner, wird dieser beibehalten.
- Bei Namenskonflikten: bestehende Dateien werden nicht ueberschrieben.

### BR-POST-005: Feature-Toggle

- Auto-Extraktion ist in den Einstellungen deaktivierbar (Standard: aus).
- `delete_archives_after_extract` ist separat schaltbar (Standard: aus).

## Input

- `package_files[]`: Heruntergeladene Dateien eines Pakets
- `target_directory: String`
- `delete_after: bool`

## Output

- `ExtractionTask[]` mit Status pro Archiv
