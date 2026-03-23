# DL-001: Dateien zum Download auswählen

**Use Case ID:** DL-001
**Requirements:** FR-04, FR-17
**Primary Actor:** Benutzer
**Preconditions:** Ein aufgelöster Container mit FileEntries liegt vor (SFDL-003 abgeschlossen).
**Postconditions (Erfolg):** Eine Selektion mit ausgewählten Dateien und berechneter Gesamtgrösse liegt vor.
**Postconditions (Fehlschlag):** —

---

## Main Success Scenario

1. System erstellt eine initiale Selektion: Alle FileEntries sind ausgewählt.
2. → **include** DL-002 (Ausschlussmuster anwenden): Dateien, die auf ein Muster passen, werden als `excluded=true` markiert und aus der Selektion entfernt.
3. System berechnet die Gesamtgrösse der aktiven Selektion.
4. System gibt die Selektion zurück.
5. Actor kann die Selektion verändern:
    - Einzelne Dateien an-/abwählen
    - Ganzes Paket an-/abwählen (Toggle aller Dateien des Pakets)
6. System aktualisiert die Gesamtgrösse nach jeder Änderung.
7. Actor bestätigt die Selektion.

## Alternative Paths

**2a. Alle Dateien ausgeschlossen:**
2a.1. Nach Anwendung der Ausschlussmuster sind 0 Dateien in der Selektion.
2a.2. System meldet: „Alle Dateien wurden durch Ausschlussmuster gefiltert."
2a.3. Actor kann manuell Dateien wieder hinzufügen (Schritt 5).

**5a. Actor wählt alle ab:**
5a.1. Actor entfernt alle Dateien aus der Selektion.
5a.2. Selektion hat 0 Dateien, Gesamtgrösse = 0.
5a.3. Download kann nicht gestartet werden, bis mindestens eine Datei ausgewählt ist.

## Business Rules

**BR-DL-001: Standard-Selektion**

- Initial sind alle Dateien ausgewählt (nach Anwendung der Ausschlussmuster).
- Manuell ausgeschlossene Dateien werden auch durch Paket-Toggle nicht wieder aktiviert, wenn sie auf ein Ausschlussmuster passen.

**BR-DL-002: Grössen-Berechnung**

- Nur Dateien mit bekannter Grösse fliessen in die Berechnung ein.
- Dateien ohne Grösse werden separat gezählt: „X Dateien, Y MB + Z Dateien mit unbekannter Grösse."

## Input

- `container`: Aufgelöster Container mit FileEntries

## Output

- `Selection` mit `selected_files[]`, `total_bytes`, `total_files`
