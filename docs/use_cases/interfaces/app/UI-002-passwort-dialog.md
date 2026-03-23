# Use Case: Passwort-Dialog

## Overview

**Use Case ID:** UI-002
**Use Case Name:** Passwort-Dialog
**Primary Actor:** Benutzer
**Goal:** Passwort für einen verschlüsselten SFDL-Container eingeben und den Container entschlüsseln.
**Implements:** SFDL-002
**Interface:** GUI (Dioxus Desktop)
**Status:** Stable

## Preconditions

- Ein verschlüsselter SFDL-Container wurde geöffnet (SFDL-001).
- Kein Auto-Passwort hat gepasst.
- `needs_password` ist `true` im AppState.

## Main Success Scenario

1. System zeigt einen modalen Dialog mit Titel „Password Required" und Hinweis „This container is encrypted."
2. Benutzer gibt das Passwort in das Eingabefeld ein.
3. Benutzer klickt „Decrypt" oder drückt Enter.
4. System versucht den Container zu entschlüsseln (-> SFDL-002).
5. Entschlüsselung erfolgreich: Dialog schliesst.
6. System lädt den Container und zeigt die Dateiliste (-> UI-001 Schritt 7).

## Alternative Flows

### A1: Falsches Passwort

**Trigger:** Entschlüsselung schlägt fehl mit `InvalidPassword` (Schritt 4)
**Flow:**

1. System zeigt roten Hinweis unter dem Eingabefeld: „Invalid password".
2. Dialog bleibt offen.
3. Use Case fährt mit Schritt 2 fort.

### A2: Entschlüsselungsfehler (kein Passwort-Fehler)

**Trigger:** Entschlüsselung schlägt aus anderem Grund fehl (Schritt 4)
**Flow:**

1. System zeigt Fehlermeldung: „Decryption failed: [Fehlerdetail]".
2. Dialog bleibt offen.
3. Use Case fährt mit Schritt 2 fort.

### A3: Abbrechen

**Trigger:** Benutzer klickt „Cancel" (statt Schritt 3)
**Flow:**

1. Dialog schliesst.
2. Container wird verworfen (`container` = None).
3. Hauptfenster kehrt zum leeren Zustand zurück.

### A4: Auto-Passwort-Treffer

**Trigger:** Beim Öffnen der SFDL-Datei passt ein Auto-Passwort (vor Schritt 1)
**Flow:**

1. Dieser Dialog wird komplett übersprungen.
2. Container wird direkt geladen (-> UI-001 Schritt 7).

## Postconditions

### Success Postconditions

- Container ist entschlüsselt und geladen.
- `needs_password` ist `false`.
- Dateiliste wird im Hauptfenster angezeigt.

### Failure Postconditions

- Bei A3: Kein Container geladen, Hauptfenster leer.
- Bei A1/A2: Dialog bleibt offen, Benutzer kann erneut versuchen.

## Business Rules

### BR-UI-002-001: Passwort-Eingabe

- Eingabefeld ist verdeckt (`type="password"`).
- Enter-Taste löst „Decrypt" aus.

### BR-UI-002-002: Modaler Dialog

- Dialog ist modal (Backdrop blockiert Interaktion mit dem Hauptfenster).
- Nur „Decrypt" und „Cancel" als Aktionen.

## Layout

- Modaler Dialog (Backdrop blockiert Hauptfenster)
- Titel: „Password Required"
- Hinweis: „This container is encrypted."
- Passwort-Eingabefeld (verdeckt, `type="password"`)
- Fehleranzeige (rot, unter dem Eingabefeld, nur bei Fehler)
- Buttons: „Cancel", „Decrypt" (Primary)
