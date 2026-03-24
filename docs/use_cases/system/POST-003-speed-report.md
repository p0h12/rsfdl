# Use Case: Speed-Report generieren

## Overview

**Use Case ID:** POST-003
**Use Case Name:** Speed-Report generieren
**Primary Actor:** Benutzer
**Goal:** Nach Abschluss einer Download-Session einen formatierten BBCode-Report generieren.
**Requirements:** FR-18
**Status:** Tested

## Preconditions

- Eine DownloadSession ist abgeschlossen (Status `Completed` oder `Failed`).

## Main Success Scenario

1. Actor fordert einen Speed-Report an.
2. System sammelt die Statistiken der DownloadSession:
    - Gesamtgrösse (heruntergeladene Bytes)
    - Gesamtdauer (Start bis Ende)
    - Durchschnittsgeschwindigkeit
    - Anzahl Dateien (erfolgreich / fehlgeschlagen / übersprungen)
3. System lädt das BBCode-Template aus den Einstellungen (→ BR-POST-006).
4. System ersetzt die Template-Variablen durch die berechneten Werte.
5. System gibt den gerenderten Report-Text zurück.

## Alternative Flows

### A1: Kein Template konfiguriert

**Trigger:** Kein benutzerdefiniertes Template vorhanden (Schritt 3)
**Flow:**

1. System verwendet das Standard-Template (→ BR-POST-006).

## Postconditions

### Success Postconditions

- Ein formatierter BBCode-Report liegt als String vor.

### Failure Postconditions

- Keine — der Report kann immer generiert werden (Fallback auf Standard-Template).

## Business Rules

### BR-POST-006: Template-Variablen

Das Standard-Template enthält BBCode-formatierte Zeilen mit Tool-Name, Dateizähler (total, OK, fehlgeschlagen), Gesamtgrösse, Dauer und Durchschnittsgeschwindigkeit.

Verfügbare Variablen:

- `{{username}}` — aus Einstellungen
- `{{total_files}}`, `{{completed_files}}`, `{{failed_files}}`, `{{skipped_files}}`
- `{{total_size_mb}}`, `{{total_size_gb}}`
- `{{duration}}` — formatiert als `HH:MM:SS`
- `{{avg_speed_mbps}}`, `{{avg_speed_kbps}}`
- `{{max_threads}}`
- `{{container_name}}` — Beschreibung aus dem Container

## Input

- `session: DownloadSession` — abgeschlossene Session
- `template: String` — BBCode-Template

## Output

- `report: String` — gerenderter BBCode-Text
