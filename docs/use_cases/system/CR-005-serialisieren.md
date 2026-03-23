# CR-005: Container serialisieren

**Use Case ID:** CR-005
**Requirements:** FR-24
**Primary Actor:** System
**Trigger:** Wird von CR-001 aufgerufen (include).
**Preconditions:** Ein vollständiger Container liegt vor (ggf. verschlüsselt).
**Postconditions (Erfolg):** Gültiges SFDL v3 XML als String.
**Postconditions (Fehlschlag):** Fehlermeldung mit Ursache. Keine Datei wird geschrieben.

---

## Main Success Scenario

1. System erstellt den XML-Header: `<?xml version="1.0" encoding="utf-8"?>`.
2. System serialisiert das Container-Objekt in das v3 XML-Schema:
    - Root-Element: `<Container>`
    - Alle Elementnamen in PascalCase
    - Enum-Werte als Strings (z.B. `UTF8`, `Binary`, `Passive`)
3. System serialisiert leere Listen als leere XML-Elemente (nicht weglassen).
4. System gibt den XML-String zurück.

## Alternative Paths

**2a. Ungültige Daten:**
2a.1. System erkennt Daten, die nicht im XML-Schema abbildbar sind.
2a.2. System meldet: „Container enthält ungültige Daten: [Details]."
2a.3. Use Case endet mit Fehler.

## Business Rules

**BR-CR-010: XML-Konformität**

- Ausgabe folgt dem v3 XML-Schema mit Root-Element `<Container>`.
- PascalCase für alle XML-Elementnamen: `ContainerVersion`, `MaxDownloadThreads`, etc.
- Encoding: UTF-8.

**BR-CR-011: Round-Trip**

- `parse_sfdl(serialize_v3(container))` muss einen semantisch identischen Container ergeben.
- Dies gilt als Hauptvalidierungskriterium.

**BR-CR-012: Leere Elemente**

- Leere Listen (`FileList`, `BulkFolderList`) erzeugen leere XML-Elemente, werden nicht weggelassen.

## Input

- `container`: Vollständiger Container (optional verschlüsselt)

## Output (Erfolg)

- XML-String im SFDL v3 Format
