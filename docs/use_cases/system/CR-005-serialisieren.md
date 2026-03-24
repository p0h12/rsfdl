# Use Case: Container serialisieren

## Overview

**Use Case ID:** CR-005
**Use Case Name:** Container serialisieren
**Primary Actor:** System
**Goal:** Einen vollständigen Container-Objekt in einen gültigen SFDL v3 XML-String serialisieren.
**Requirements:** FR-24
**Status:** Stable

## Preconditions

- Ein vollständiger Container liegt vor (ggf. verschlüsselt).
- Wird von CR-001 aufgerufen (include).

## Main Success Scenario

1. System erstellt den XML-Header: `<?xml version="1.0" encoding="utf-8"?>`.
2. System serialisiert das Container-Objekt in das v3 XML-Schema:
    - Root-Element: `<Container>`
    - Alle Elementnamen in PascalCase
    - Enum-Werte als Strings (z.B. `UTF8`, `Binary`, `Passive`)
3. System serialisiert leere Listen als leere XML-Elemente (nicht weglassen).
4. System gibt den XML-String zurück.

## Alternative Flows

### A1: Ungültige Daten

**Trigger:** System erkennt Daten, die nicht im XML-Schema abbildbar sind (Schritt 2)
**Flow:**

1. System erkennt Daten, die nicht im XML-Schema abbildbar sind.
2. System meldet: "Container enthält ungültige Daten: [Details]."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Gültiges SFDL v3 XML als String liegt vor.

### Failure Postconditions

- Fehlermeldung mit Ursache liegt vor.
- Keine Datei wird geschrieben.

## Business Rules

### BR-CR-010: XML-Konformität

- Ausgabe folgt dem v3 XML-Schema mit Root-Element `<Container>`.
- PascalCase für alle XML-Elementnamen: `ContainerVersion`, `MaxDownloadThreads`, etc.
- Encoding: UTF-8.

### BR-CR-011: Round-Trip

- `parse_sfdl(serialize_v3(container))` muss einen semantisch identischen Container ergeben.
- Dies gilt als Hauptvalidierungskriterium.

### BR-CR-012: Leere Elemente

- Leere Listen (`FileList`, `BulkFolderList`) erzeugen leere XML-Elemente, werden nicht weggelassen.

## Input

- `container`: Vollständiger Container (optional verschlüsselt)

## Output

- XML-String im SFDL v3 Format
