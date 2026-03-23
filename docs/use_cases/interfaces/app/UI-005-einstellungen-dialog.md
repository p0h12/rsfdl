# Use Case: Einstellungen-Dialog

## Overview

**Use Case ID:** UI-005
**Use Case Name:** Einstellungen-Dialog
**Primary Actor:** Benutzer
**Goal:** App-Einstellungen in einer eigenen View einsehen, bearbeiten, speichern und auf Standardwerte zurücksetzen.
**Implements:** CFG-001
**Interface:** GUI (Dioxus Desktop)
**Status:** Draft

## Preconditions

- Die Desktop-App ist gestartet und das Hauptfenster (UI-001) wird angezeigt.
- Settings sind geladen (CFG-001 Variante A).

## Main Success Scenario

1. Benutzer klickt auf das Zahnrad-Icon / Menü im Hauptfenster.
2. System wechselt zur Einstellungen-View.
3. System zeigt die aktuellen Einstellungswerte in den Formularfeldern an (gruppiert nach Kategorien, siehe Layout).
4. Benutzer ändert einen oder mehrere Werte.
5. System validiert die Eingaben inline (-> BR-UI-005-001).
6. Benutzer klickt „Save".
7. System speichert die Einstellungen auf Disk (-> CFG-001 Variante B).
8. Benutzer klickt „Back".
9. System wechselt zurück zum Hauptfenster (UI-001).

## Alternative Flows

### A1: Validierung schlägt fehl

**Trigger:** Benutzer gibt ungültige Werte ein (Schritt 5)
**Flow:**

1. System markiert das betroffene Feld visuell als ungültig (rot).
2. „Save" bleibt aktiv, aber beim Speichern wird eine Fehlermeldung angezeigt (-> CFG-001 A4).
3. Benutzer korrigiert den Wert.
4. Use Case fährt mit Schritt 5 fort.

### A2: Speichern schlägt fehl

**Trigger:** Schreibfehler beim Speichern (Schritt 7)
**Flow:**

1. System zeigt Fehlermeldung: „Failed to save settings: [Fehlerdetail]".
2. Die Einstellungen im Speicher bleiben aktuell.
3. Use Case verbleibt in der Einstellungen-View.

### A3: Einstellungen zurücksetzen

**Trigger:** Benutzer klickt „Reset" (statt Schritt 6)
**Flow:**

1. System setzt alle Werte auf Standardwerte zurück (-> CFG-001 Variante C).
2. System aktualisiert alle Formularfelder mit den Standardwerten.
3. Use Case fährt mit Schritt 4 fort (Benutzer kann weiter bearbeiten).

### A4: Reset schlägt fehl

**Trigger:** Dateisystem-Fehler beim Zurücksetzen (A3, Schritt 1)
**Flow:**

1. System zeigt Fehlermeldung: „Failed to reset settings: [Fehlerdetail]".
2. Die bisherigen Einstellungen bleiben erhalten.
3. Use Case verbleibt in der Einstellungen-View.

### A5: Abbrechen ohne Speichern

**Trigger:** Benutzer klickt „Back" ohne vorher zu speichern (statt Schritt 6)
**Flow:**

1. System wechselt zurück zum Hauptfenster.
2. Nicht gespeicherte Änderungen im Speicher bleiben erhalten, sind aber nicht auf Disk persistiert.

### A6: Download-Verzeichnis per Dialog wählen

**Trigger:** Benutzer klickt „Browse..." neben dem Download-Verzeichnis (Schritt 4)
**Flow:**

1. System öffnet den OS-Dateidialog zur Ordnerauswahl.
2. Benutzer wählt einen Ordner.
3. System übernimmt den Pfad in das Textfeld.
4. Use Case fährt mit Schritt 5 fort.

## Postconditions

### Success Postconditions

- Einstellungen sind auf Disk gespeichert und im Speicher aktualisiert.
- Benutzer befindet sich im Hauptfenster.
- Nachfolgende Downloads verwenden die neuen Einstellungen.

### Failure Postconditions

- Bei A1/A2: Einstellungen auf Disk sind unverändert; Änderungen existieren nur im Speicher.
- Bei A4: Einstellungen bleiben auf dem Stand vor dem Reset-Versuch.

## Business Rules

### BR-UI-005-001: Inline-Validierung

Eingabefelder begrenzen Werte gemäss BR-CFG-003:

- Max. parallele Downloads: Spinner (1–20), Werte werden auf gültigen Bereich geclamp't.
- Max. Geschwindigkeit: Eingabefeld (KB/s), >= 0, 0 = unbegrenzt.
- Max. Retries: Spinner (0–50), Werte werden geclamp't.
- Retry-Wartezeit: Eingabefeld (1–3600 Sekunden), Werte werden geclamp't.

### BR-UI-005-002: Bedingte Felder

- „Delete Archives After Extraction" ist nur aktiv, wenn „Auto Extract Archives" aktiviert ist.

### BR-UI-005-003: Passwort-Anzeige

- Passwörter werden als Klartext im Textarea bearbeitet (ein Passwort pro Zeile).
- Leere Zeilen werden beim Parsen ignoriert.

### BR-UI-005-004: Ausschlussmuster

- Ein Glob-Muster pro Zeile im Textarea.
- Leere Zeilen werden beim Parsen ignoriert.

## Layout

### Allgemein

- Download-Verzeichnis: Textfeld (readonly) + „Browse..."-Button
- Max. parallele Downloads: Spinner (1–20)
- Max. Geschwindigkeit: Eingabefeld (KB/s), 0 = unbegrenzt

### Download-Verhalten

- Max. Retries: Spinner (0–50)
- Retry-Wartezeit: Eingabefeld (Sekunden)

### Nachbearbeitung

- Auto-Extraktion: Checkbox (Standard: aus)
- Archive nach Extraktion löschen: Checkbox (Standard: aus, nur aktiv wenn Auto-Extraktion an)
- Speicherplatz strikt prüfen: Checkbox

### Ausschlussmuster

- Mehrzeiliges Textfeld, ein Glob-Muster pro Zeile

### Passwörter

- Mehrzeiliges Textfeld, ein Passwort pro Zeile

### Aktionen

- „Save" -> CFG-001 Variante B
- „Reset" -> CFG-001 Variante C
- „Back" -> Zurück zum Hauptfenster (UI-001)
