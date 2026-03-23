# POST-003: Speed-Report generieren

**Use Case ID:** POST-003
**Requirements:** FR-18
**Primary Actor:** Benutzer
**Preconditions:** Eine DownloadSession ist abgeschlossen (Status `Completed` oder `Failed`).
**Postconditions:** Ein formatierter BBCode-Report liegt als String vor.

---

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

## Alternative Paths

**3a. Kein Template konfiguriert:**
3a.1. System verwendet das Standard-Template (→ BR-POST-006).

## Business Rules

**BR-POST-006: Template-Variablen**

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
