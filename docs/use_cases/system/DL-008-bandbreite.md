# Use Case: Bandbreite begrenzen

## Overview

**Use Case ID:** DL-008
**Use Case Name:** Bandbreite begrenzen
**Primary Actor:** Benutzer (Konfiguration), System (Durchsetzung)
**Goal:** Die Download-Geschwindigkeit auf ein konfiguriertes Limit begrenzen.
**Requirements:** FR-15
**Status:** Tested

## Preconditions

- `max_speed_kbps` ist in den Einstellungen oder als Parameter gesetzt und > 0.

## Main Success Scenario

1. System berechnet das Limit pro Thread: `max_bytes_per_thread = (max_speed_kbps * 1024) / aktive_threads`.
2. Nach jedem geschriebenen Block misst der Worker die aktuelle Geschwindigkeit.
3. Wenn die Geschwindigkeit das Pro-Thread-Limit überschreitet:
    - System berechnet die erforderliche Pause.
    - Worker wartet (sleep) die berechnete Zeit.
4. System passt das Pro-Thread-Limit dynamisch an, wenn sich die Anzahl aktiver Threads ändert (Task endet, neue Task startet).

## Alternative Flows

### A1: Nur 1 Thread aktiv

**Trigger:** Anzahl aktiver Threads sinkt auf 1 (Schritt 4)
**Flow:**

1. Der eine aktive Thread erhält das gesamte Bandbreitenlimit.

## Postconditions

### Success Postconditions

- Download-Geschwindigkeit überschreitet das konfigurierte Limit nicht.

### Failure Postconditions

- Keine — Throttling kann nicht fehlschlagen.

## Business Rules

### BR-DL-017: Throttling-Mechanismus

- Throttling geschieht im Read-Loop nach jedem Buffer-Write (32 KB Block).
- Berechnung: `sleep_time = bytes_written / limit_per_second - elapsed_time`
- Wenn `sleep_time <= 0`: kein Throttling nötig.

### BR-DL-018: Dynamische Anpassung

- Wenn ein Thread fertig wird, teilen sich die verbleibenden Threads das Gesamtlimit neu auf.
- Neuberechnung bei jedem Thread-Start/Ende, nicht bei jedem Block.

### BR-DL-019: Konfiguration

- `max_speed_kbps=0` bedeutet unbegrenzt (kein Throttling aktiv).
- CLI: `--max-speed <KB/s>`

## Input

- `max_speed_kbps: int` — globales Limit
- `active_threads: int` — aktülle Anzahl aktiver Worker

## Output

- Effektives Throttling pro Worker-Thread
