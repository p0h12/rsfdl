# Use Case: Hauptfenster

## Overview

**Use Case ID:** UI-001
**Use Case Name:** Hauptfenster
**Primary Actor:** Benutzer
**Goal:** SFDL-Container öffnen, Dateiinhalt einsehen, Dateien auswählen und Download starten/abbrechen.
**Implements:** SFDL-001, SFDL-003, DL-001
**Interface:** GUI (Dioxus Desktop)
**Status:** Stable

## Preconditions

- Die Desktop-App ist gestartet.
- Settings sind geladen (CFG-001 Variante A).

## Main Success Scenario

1. System zeigt das Hauptfenster mit Header (App-Name, Buttons: Open File, Create, Settings) und einer leeren Fläche mit Hinweis „Open an .sfdl file to begin".
2. Benutzer klickt „Open File".
3. System öffnet den OS-Dateidialog (Filter: `*.sfdl`).
4. Benutzer wählt eine SFDL-Datei.
5. System liest und parst die Datei (-> SFDL-001).
6. System prüft ob Auto-Passwörter greifen. Bei unverschlüsseltem oder auto-entschlüsseltem Container: weiter.
7. System löst BulkFolders auf, falls vorhanden (-> SFDL-003), und zeigt einen Lade-Indikator.
8. System zeigt Container-Info (Beschreibung, Uploader, Host, Protokoll) und Dateiliste mit Checkboxen.
9. System wendet Ausschlussmuster an (-> DL-002) und setzt die initiale Selektion.
10. Benutzer wählt/deselektiert Dateien über Checkboxen.
11. Benutzer klickt „Start Download".
12. System startet den Download (-> DL-004) und zeigt das Progress-Panel (-> UI-003).
13. Nach Abschluss zeigt System die Zusammenfassung (-> UI-004 via SummaryBanner).

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

1. System zeigt Warnung: „Failed to resolve bulk folders: [Fehlerdetail]".
2. Container wird mit den bereits vorhandenen Dateien angezeigt.
3. Use Case fährt mit Schritt 8 fort.

### A4: Keine Dateien ausgewählt

**Trigger:** Benutzer klickt „Start Download" ohne Selektion (Schritt 11)
**Flow:**

1. System zeigt Fehlermeldung: „No files selected".
2. Use Case verbleibt bei Schritt 10.

### A5: Download abbrechen

**Trigger:** Benutzer klickt „Cancel" während des Downloads (Schritt 12)
**Flow:**

1. System sendet Abbruch-Signal an alle aktiven Downloads (-> DL-006).
2. Progress-Panel zeigt abgebrochene Dateien.
3. System zeigt Zusammenfassung mit Cancelled-Status.

### A6: Neuen Container während Download öffnen

**Trigger:** Benutzer klickt „Open File" während ein Download läuft
**Flow:**

1. „Open File" ist deaktiviert während eines laufenden Downloads.

### A7: Download abgeschlossen — Reset

**Trigger:** Download ist abgeschlossen (Schritt 13), Benutzer klickt „Reset"
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

### BR-UI-001-001: Dateidialog-Filter

Der OS-Dateidialog filtert auf `*.sfdl`-Dateien.

### BR-UI-001-002: Button-Zustände

- „Start Download": Aktiv nur wenn Selektion > 0 und Phase = Idle.
- „Cancel": Sichtbar nur während Phase = Downloading.
- „Reset": Sichtbar nur wenn Phase = Done.
- „Open File": Deaktiviert während Phase = Downloading.

### BR-UI-001-003: Selektion

- Dateien werden per Checkbox einzeln oder paketweise ausgewählt.
- Ausschlussmuster (DL-002) setzen die initiale Selektion.

## Layout

### Zustand: Kein Container geladen

- Header mit App-Logo, App-Name und Buttons (Open File, Create, Settings, Theme-Toggle)
- Drop-Zone: Gestrichelter Rahmen, Icon, „SFDL-Container laden", „Dateien per Drag-and-Drop hier ablegen"
- Button „Datei auswählen" + Hinweis „.sfdl (v2 / v3)"

### Zustand: Container geladen

- Container-Toolbar: Zähler „N Container", Buttons „Hinzufügen" / „Alle entfernen"
- Pro Container eine aufklappbare **Card** mit:
    - Drag-Handle zum Sortieren
    - Icon (Lock bei verschlüsselt, Archiv bei offen)
    - Container-Name, Badges (Encrypted, V2/V3, Downloading)
    - Entfernen-Button (X)
    - **Info-Banner**: Server, Beschreibung, Pakete, Dateien
    - **Dateiliste**: Paket-Header mit Checkbox + Chevron, Dateien mit Checkbox, Icon, Name, Grösse
    - **Datei-Toolbar**: „Alle" / „Keine" Buttons, Selektionszähler
    - **Download-Aktionen**: Dateizähler + „Download starten" Button

### Zustand: Download läuft

- Progress-Sektion ersetzt Dateiliste in der Card
- Pro-Datei-Fortschritt mit Balken und Status
- Globaler Fortschritt + Geschwindigkeit
- „Abbrechen"-Button

### Zustand: Download abgeschlossen

- Ergebnis-Sektion in der Card (-> UI-004)

## Interaktionen

| Aktion                     | Auslöst                  | Use Case         |
|----------------------------|--------------------------|------------------|
| „Open File" Button         | OS-Dateidialog           | SFDL-001         |
| Datei per Drag-and-Drop    | Container laden          | SFDL-001, UI-006 |
| Checkbox Datei an/abwählen | Selektion aktualisieren  | DL-001           |
| „Start Download"           | Download-Session starten | DL-004           |
| „Cancel"                   | Globaler Abbruch         | DL-006           |
| „Settings" Button          | Einstellungen öffnen     | UI-005           |
