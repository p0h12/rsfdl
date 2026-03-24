# Use Case: Download-Fortschritt

## Overview

**Use Case ID:** UI-003
**Use Case Name:** Download-Fortschritt
**Primary Actor:** Benutzer
**Goal:** Echtzeit-Fortschritt einer aktiven Download-Session verfolgen und einzelne/alle Downloads abbrechen.
**Implements:** DL-004, DL-005, DL-006, DL-007, DL-008
**Interface:** GUI (Dioxus Desktop)
**Status:** Stable

## Preconditions

- Ein Download wurde gestartet (UI-001 Schritt 12).
- `download_phase` ist `Downloading`.

## Main Success Scenario

1. System blendet das Progress-Panel unterhalb der Dateiliste ein.
2. System zeigt globalen Fortschritt: Fortschrittsbalken, Dateien-Zähler, Bytes-Zähler, Geschwindigkeit, ETA.
3. System zeigt Pro-Datei-Fortschritt als Liste, sortiert: aktive Downloads zuerst, dann abgeschlossene/fehlgeschlagene.
4. System empfängt ProgressEvents vom DownloadManager und aktualisiert die Anzeige (gedrosselt auf max. 10 Hz).
5. Alle Dateien sind heruntergeladen: System setzt `download_phase` auf `Done`.
6. System zeigt die Zusammenfassung (-> UI-004 via SummaryBanner).

## Alternative Flows

### A1: Einzelne Datei abbrechen

**Trigger:** Benutzer klickt „X" bei einer aktiven Datei (Schritt 3)
**Flow:**

1. System sendet Abbruch-Signal für diese Datei (-> DL-006 Variante A).
2. Datei-Status wechselt auf „Cancelled".
3. Verbleibende Downloads laufen weiter.

### A2: Alle Downloads abbrechen

**Trigger:** Benutzer klickt „Cancel" im Hauptfenster (Schritt 4)
**Flow:**

1. System sendet globales Abbruch-Signal über den CancellationToken (-> DL-006 Variante B).
2. Alle aktiven Downloads werden abgebrochen.
3. System setzt `download_phase` auf `Done`.
4. Zusammenfassung zeigt Cancelled-Status.

### A3: Download fehlgeschlagen

**Trigger:** Eine Datei schlägt nach Retries fehl (Schritt 4)
**Flow:**

1. Datei-Status wechselt auf „Failed" mit roter Markierung.
2. Fehlertext wird angezeigt.
3. Verbleibende Downloads laufen weiter.

### A4: Download wird fortgesetzt (Resume)

**Trigger:** Datei war teilweise heruntergeladen (Schritt 4)
**Flow:**

1. System erkennt die partielle Datei (-> DL-005).
2. Fortschrittsbalken beginnt ab dem bereits heruntergeladenen Offset.

### A5: Datei übersprungen

**Trigger:** Datei existiert bereits vollständig auf Disk (Schritt 4)
**Flow:**

1. Datei-Status ist sofort „Skipped".
2. Zähler `files_done` wird inkrementiert.

## Postconditions

### Success Postconditions

- Alle Dateien sind heruntergeladen oder haben einen Endstatus (Completed, Failed, Cancelled, Skipped).
- `download_phase` ist `Done`.
- Zusammenfassung ist verfügbar.

### Failure Postconditions

- Bei A2: Teils heruntergeladene Dateien bleiben auf Disk (für späteren Resume).
- Bei A3: Fehlgeschlagene Dateien werden in der Zusammenfassung aufgelistet.

## Business Rules

### BR-UI-003-001: UI-Update-Throttling

- ProgressEvents werden mit max. 100 ms Intervall verarbeitet (10 Hz).
- BytesWritten-Events werden gesammelt und in Batches angewendet.

### BR-UI-003-002: Sortierung

Datei-Einträge werden nach Status sortiert:

1. Downloading (aktiv)
2. Pending
3. Completed
4. Skipped
5. Failed
6. Cancelled

### BR-UI-003-003: Geschwindigkeit und ETA

- Geschwindigkeit: Gesamtbytes / verstrichene Zeit seit Start.
- ETA: Verbleibende Bytes / aktuelle Geschwindigkeit.
- Anzeige nur wenn Geschwindigkeit > 0.

### BR-UI-003-004: Panel-Sichtbarkeit

- Panel ist unsichtbar wenn `download_phase` = `Idle`.
- Panel bleibt sichtbar nach `Done` bis expliziter Reset.

## Layout

### Globaler Fortschritt

- Fortschrittsbalken: geschriebene Bytes / total Bytes
- Text: „X/Y files" + „Z MB / W MB" + Geschwindigkeit + ETA

### Pro-Datei-Fortschritt

- Dateiname (abgeschnitten bei Overflow)
- Status-Text (Bytes bei Downloading, „completed"/„failed"/etc. sonst)
- Fortschrittsbalken (nur bei Downloading)
- „X"-Button zum Abbrechen (nur bei Downloading)

## Zustandsübergänge

| Event                       | UI-Reaktion                                        |
|-----------------------------|----------------------------------------------------|
| ProgressEvent::Started      | Neuer Eintrag in der Dateiliste                    |
| ProgressEvent::BytesWritten | Fortschrittsbalken + Geschwindigkeit aktualisieren |
| ProgressEvent::Completed    | Status → Completed, Icon/Farbe wechseln            |
| ProgressEvent::Failed       | Status → Failed, Fehlertext anzeigen               |
| ProgressEvent::Cancelled    | Status → Cancelled                                 |
| ProgressEvent::Skipped      | Status → Skipped                                   |
| ProgressEvent::AllDone      | Phase → Done, Zusammenfassung anzeigen             |
