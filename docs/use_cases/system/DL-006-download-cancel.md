# Use Case: Download abbrechen

## Overview

**Use Case ID:** DL-006
**Use Case Name:** Download abbrechen
**Primary Actor:** Benutzer
**Goal:** Einen laufenden Download (einzelne Datei oder alle) sauber abbrechen und teilweise heruntergeladene Dateien erhalten.
**Requirements:** FR-07
**Status:** Stable

## Preconditions

- Eine DownloadSession mit Status `Running` existiert.

## Main Success Scenario

### Variante A: Einzelne Datei abbrechen

1. Actor signalisiert Abbruch für eine spezifische DownloadTask.
2. System setzt ein Cancel-Flag für die betroffene Task.
3. Worker erkennt das Flag beim nächsten Block-Read.
4. Worker bricht den FTP-Transfer ab (`ABOR`).
5. Worker schliesst die FTP-Verbindung.
6. Task-Status → `Cancelled`.
7. Lokale Datei bleibt erhalten (für Resume).
8. Worker nimmt nächste `Pending`-Task.

### Variante B: Alle Downloads abbrechen

1. Actor signalisiert globalen Abbruch.
2. System setzt ein globales Cancel-Flag.
3. Alle Worker erkennen das Flag.
4. Jeder Worker bricht seinen aktuellen Transfer ab.
5. Alle `Pending`-Tasks → `Cancelled` (ohne gestartet zu werden).
6. Alle `Active`-Tasks → `Cancelled`.
7. DownloadSession-Status → `Cancelled`.

## Alternative Flows

### A1: ABOR fehlgeschlagen

**Trigger:** FTP-Server reagiert nicht auf `ABOR` (Schritt 4, Variante A)
**Flow:**

1. FTP-Server reagiert nicht auf `ABOR`.
2. System schliesst die TCP-Verbindung direkt (hard close).
3. Task wird trotzdem als `Cancelled` markiert.

## Postconditions

### Success Postconditions

- Betroffene Tasks haben Status `Cancelled`.
- Teilweise heruntergeladene Dateien bleiben erhalten.
- FTP-Verbindungen sind geschlossen.

### Failure Postconditions

- Keine — der Abbruch wird immer durchgeführt (ggf. via hard close).

## Business Rules

### BR-DL-013: Cancel-Granularität

- Abbruch ist pro Datei und global möglich.
- `Completed`-Tasks werden durch globalen Abbruch nicht rückgängig gemacht.
- `Failed`-Tasks behalten ihren Status.

### BR-DL-014: Dateierhaltung

- Teilweise heruntergeladene Dateien werden nicht gelöscht.
- Bei erneutem Download derselben Datei greift DL-005 (Resume).

## Input

- `task_id: Option<TaskId>` — spezifische Task oder None für global
- `session: DownloadSession`

## Output

- Aktualisierte DownloadSession mit `Cancelled`-Tasks
