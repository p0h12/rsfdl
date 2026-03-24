# Use Case: SFDL-Datei oeffnen und parsen

## Overview

**Use Case ID:** SFDL-001
**Use Case Name:** SFDL-Datei oeffnen und parsen
**Primary Actor:** Benutzer
**Goal:** Eine `.sfdl`-Datei einlesen, parsen und als aufgeloesten Container zurueckgeben.
**Requirements:** FR-01
**Status:** Stable

## Preconditions

- Eine `.sfdl`-Datei ist als Pfad verfuegbar.

## Main Success Scenario

1. Actor uebergibt einen Dateipfad an das System.
2. System liest die Datei vom Dateisystem.
3. System erkennt die SFDL-Version anhand der XML-Struktur und des numerischen Werts:
    - `<ContainerVersion>` vorhanden, Wert 10 → v3
    - `<SFDLFileVersion>` vorhanden, Wert 6–9 → v2
    - Wert 0–5 oder >10 → ungueltig (siehe BR-SFDL-001)
4. System parst den XML-Inhalt gemaess erkannter Version.
5. **[v2]** System normalisiert die v2-Struktur intern auf das v3-Datenmodell.
6. System prueft, ob der Container verschluesselt ist (`Encrypted=true`).
    - Falls ja: → **extend** SFDL-002 (Container entschluesseln)
7. System erstellt ein Container-Objekt mit Metadaten und Paketstruktur.
8. → **include** SFDL-003 (Container-Inhalt aufloesen)
9. System gibt den aufgeloesten Container zurueck.

## Alternative Flows

### A1: Kein gueltiges XML

**Trigger:** Datei enthaelt kein valides XML (Schritt 3)
**Flow:**

1. System erkennt, dass die Datei kein valides XML enthaelt.
2. System meldet: "Datei ist keine gueltige SFDL-Datei."
3. Use Case endet.

### A2: Unbekannte SFDL-Version

**Trigger:** System findet weder v2- noch v3-Marker (Schritt 3)
**Flow:**

1. System findet weder v2- noch v3-Marker.
2. System meldet: "Unbekanntes SFDL-Format."
3. Use Case endet.

### A3: Pflichtfelder fehlen

**Trigger:** System erkennt fehlende Pflichtfelder nach dem Parsen (Schritt 4)
**Flow:**

1. System erkennt fehlende Pflichtfelder (z.B. kein Host, keine Pakete).
2. System meldet: "SFDL-Datei unvollstaendig: [fehlende Felder]."
3. Use Case endet.

### A4: Datei nicht lesbar

**Trigger:** System kann die Datei nicht lesen (Schritt 2)
**Flow:**

1. System kann die Datei nicht lesen (Berechtigung, nicht vorhanden).
2. System meldet: "Datei konnte nicht geoeffnet werden: [OS-Fehler]."
3. Use Case endet.

## Postconditions

### Success Postconditions

- Ein Container-Objekt mit geparsten Metadaten und Paketstruktur liegt vor.

### Failure Postconditions

- Fehlermeldung mit Ursache vorhanden.
- Kein teilweise geparster Zustand.

## Business Rules

### BR-SFDL-001: Versionserkennung

Numerische ContainerVersion-Werte (Quelle: SFDL.NET Referenzimplementierung):

| ContainerVersion | SFDL Version | Verhalten                         |
|------------------|--------------|-----------------------------------|
| 0                | ungueltig    | Fehler: "nicht kompatibel"        |
| 1–5              | v1           | Fehler: "nicht mehr unterstuetzt" |
| 6–9              | v2 (legacy)  | Wird intern zu v3 konvertiert     |
| **10**           | **v3**       | Aktuelles Format, direkt parsen   |
| >10              | ungueltig    | Fehler: "nicht kompatibel"        |

- v3 hat Vorrang: Enthaelt eine Datei sowohl v2- als auch v3-Marker, wird v3 verwendet.
- Die Versionserkennung basiert auf dem XML-Element-Namen (`<ContainerVersion>` vs `<SFDLFileVersion>`) und dem numerischen Wert.

### BR-SFDL-002: v2-Normalisierung

- v2-Felder werden auf das v3-Datenmodell gemappt (siehe entity-model.md).
- Felder ohne v2-Entsprechung erhalten Standardwerte.

## Input

- `path`: Dateisystempfad zur `.sfdl`-Datei

## Output

- `Container` mit `ConnectionInfo`, `Package[]`, Metadaten
- `encrypted: bool` — ob Entschluesselung noetig war
- `version: v2 | v3` — erkannte Originalversion
