# DL-007: Fehlgeschlagene Downloads wiederholen

**Use Case ID:** DL-007
**Requirements:** FR-10
**Primary Actor:** System (automatisch bei Fehler)
**Trigger:** Eine DownloadTask wechselt zu Status `Failed` mit einem retry-fähigen Fehlertyp.
**Preconditions:** Task hat Status `Failed`. `retry_count < max_retries`. Fehlertyp ist retry-fähig (→ BR-DL-010).
**Postconditions (Erfolg):** Task wird erneut als `Pending` eingereiht und beim nächsten freien Worker ausgeführt.
**Postconditions (Max Retries):** Task bleibt `Failed`. Fehlermeldung enthält Retry-Historie.

---

## Main Success Scenario

1. System prüft den Fehlertyp der fehlgeschlagenen Task (→ BR-DL-010).
2. Fehlertyp ist retry-fähig.
3. System prüft: `retry_count < max_retries`.
4. System wartet `delay_seconds` (→ BR-DL-015).
5. System erhöht `retry_count` um 1.
6. System setzt Task-Status zurück auf `Pending`.
7. Task wird in die Worker-Queue eingereiht.
8. → DL-004 verarbeitet die Task erneut (inkl. Resume, falls teilweise vorhanden).

## Alternative Paths

**1a. Permanenter Fehler:**
1a.1. Fehlertyp ist nicht retry-fähig (FileNotFound, IOError, etc.).
1a.2. Task bleibt `Failed` mit Fehlermeldung.
1a.3. Kein Retry. Use Case endet.

**3a. Max Retries erreicht:**
3a.1. `retry_count >= max_retries`.
3a.2. Task bleibt permanent `Failed`.
3a.3. System meldet: „Download fehlgeschlagen nach X Versuchen: [letzter Fehler]."
3a.4. Use Case endet.

## Business Rules

**BR-DL-015: Retry-Timing**

- Wartezeit zwischen Retries: konfigurierbar (Standard: 10s)
- Bei `ServerFull (421)`: doppelte Wartezeit (Exponential Backoff bis max 120s)
- Bei `AuthError (530)`: einfache Wartezeit (Server könnte temporär Verbindungen begrenzen)

**BR-DL-016: Retry-Konfiguration**

- `max_retries`: Standard 3, konfigurierbar in Einstellungen
- `delay_seconds`: Standard 10, konfigurierbar in Einstellungen
- CLI: `--retries <n>`, `--retry-delay <seconds>`

## Input

- `task: DownloadTask` — fehlgeschlagene Task
- `retry_policy: RetryPolicy`

## Output

- Task zurück in `Pending`-Queue oder permanent `Failed`
