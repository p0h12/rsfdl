# Use Case: Speed-Report generieren

## Overview

**Use Case ID:** POST-003
**Use Case Name:** Speed-Report generieren
**Primary Actor:** Benutzer
**Goal:** Nach Abschluss einer Download-Session einen formatierten Report generieren.
**Requirements:** FR-18
**Status:** Tested

## Preconditions

- Eine DownloadSession ist abgeschlossen (Status `Completed` oder `Failed`).

## Main Success Scenario

1. Download-Session ist abgeschlossen.
2. System sammelt die Statistiken der Session:
    - Container-Name und Uploader
    - Gesamtgrösse (heruntergeladene Bytes)
    - Gesamtdauer (Start bis Ende)
    - Durchschnittsgeschwindigkeit
    - Anzahl Dateien (total / erfolgreich / fehlgeschlagen / übersprungen)
    - Max. Threads
3. System lädt das Template aus den Einstellungen (`speedreport_template`) (→ BR-POST-006).
4. System ersetzt die Template-Variablen durch die berechneten Werte.
5. System schreibt den Report als `speedreport.txt` in den Paket-Unterordner im Download-Verzeichnis.

## Alternative Flows

### A1: Leeres Template

**Trigger:** `speedreport_template` ist leer (Schritt 3)
**Flow:**

1. System verwendet das Standard-Template (→ BR-POST-006).

### A2: Speed-Report deaktiviert

**Trigger:** CLI-Flag `--no-speedreport` gesetzt
**Flow:**

1. Kein Report wird generiert.

## Postconditions

### Success Postconditions

- `speedreport.txt` liegt im Paket-Unterordner (z.B. `download_dir/Container.Name/speedreport.txt`).

### Failure Postconditions

- Schreibfehler wird als Warnung gemeldet. Download-Erfolg wird nicht beeinflusst.

## Business Rules

### BR-POST-006: Template-Variablen

Standard-Template:

```
rsfdl v{{version}} speed report

SFDL: {{container_name}}
Uploader: {{uploader}}
{{total_size_formatted}} in {{duration}} heruntergeladen - ⌀Speed: {{avg_speed_formatted}}
{{total_files}} Dateien ({{completed_files}}✓ {{failed_files}}✗ {{skipped_files}}⊘)

Besten Dank!
```

Verfügbare Variablen:

- `{{version}}` — Programm-Version (compile-time)
- `{{uploader}}` — Uploader aus dem Container
- `{{container_name}}` — Beschreibung aus dem Container
- `{{total_files}}`, `{{completed_files}}`, `{{failed_files}}`, `{{skipped_files}}`
- `{{total_size_formatted}}` — auto-formatiert (z.B. "5.0 GiB", "286.1 MiB")
- `{{total_size_mb}}`, `{{total_size_gb}}` — numerisch mit 2 Dezimalstellen
- `{{duration}}` — formatiert als `HH:MM:SS`
- `{{avg_speed_formatted}}` — auto-formatiert (z.B. "11.68 MiB/s", "512.00 KiB/s")
- `{{avg_speed_mbps}}`, `{{avg_speed_kbps}}` — numerisch mit 2 Dezimalstellen
- `{{max_threads}}`

Das Standard-Template wird als Default in `settings.speedreport_template` gesetzt und ist in der `settings.toml` sichtbar und editierbar.

## Input

- `stats: SessionStats` — Statistiken der abgeschlossenen Session
- `template: String` — Template aus Einstellungen (`speedreport_template`)

## Output

- `speedreport.txt` im Paket-Unterordner
- `report: String` — gerenderter Report-Text (API)
