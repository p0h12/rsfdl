# Use Case: Passwort-Eingabe

## Overview

**Use Case ID:** UI-002
**Use Case Name:** Passwort-Eingabe
**Primary Actor:** Benutzer
**Goal:** Passwort für einen verschlüsselten SFDL-Container eingeben und den Container entschlüsseln.
**Implements:** SFDL-002
**Interface:** GUI (Dioxus Desktop)
**Status:** Stable

## Preconditions

- Ein verschlüsselter SFDL-Container wurde geöffnet (SFDL-001).
- Kein Auto-Passwort hat gepasst.
- Die Container-Card zeigt den Zustand „NeedsPassword".

## Main Success Scenario

1. System zeigt in der Container-Card eine Inline-Passwort-Sektion mit Schluessel-Icon, Titel „Passwort erforderlich" und Hinweis.
2. Benutzer gibt das Passwort in das Eingabefeld ein.
3. Benutzer klickt „Entschluesseln" oder drueckt Enter.
4. System versucht den Container zu entschluesseln (-> SFDL-002).
5. Entschluesselung erfolgreich: Passwort-Sektion wird durch die Dateiliste ersetzt.
6. System loest BulkFolders auf, falls vorhanden (-> SFDL-003).
7. Container-Card zeigt Info-Banner und Dateiliste.

## Alternative Flows

### A1: Falsches Passwort

**Trigger:** Entschluesselung schlaegt fehl mit `InvalidPassword` (Schritt 4)
**Flow:**

1. System zeigt roten Hinweis unter dem Eingabefeld: „Invalid password".
2. Passwort-Sektion bleibt sichtbar.
3. Use Case faehrt mit Schritt 2 fort.

### A2: Entschluesselungsfehler (kein Passwort-Fehler)

**Trigger:** Entschluesselung schlaegt aus anderem Grund fehl (Schritt 4)
**Flow:**

1. System zeigt Fehlermeldung: „Decryption failed: [Fehlerdetail]".
2. Passwort-Sektion bleibt sichtbar.
3. Use Case faehrt mit Schritt 2 fort.

### A3: Container entfernen

**Trigger:** Benutzer klickt den Entfernen-Button (X) im Card-Header (statt Schritt 3)
**Flow:**

1. Container wird aus der Liste entfernt.
2. Use Case endet.

### A4: Auto-Passwort-Treffer

**Trigger:** Beim Oeffnen der SFDL-Datei passt ein Auto-Passwort (vor Schritt 1)
**Flow:**

1. Die Passwort-Eingabe wird komplett uebersprungen.
2. Container wird direkt mit Phase „Ready" geladen (-> UI-001 Schritt 7).

## Postconditions

### Success Postconditions

- Container ist entschluesselt und zeigt die Dateiliste.
- Container-Phase ist „Ready".
- BulkFolders werden aufgeloest (falls vorhanden).

### Failure Postconditions

- Bei A1/A2: Passwort-Sektion bleibt sichtbar, Benutzer kann erneut versuchen.
- Bei A3: Container ist aus der Liste entfernt.

## Business Rules

### BR-UI-005: Passwort-Eingabe

- Eingabefeld ist verdeckt (`type="password"`).
- Sichtbarkeits-Toggle (Auge-Icon) erlaubt das Passwort anzuzeigen.
- Enter-Taste loest „Entschluesseln" aus.

### BR-UI-006: Inline-Darstellung

- Die Passwort-Eingabe ist inline im Card-Body (kein separater modaler Dialog).
- Die Card bleibt in der Container-Liste an ihrer Position.
- Andere Container bleiben unberuehrt.

## Layout

- Inline-Sektion in der Container-Card (kein modaler Dialog)
- Schluessel-Icon + Titel „Passwort erforderlich"
- Hinweis: „Dieser Container ist verschluesselt. Bitte Passwort eingeben."
- Passwort-Eingabefeld mit Sichtbarkeits-Toggle (Auge-Icon)
- Button „Entschluesseln"
- Fehleranzeige (rot, unter dem Eingabefeld, nur bei Fehler)
