# Use Case: Drag-and-Drop

## Overview

**Use Case ID:** UI-006
**Use Case Name:** Drag-and-Drop
**Primary Actor:** Benutzer
**Goal:** Eine SFDL-Datei per Drag-and-Drop auf das App-Fenster ziehen, um sie zu öffnen.
**Implements:** SFDL-001
**Interface:** GUI (Dioxus Desktop)
**Requirements:** FR-13
**Status:** Stable

## Preconditions

- Die Desktop-App ist gestartet.

## Main Success Scenario

1. Benutzer zieht eine `.sfdl`-Datei auf das App-Fenster.
2. System zeigt visuellen Drop-Indikator (Rand leuchtet, Overlay „Datei hier ablegen").
3. Benutzer lässt die Datei los (Drop).
4. System prüft die Dateiendung.
5. System öffnet die SFDL-Datei (-> SFDL-001, wie UI-001 Schritt 5ff).
6. Container wird geladen und angezeigt.

## Alternative Flows

### A1: Falsche Dateiendung

**Trigger:** Die gedropte Datei hat nicht die Endung `.sfdl` (Schritt 4)
**Flow:**

1. System zeigt Fehlermeldung: „Nur .sfdl-Dateien werden unterstützt."
2. Kein Container wird geladen.

### A2: Mehrere Dateien

**Trigger:** Benutzer zieht mehrere Dateien gleichzeitig (Schritt 3)
**Flow:**

1. System öffnet nur die erste `.sfdl`-Datei.
2. Restliche Dateien werden ignoriert, mit Hinweis.

### A3: Drop während laufendem Download

**Trigger:** Ein Download läuft bereits (Schritt 3)
**Flow:**

1. System zeigt Bestätigungsdialog: „Laufenden Download abbrechen und neue Datei öffnen?"
2. Bei Bestätigung: Download wird abgebrochen, neuer Container wird geladen.
3. Bei Abbrechen: Nichts passiert, Download läuft weiter.

### A4: Drag-Leave

**Trigger:** Benutzer zieht die Datei wieder aus dem Fenster heraus (Schritt 2)
**Flow:**

1. Drop-Indikator verschwindet.
2. Kein weiterer Effekt.

## Postconditions

### Success Postconditions

- Container ist geladen und Dateiliste wird angezeigt.
- Bisheriger Container (falls vorhanden) wurde ersetzt.

### Failure Postconditions

- Bei A1: Kein Container geladen, vorheriger Zustand bleibt erhalten.
- Bei A3 (Abbrechen): Laufender Download läuft weiter.

## Business Rules

### BR-UI-006-001: Dateifilter

- Nur Dateien mit Endung `.sfdl` werden akzeptiert.
- Gross-/Kleinschreibung wird ignoriert.

### BR-UI-006-002: Fenster-Zustand

- Drop funktioniert in allen Zuständen (leer, Container geladen, Download läuft).
- Bei laufendem Download: Bestätigungsdialog erforderlich.

### BR-UI-006-003: Visuelles Feedback

| Phase      | UI-Feedback                           |
|------------|---------------------------------------|
| Drag-Enter | Drop-Indikator sichtbar               |
| Drag-Over  | Drop-Indikator bleibt aktiv           |
| Drag-Leave | Drop-Indikator verschwindet           |
| Drop       | Datei wird verarbeitet                |
