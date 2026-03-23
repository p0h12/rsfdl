# DL-002: Ausschlussmuster anwenden

**Use Case ID:** DL-002
**Requirements:** FR-17
**Primary Actor:** System (automatisch)
**Trigger:** Wird als `<<include>>` von SFDL-003 und DL-001 aufgerufen.
**Preconditions:** Eine Liste von FileEntries liegt vor. Einstellungen mit Ausschlussmustern sind geladen.
**Postconditions:** FileEntries sind mit `excluded=true/false` markiert.

---

## Main Success Scenario

1. System lädt die Ausschlussmuster aus den Einstellungen (CFG-001).
2. Für jeden FileEntry prüft das System den Dateinamen gegen alle Muster:
   2a. System führt einen case-insensitiven Glob-Match des Dateinamens gegen jedes Muster durch.
   2b. Wenn mindestens ein Muster passt: FileEntry wird als `excluded=true` markiert.
3. System gibt die markierte Liste zurück.

## Alternative Paths

**1a. Keine Muster konfiguriert:**
1a.1. Ausschlussmuster-Liste ist leer.
1a.2. Alle FileEntries bleiben `excluded=false`.
1a.3. Use Case endet.

## Business Rules

**BR-DL-003: Glob-Syntax**

- Unterstützte Wildcards: `*` (beliebig viele Zeichen), `?` (ein Zeichen)
- Matching ist case-insensitiv
- Muster werden nur auf den Dateinamen angewendet, nicht auf den Pfad

**BR-DL-004: Standard-Blacklist**

- Default-Muster (bei Erstinstallation): `*.nfo`, `*.jpg`, `*.png`, `*.txt`, `*sample*`, `*Sample*`
- Actor kann Muster hinzufügen und entfernen (CFG-001)
- Standard-Muster sind als `is_default=true` markiert, können aber deaktiviert werden

**BR-DL-005: CLI-Überschreibung**

- CLI-Parameter `--exclude <pattern>` fügt Muster zusätzlich zu den gespeicherten Mustern hinzu
- CLI-Parameter `--no-exclude` deaktiviert alle Ausschlussmuster (auch gespeicherte)

## Input

- `file_entries[]`: Liste von FileEntries
- `patterns[]`: Ausschlussmuster aus Einstellungen + CLI

## Output

- `file_entries[]`: Gleiche Liste mit aktualisiertem `excluded`-Flag
- `excluded_count: int`: Anzahl ausgeschlossener Dateien
