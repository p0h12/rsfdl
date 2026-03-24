# Use Case: Fehlgeschlagene Downloads wiederholen

## Overview

**Use Case ID:** DL-007
**Use Case Name:** Fehlgeschlagene Downloads wiederholen
**Primary Actor:** System (automatisch bei Fehler)
**Goal:** Fehlgeschlagene Downloads mit retry-faehigen Fehlern automatisch erneut versuchen.
**Requirements:** FR-10
**Status:** Stable

## Preconditions

- Task hat Status `Failed`.
- `retry_count < max_retries`.
- Fehlertyp ist retry-faehig (→ BR-DL-010).

## Main Success Scenario

1. System prueft den Fehlertyp der fehlgeschlagenen Task (→ BR-DL-010).
2. Fehlertyp ist retry-faehig.
3. System prueft: `retry_count < max_retries`.
4. System wartet `delay_seconds` (→ BR-DL-015).
5. System erhoeht `retry_count` um 1.
6. System setzt Task-Status zurueck auf `Pending`.
7. Task wird in die Worker-Queue eingereiht.
8. → DL-004 verarbeitet die Task erneut (inkl. Resume, falls teilweise vorhanden).

## Alternative Flows

### A1: Permanenter Fehler

**Trigger:** Fehlertyp ist nicht retry-faehig (Schritt 1)
**Flow:**

1. Fehlertyp ist nicht retry-faehig (FileNotFound, IOError, etc.).
2. Task bleibt `Failed` mit Fehlermeldung.
3. Kein Retry. Use Case endet.

### A2: Max Retries erreicht

**Trigger:** `retry_count >= max_retries` (Schritt 3)
**Flow:**

1. `retry_count >= max_retries`.
2. Task bleibt permanent `Failed`.
3. System meldet: „Download fehlgeschlagen nach X Versuchen: [letzter Fehler]."
4. Use Case endet.

## Postconditions

### Success Postconditions

- Task wird erneut als `Pending` eingereiht und beim naechsten freien Worker ausgefuehrt.

### Failure Postconditions

- Task bleibt `Failed`. Fehlermeldung enthaelt Retry-Historie.

## Business Rules

### BR-DL-015: Retry-Timing

- Wartezeit zwischen Retries: konfigurierbar (Standard: 10s)
- Bei `ServerFull (421)`: doppelte Wartezeit (Exponential Backoff bis max 120s)
- Bei `AuthError (530)`: einfache Wartezeit (Server koennte temporaer Verbindungen begrenzen)

### BR-DL-016: Retry-Konfiguration

- `max_retries`: Standard 3, konfigurierbar in Einstellungen
- `delay_seconds`: Standard 10, konfigurierbar in Einstellungen
- CLI: `--retries <n>`, `--retry-delay <seconds>`

## Input

- `task: DownloadTask` — fehlgeschlagene Task
- `retry_policy: RetryPolicy`

## Output

- Task zurueck in `Pending`-Queue oder permanent `Failed`
