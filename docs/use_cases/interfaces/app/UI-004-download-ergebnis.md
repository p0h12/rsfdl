# Use Case: Download-Ergebnis

## Overview

**Use Case ID:** UI-004
**Use Case Name:** Download-Ergebnis
**Primary Actor:** Benutzer
**Goal:** Zusammenfassung einer abgeschlossenen Download-Session einsehen.
**Implements:** POST-001, POST-002, POST-003
**Interface:** GUI (Dioxus Desktop)
**Status:** Stable

## Preconditions

- Ein Download ist abgeschlossen (`download_phase` = `Done`).
- `DownloadSummary` ist im AppState verfügbar.

## Main Success Scenario

1. System zeigt ein farbkodiertes Summary-Banner:
    - Grün: Alle Dateien erfolgreich.
    - Gelb: Abbrüche vorhanden.
    - Rot: Fehler vorhanden.
2. System zeigt Statistiken: Total, Completed, Skipped, Failed, Cancelled.
3. Benutzer sieht die detaillierte Dateiliste im Progress-Panel (-> UI-003) mit Endstatus pro Datei.

## Alternative Flows

### A1: Reset nach Ergebnis

**Trigger:** Benutzer klickt „Reset" (nach Schritt 3)
**Flow:**

1. System setzt Download-Zustand zurück (Phase → Idle, File States leer, Summary leer).
2. Progress-Panel und Summary-Banner verschwinden.
3. Dateiliste wird wieder mit Checkboxen angezeigt.

### A2: Neuen Container öffnen

**Trigger:** Benutzer klickt „Open File" (nach Schritt 3)
**Flow:**

1. System öffnet Dateidialog für neuen Container.
2. Bisheriger Container und Download-Zustand werden ersetzt.
3. Use Case wechselt zu UI-001.

## Postconditions

### Success Postconditions

- Benutzer hat die Download-Zusammenfassung eingesehen.
- Bei A1: App ist bereit für einen neuen Download mit demselben Container.

### Failure Postconditions

- Keine (reine Anzeige-View).

## Business Rules

### BR-UI-004-001: Banner-Farbkodierung

| Bedingung                      | Farbe | CSS-Klasse                      |
|--------------------------------|-------|---------------------------------|
| `failed > 0`                   | Rot   | `bg-red-100 text-red-800`       |
| `cancelled > 0` (kein Failure) | Gelb  | `bg-yellow-100 text-yellow-800` |
| Sonst                          | Grün  | `bg-green-100 text-green-800`   |

### BR-UI-004-002: Statistik-Format

Anzeige: „Done: {total} total, {completed} completed, {skipped} skipped, {failed} failed, {cancelled} cancelled"

## Layout

### Summary-Banner

- Farbkodiertes Banner mit Zusammenfassungstext.
- Sichtbar nur wenn `download_phase` = `Done` und `DownloadSummary` vorhanden.

### Detaillierte Dateiliste

- Pro-Datei-Status aus dem Progress-Panel (-> UI-003) bleibt sichtbar.

### Aktionen

- „Reset" -> Download-Zustand zurücksetzen, Dateiliste wiederherstellen.
- „Open File" -> Neuen Container öffnen.

## Hinweise

- Hash-Verifikation (POST-001), Auto-Extraktion (POST-002) und Speed-Report (POST-003) sind in den Specs vorgesehen, aber in der GUI noch nicht vollständig integriert. Extraction-Events werden empfangen aber nicht dargestellt.
