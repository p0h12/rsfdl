# Use Case: Dateien zum Download auswaehlen

## Overview

**Use Case ID:** DL-001
**Use Case Name:** Dateien zum Download auswaehlen
**Primary Actor:** Benutzer
**Goal:** Aus einem geoeffneten SFDL-Container die gewuenschten Dateien fuer den Download auswaehlen.
**Requirements:** FR-04, FR-17
**Status:** Stable

## Preconditions

- Ein aufgeloester Container mit FileEntries liegt vor (SFDL-003 abgeschlossen).
- Ausschlussmuster sind in den Einstellungen konfiguriert (CFG-001).

## Main Success Scenario

1. System erstellt eine initiale Selektion: Alle FileEntries sind ausgewaehlt.
2. System wendet Ausschlussmuster an (-> include DL-002): Dateien, die auf ein Muster passen, werden aus der Selektion entfernt.
3. System berechnet die Gesamtgroesse der aktiven Selektion.
4. System zeigt die Selektion an (Dateiliste mit Checkboxen, Gesamtgroesse).
5. Benutzer veraendert die Selektion nach Bedarf:
    - Einzelne Dateien an-/abwaehlen
    - Ganzes Paket an-/abwaehlen (Toggle aller Dateien des Pakets)
    - "Alle"- / "Keine"-Buttons
6. System aktualisiert die Gesamtgroesse und den Selektionszaehler nach jeder Aenderung.
7. Benutzer bestaetigt die Selektion und startet den Download.

## Alternative Flows

### A1: Alle Dateien durch Muster ausgeschlossen

**Trigger:** Nach Anwendung der Ausschlussmuster sind 0 Dateien in der Selektion (Schritt 2)
**Flow:**

1. System zeigt die Dateiliste mit allen Dateien als abgewaehlt an.
2. Selektionszaehler zeigt "0 von N ausgewaehlt".
3. "Download starten" ist deaktiviert.
4. Benutzer kann manuell Dateien wieder hinzufuegen (Schritt 5).

### A2: Benutzer waehlt alle ab

**Trigger:** Benutzer entfernt alle Dateien aus der Selektion (Schritt 5)
**Flow:**

1. Selektion hat 0 Dateien, Gesamtgroesse = 0.
2. "Download starten" ist deaktiviert.
3. Benutzer muss mindestens eine Datei auswaehlen, um fortzufahren.

### A3: CLI-Modus (keine manuelle Selektion)

**Trigger:** Download wird ueber die CLI gestartet (Schritt 5 entfaellt)
**Flow:**

1. System verwendet die initiale Selektion (nach Ausschlussmuster) ohne manuelle Aenderung.
2. Use Case faehrt mit Schritt 7 fort.

## Postconditions

### Success Postconditions

- Eine Selektion mit mindestens einer ausgewaehlten Datei liegt vor.
- Die Gesamtgroesse der selektierten Dateien ist berechnet.
- Der Download kann gestartet werden (-> DL-004).

### Failure Postconditions

- Keine Dateien ausgewaehlt: Download kann nicht gestartet werden.
- Benutzer kann die Selektion jederzeit aendern.

## Business Rules

### BR-DL-001: Standard-Selektion

- Initial sind alle Dateien ausgewaehlt, abzueglich der durch Ausschlussmuster gefilterten.
- Die initiale Selektion wird als flache Boolean-Liste berechnet, aligniert mit der Dateiliste ueber alle Pakete.

### BR-DL-002: Groessen-Berechnung

- Nur Dateien mit bekannter Groesse (file_size > 0) fliessen in die Berechnung ein.
- BulkFolder-Dateien haben nach Aufloesung (SFDL-003) eine bekannte Groesse.

## Input

- Aufgeloester Container mit FileEntries (ueber alle Pakete)
- Konfigurierte Ausschlussmuster (aus Einstellungen)

## Output

- Selektion: Boolean-Liste pro Datei (ausgewaehlt / nicht ausgewaehlt)
- Gesamtgroesse der selektierten Dateien in Bytes
- Anzahl selektierter Dateien
