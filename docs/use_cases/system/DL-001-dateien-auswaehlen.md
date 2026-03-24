# Use Case: Dateien zum Download auswählen

## Overview

**Use Case ID:** DL-001
**Use Case Name:** Dateien zum Download auswählen
**Primary Actor:** Benutzer
**Goal:** Aus einem geöffneten SFDL-Container die gewünschten Dateien für den Download auswählen.
**Requirements:** FR-04, FR-17
**Status:** Stable

## Preconditions

- Ein aufgelöster Container mit FileEntries liegt vor (SFDL-003 abgeschlossen).
- Ausschlussmuster sind in den Einstellungen konfiguriert (CFG-001).

## Main Success Scenario

1. System erstellt eine initiale Selektion: Alle FileEntries sind ausgewählt.
2. System wendet Ausschlussmuster an (-> include DL-002): Dateien, die auf ein Muster passen, werden aus der Selektion entfernt.
3. System berechnet die Gesamtgrösse der aktiven Selektion.
4. System zeigt die Selektion an (Dateiliste mit Checkboxen, Gesamtgrösse).
5. Benutzer verändert die Selektion nach Bedarf:
    - Einzelne Dateien an-/abwählen
    - Ganzes Paket an-/abwählen (Toggle aller Dateien des Pakets)
    - "Alle"- / "Keine"-Buttons
6. System aktualisiert die Gesamtgrösse und den Selektionszähler nach jeder Änderung.
7. Benutzer bestätigt die Selektion und startet den Download.

## Alternative Flows

### A1: Alle Dateien durch Muster ausgeschlossen

**Trigger:** Nach Anwendung der Ausschlussmuster sind 0 Dateien in der Selektion (Schritt 2)
**Flow:**

1. System zeigt die Dateiliste mit allen Dateien als abgewählt an.
2. Selektionszähler zeigt "0 von N ausgewählt".
3. "Download starten" ist deaktiviert.
4. Benutzer kann manuell Dateien wieder hinzufügen (Schritt 5).

### A2: Benutzer wählt alle ab

**Trigger:** Benutzer entfernt alle Dateien aus der Selektion (Schritt 5)
**Flow:**

1. Selektion hat 0 Dateien, Gesamtgrösse = 0.
2. "Download starten" ist deaktiviert.
3. Benutzer muss mindestens eine Datei auswählen, um fortzufahren.

### A3: CLI-Modus (keine manuelle Selektion)

**Trigger:** Download wird über die CLI gestartet (Schritt 5 entfällt)
**Flow:**

1. System verwendet die initiale Selektion (nach Ausschlussmuster) ohne manuelle Änderung.
2. Use Case fährt mit Schritt 7 fort.

## Postconditions

### Success Postconditions

- Eine Selektion mit mindestens einer ausgewählten Datei liegt vor.
- Die Gesamtgrösse der selektierten Dateien ist berechnet.
- Der Download kann gestartet werden (-> DL-004).

### Failure Postconditions

- Keine Dateien ausgewählt: Download kann nicht gestartet werden.
- Benutzer kann die Selektion jederzeit ändern.

## Business Rules

### BR-DL-001: Standard-Selektion

- Initial sind alle Dateien ausgewählt, abzüglich der durch Ausschlussmuster gefilterten.
- Die initiale Selektion wird als flache Boolean-Liste berechnet, aligniert mit der Dateiliste über alle Pakete.

### BR-DL-002: Grössen-Berechnung

- Nur Dateien mit bekannter Grösse (file_size > 0) fliessen in die Berechnung ein.
- BulkFolder-Dateien haben nach Auflösung (SFDL-003) eine bekannte Grösse.

## Input

- Aufgelöster Container mit FileEntries (über alle Pakete)
- Konfigurierte Ausschlussmuster (aus Einstellungen)

## Output

- Selektion: Boolean-Liste pro Datei (ausgewählt / nicht ausgewählt)
- Gesamtgrösse der selektierten Dateien in Bytes
- Anzahl selektierter Dateien
