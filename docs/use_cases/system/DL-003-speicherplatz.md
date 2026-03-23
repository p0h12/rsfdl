# DL-003: Speicherplatz prüfen

**Use Case ID:** DL-003
**Requirements:** FR-19
**Primary Actor:** System (automatisch vor Download-Start)
**Trigger:** Wird als `<<include>>` von DL-004 aufgerufen, bevor der erste Download-Task startet.
**Preconditions:** Eine Selektion mit Dateien und Grössen liegt vor. Ein Zielverzeichnis ist konfiguriert.
**Postconditions (Erfolg):** Genügend Speicherplatz vorhanden, Download kann starten.
**Postconditions (Warnung):** Speicherplatz knapp, Actor wurde informiert.

---

## Main Success Scenario

1. System ermittelt den verfügbaren Speicherplatz im Zielverzeichnis.
2. System berechnet den benötigten Speicherplatz:
    - Für neue Dateien: volle Dateigrösse
    - Für teilweise vorhandene Dateien (Resume): nur die Restgrösse
3. System vergleicht verfügbaren mit benötigtem Speicherplatz.
4. Verfügbarer Platz >= benötigter Platz → Use Case endet erfolgreich.

## Alternative Paths

**4a. Speicherplatz unzureichend:**
4a.1. System meldet: „Nicht genügend Speicherplatz. Benötigt: X MB, Verfügbar: Y MB."
4a.2. Actor kann den Download trotzdem starten (bestätigen).
4a.3. Actor kann den Download abbrechen.

**4b. Speicherplatz unzureichend + strict mode:**
4b.1. `strict_disk_check=true` (CLI: `--strict-disk-check`)
4b.2. System meldet Fehler und bricht ab. Kein Bestätigen möglich.
4b.3. Use Case endet mit Fehler.

**2a. Grössen teilweise unbekannt:**
2a.1. Nicht alle Dateien haben eine bekannte Grösse.
2a.2. System berechnet mit bekannten Grössen und warnt:
„Prüfung basiert auf X von Y Dateien. Tatsächlicher Bedarf kann höher sein."

## Business Rules

**BR-DL-006: Speicherplatz-Berechnung**

- Benötigter Platz = Σ(dateigrösse - bereits_heruntergeladen) für alle selektierten Dateien
- Ein Sicherheitspuffer von 1% wird addiert (mindestens 10 MB)

## Input

- `selection`: Aktive Selektion mit Dateigrössen
- `target_directory`: Zielverzeichnis
- `strict: bool`: Ob bei Unterschreitung abgebrochen wird

## Output

- `sufficient: bool`
- `available_bytes: int`
- `required_bytes: int`
