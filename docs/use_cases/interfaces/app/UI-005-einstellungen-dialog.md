# Use Case: Einstellungen-Dialog

## Overview

**Use Case ID:** UI-005
**Use Case Name:** Einstellungen-Dialog
**Primary Actor:** Benutzer
**Goal:** App-Einstellungen in einer eigenen View einsehen, bearbeiten und speichern.
**Implements:** CFG-001
**Status:** Stable

## Preconditions

- Die Desktop-App ist gestartet.
- Settings sind geladen (CFG-001 Variante A).

## Main Success Scenario

1. Benutzer klickt auf das Settings-Icon im Header.
2. System wechselt zur Einstellungen-View.
3. System zeigt die aktuellen Einstellungswerte in Card-basierten Sektionen an (gruppiert nach Kategorien, siehe Layout).
4. Benutzer aendert einen oder mehrere Werte.
5. System validiert die Eingaben inline (-> BR-UI-013).
6. Benutzer klickt „Speichern".
7. System speichert die Einstellungen auf Disk (-> CFG-001 Variante B).
8. System wechselt zurueck zum Hauptfenster (UI-001).

## Alternative Flows

### A1: Validierung schlaegt fehl

**Trigger:** Benutzer gibt ungueltige Werte ein (Schritt 5)
**Flow:**

1. Numerische Werte werden auf den gueltigen Bereich geclamp't.
2. Beim Speichern wird ggf. eine Fehlermeldung angezeigt (-> CFG-001 A4).
3. Benutzer korrigiert den Wert.
4. Use Case faehrt mit Schritt 5 fort.

### A2: Speichern schlaegt fehl

**Trigger:** Schreibfehler beim Speichern (Schritt 7)
**Flow:**

1. System zeigt Fehlermeldung: „Failed to save settings: [Fehlerdetail]".
2. Die Einstellungen im Speicher bleiben aktuell.
3. Use Case verbleibt in der Einstellungen-View.

### A3: Abbrechen

**Trigger:** Benutzer klickt „Abbrechen" oder den Zurueck-Pfeil (statt Schritt 6)
**Flow:**

1. System wechselt zurueck zum Hauptfenster.
2. Aenderungen im Speicher bleiben erhalten, sind aber nicht auf Disk persistiert.

### A4: Download-Verzeichnis per Dialog waehlen

**Trigger:** Benutzer klickt „Browse..." neben dem Download-Verzeichnis (Schritt 4)
**Flow:**

1. System oeffnet den OS-Dateidialog zur Ordnerauswahl.
2. Benutzer waehlt einen Ordner.
3. System uebernimmt den Pfad in das Textfeld.
4. Use Case faehrt mit Schritt 5 fort.

### A5: Ausschlussmuster hinzufuegen/entfernen

**Trigger:** Benutzer interagiert mit der Tag-Liste fuer Ausschlussmuster (Schritt 4)
**Flow:**

1. Hinzufuegen: Benutzer tippt ein Muster ins Eingabefeld und drueckt Enter oder klickt „+".
2. Entfernen: Benutzer klickt das „x" an einem bestehenden Tag.
3. Use Case faehrt mit Schritt 5 fort.

### A6: Auto-Passwort hinzufuegen/entfernen

**Trigger:** Benutzer interagiert mit der Tag-Liste fuer Passwoerter (Schritt 4)
**Flow:**

1. Hinzufuegen: Benutzer tippt ein Passwort ins Eingabefeld (verdeckt) und drueckt Enter oder klickt „+".
2. Entfernen: Benutzer klickt das „x" an einem bestehenden Tag.
3. Gespeicherte Passwoerter werden verdeckt als Bullet-Zeichen angezeigt.
4. Use Case faehrt mit Schritt 5 fort.

## Postconditions

### Success Postconditions

- Einstellungen sind auf Disk gespeichert und im Speicher aktualisiert.
- Benutzer befindet sich im Hauptfenster.
- Nachfolgende Downloads verwenden die neuen Einstellungen.

### Failure Postconditions

- Bei A2: Einstellungen auf Disk sind unveraendert; Aenderungen existieren nur im Speicher.

## Business Rules

### BR-UI-013: Inline-Validierung

Eingabefelder begrenzen Werte gemaess BR-CFG-003:

- Max. parallele Downloads: Spinner (1-20), Werte werden auf gueltigen Bereich geclamp't.
- Max. Geschwindigkeit: Eingabefeld (KB/s), >= 0, 0 = unbegrenzt.
- Max. Retries: Spinner (0-50), Werte werden geclamp't.
- Retry-Wartezeit: Eingabefeld (1-3600 Sekunden), Werte werden geclamp't.

### BR-UI-014: Toggle-Switches

- Boolean-Einstellungen werden als Toggle-Switches dargestellt (nicht als Checkboxen).
- Toggles: Auto-Extraktion, Archive nach Extraktion loeschen, Strikte Speicherplatzpruefung.

### BR-UI-015: Passwort-Anzeige

- Passwoerter werden verdeckt als Bullet-Zeichen in Tags angezeigt.
- Eingabefeld zum Hinzufuegen ist verdeckt (`type="password"`).
- Jedes Tag hat einen Entfernen-Button.

### BR-UI-016: Ausschlussmuster

- Muster werden als Tags angezeigt (ein Tag pro Muster).
- Eingabefeld + „+"-Button zum Hinzufuegen neuer Muster.
- Jedes Tag hat einen Entfernen-Button.

## Layout

Die Einstellungen-View besteht aus einem Zurueck-Pfeil + Titel-Header und mehreren Card-Sektionen:

### Allgemein

- Download-Verzeichnis: Textfeld (readonly) + „Browse..."-Button
- Max. parallele Downloads: Numerisches Eingabefeld (1-20)
- Max. Geschwindigkeit: Numerisches Eingabefeld (KB/s), 0 = unbegrenzt

### Download-Verhalten

- Max. Wiederholungen: Numerisches Eingabefeld (0-50)
- Retry-Wartezeit: Numerisches Eingabefeld (Sekunden)
- Strikte Speicherplatzpruefung: Toggle-Switch

### Nachbearbeitung

- Auto-Extraktion: Toggle-Switch (Standard: aus)
- Archive nach Extraktion loeschen: Toggle-Switch (Standard: aus)

### Ausschlussmuster

- Tag-Liste der aktuellen Muster mit Entfernen-Button pro Tag
- Eingabefeld + „+"-Button fuer neue Muster

### Auto-Passwoerter

- Tag-Liste gespeicherter Passwoerter (verdeckt als Bullet-Zeichen)
- Eingabefeld (`type="password"`) + „+"-Button

### Aktionen (Footer)

- „Abbrechen" -> Zurueck zum Hauptfenster ohne Speichern
- „Speichern" -> CFG-001 Variante B, dann zurueck zum Hauptfenster
