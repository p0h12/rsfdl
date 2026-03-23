# CR-006: Metadaten setzen

**Use Case ID:** CR-006
**Requirements:** FR-25
**Primary Actor:** Benutzer
**Trigger:** Wird von CR-001 aufgerufen, wenn Metadaten angegeben werden.
**Preconditions:** Ein Container-Objekt wird gerade erstellt.
**Postconditions (Erfolg):** Container enthält die gesetzten Metadaten.
**Postconditions (Fehlschlag):** Standardwerte werden verwendet.

---

## Main Success Scenario

1. Actor gibt optionale Metadaten an:
    - Beschreibung (z.B. Release-Name)
    - Uploader-Name
    - Max-Download-Threads
2. System validiert die Werte (→ BR-CR-013).
3. System setzt die Metadaten im Container-Objekt.
4. System gibt den Container an CR-001 zurück.

## Alternative Paths

**1a. Keine Metadaten angegeben:**
1a.1. System verwendet die Standardwerte (→ BR-CR-013).
1a.2. Use Case endet erfolgreich.

**2a. Ungültiger Wert für MaxDownloadThreads:**
2a.1. Actor gibt einen Wert < 1 oder > 10 an.
2a.2. System meldet: „MaxDownloadThreads muss zwischen 1 und 10 liegen."
2a.3. Use Case endet mit Fehler.

## Business Rules

**BR-CR-013: Standardwerte**

- `Description`: leer
- `Uploader`: "rsfdl"
- `MaxDownloadThreads`: 3

**BR-CR-014: Validierung**

- `MaxDownloadThreads` muss ein positiver Integer sein (1–10).
- `Description` und `Uploader` sind Freitextfelder ohne Längenbeschränkung.

## Input

- `description`: Optional — Beschreibung des Containers
- `uploader`: Optional — Uploader-Name (Standard: "rsfdl")
- `max_threads`: Optional — Maximale Download-Threads (Standard: 3)

## Output (Erfolg)

- Container mit gesetzten Metadaten
