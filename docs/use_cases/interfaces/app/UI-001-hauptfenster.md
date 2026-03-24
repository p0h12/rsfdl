# Use Case: Hauptfenster

## Overview

**Use Case ID:** UI-001
**Use Case Name:** Hauptfenster
**Primary Actor:** Benutzer
**Goal:** SFDL-Container öffnen, Dateiinhalt einsehen, Dateien auswählen und Download starten/abbrechen.
**Implements:** SFDL-001, SFDL-003, DL-001
**Status:** Stable

## Preconditions

- Die Desktop-App ist gestartet.
- Settings sind geladen (CFG-001 Variante A).

## Main Success Scenario

1. System zeigt das Hauptfenster mit Header (App-Name, Buttons: Open, Settings, Theme-Toggle) und der Drop-Zone.
2. Benutzer klickt "Open" im Header oder "Datei auswaehlen" in der Drop-Zone.
3. System oeffnet den OS-Dateidialog (Filter: `*.sfdl`, Mehrfachauswahl).
4. Benutzer wählt eine oder mehrere SFDL-Dateien.
5. System liest und parst jede Datei (-> SFDL-001) und fügt sie als Card zur Container-Liste hinzu.
6. System prüft pro Container ob Auto-Passwörter greifen. Bei unverschlüsseltem oder auto-entschlüsseltem Container: weiter.
7. System löst BulkFolders auf, falls vorhanden (-> SFDL-003), und zeigt einen Lade-Indikator.
8. System zeigt pro Container eine Card mit Info-Banner und Dateiliste mit Checkboxen.
9. System wendet Ausschlussmuster an (-> DL-002) und setzt die initiale Selektion.
10. Benutzer wählt/deselektiert Dateien über Checkboxen.
11. Benutzer klickt "Download starten" in einer Container-Card.
12. System startet den Download (-> DL-004) und zeigt das Progress-Panel (-> UI-003) in der Card.
13. Nach Abschluss zeigt System die Zusammenfassung (-> UI-004) in der Card.

## Alternative Flows

### A1: Verschlüsselter Container — Passwort erforderlich

**Trigger:** Container ist verschlüsselt und kein Auto-Passwort passt (Schritt 6)
**Flow:**

1. System zeigt den Passwort-Dialog (-> UI-002).
2. Benutzer gibt Passwort ein und bestätigt.
3. System entschlüsselt den Container (-> SFDL-002).
4. Use Case fährt mit Schritt 7 fort.

### A2: Parse-Fehler

**Trigger:** Datei kann nicht geparst werden (Schritt 5)
**Flow:**

1. System zeigt Fehlermeldung als Error-Banner.
2. Hauptfenster bleibt im leeren Zustand.

### A3: BulkFolder-Auflösung fehlgeschlagen

**Trigger:** FTP-Verbindung für BulkFolder schlägt fehl (Schritt 7)
**Flow:**

1. System zeigt Warnung: "Failed to resolve bulk folders: [Fehlerdetail]".
2. Container wird mit den bereits vorhandenen Dateien angezeigt.
3. Use Case fährt mit Schritt 8 fort.

### A4: Keine Dateien ausgewählt

**Trigger:** Benutzer klickt "Start Download" ohne Selektion (Schritt 11)
**Flow:**

1. System zeigt Fehlermeldung: "No files selected".
2. Use Case verbleibt bei Schritt 10.

### A5: Download abbrechen

**Trigger:** Benutzer klickt "Cancel" während des Downloads (Schritt 12)
**Flow:**

1. System sendet Abbruch-Signal an alle aktiven Downloads (-> DL-006).
2. Progress-Panel zeigt abgebrochene Dateien.
3. System zeigt Zusammenfassung mit Cancelled-Status.

### A6: Weiteren Container hinzufügen

**Trigger:** Benutzer klickt "Hinzufügen" oder öffnet eine weitere Datei (jederzeit)
**Flow:**

1. System fügt den neuen Container als weitere Card zur Liste hinzu.
2. Bestehende Container und laufende Downloads bleiben unberührt.
3. Use Case fährt mit Schritt 5 für den neuen Container fort.

### A7: Download abgeschlossen — Reset

**Trigger:** Download ist abgeschlossen (Schritt 13), Benutzer klickt "Reset"
**Flow:**

1. System setzt den Download-Zustand zurück (Phase, File States, Summary, Progress).
2. Dateiliste wird wieder mit Checkboxen angezeigt.
3. Use Case fährt mit Schritt 10 fort.

## Postconditions

### Success Postconditions

- Container ist geladen und Dateiliste wird angezeigt.
- Nach Download: Zusammenfassung ist sichtbar.
- Heruntergeladene Dateien befinden sich im konfigurierten Download-Verzeichnis.

### Failure Postconditions

- Bei A2: Kein Container geladen, Hauptfenster bleibt leer.
- Bei A5: Teilweise heruntergeladene Dateien bleiben auf Disk.

## Business Rules

### BR-UI-001: Dateidialog-Filter

Der OS-Dateidialog filtert auf `*.sfdl`-Dateien.

### BR-UI-002: Button-Zustände (pro Container-Card)

- "Download starten": Aktiv nur wenn Selektion > 0 und Phase = Idle.
- "Abbrechen": Sichtbar nur während Phase = Downloading.
- "Reset": Sichtbar nur wenn Phase = Done.
- "Entfernen" (X): Jederzeit verfügbar, entfernt den Container aus der Liste.

### BR-UI-004: Multi-Container und Download-Reihenfolge

- Mehrere Container können gleichzeitig geladen sein.
- Container werden als Cards in einer sortierbaren Liste dargestellt.
- Up/Down-Buttons im Card-Header erlauben die Reihenfolge der Cards zu aendern.
- Downloads werden von oben nach unten abgearbeitet: Der oberste Container wird zuerst heruntergeladen, dann der nächste, usw.
- Die verfügbaren Download-Slots (`max_threads`) werden auf den aktuell aktiven Container verteilt.
- Erst wenn ein Container fertig ist (Done), beginnt der nächste.
- "Alle entfernen" löscht alle Container aus der Liste.

### BR-UI-003: Selektion

- Dateien werden per Checkbox einzeln oder paketweise ausgewählt.
- Ausschlussmuster (DL-002) setzen die initiale Selektion.

## Layout

### Zustand: Kein Container geladen

- Header mit App-Name, Buttons (Open, Settings), Theme-Toggle
- Drop-Zone: "SFDL-Container laden" + "Datei auswaehlen"-Button

### Zustand: Container geladen

- Container-Toolbar: Zähler "N Container", Buttons "Hinzufügen" / "Alle entfernen"
- Pro Container eine aufklappbare **Card** mit:
    - Up/Down-Buttons zum Sortieren
    - Container-Name, Badges (Encrypted, V2/V3)
    - Entfernen-Button (X)
    - **Info-Banner**: Server, Beschreibung, Pakete, Dateien
    - **Dateiliste**: Paket-Header mit Checkbox, Dateien mit Checkbox, Dateiname, Grösse
    - **Download-Aktionen**: Selektionszähler + "Download starten" Button

### Zustand: Download läuft

- Progress-Panel (-> UI-003) ersetzt Dateiliste in der Card
- "Abbrechen"-Button

### Zustand: Download abgeschlossen

- Summary-Banner (-> UI-004) in der Card

## Interaktionen

| Aktion                     | Auslöst                  | Use Case         |
|----------------------------|--------------------------|------------------|
| "Open File" Button         | OS-Dateidialog           | SFDL-001         |
| Datei per Drag-and-Drop    | Container laden          | SFDL-001, UI-006 |
| Checkbox Datei an/abwählen | Selektion aktualisieren  | DL-001           |
| "Start Download"           | Download-Session starten | DL-004           |
| "Cancel"                   | Globaler Abbruch         | DL-006           |
| "Settings" Button          | Einstellungen öffnen     | UI-005           |
