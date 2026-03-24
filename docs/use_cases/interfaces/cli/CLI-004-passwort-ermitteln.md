# Use Case: Passwort ermitteln und entschlüsseln

## Overview

**Use Case ID:** CLI-004
**Use Case Name:** Passwort ermitteln und entschlüsseln
**Primary Actor:** Benutzer
**Goal:** Das Passwort für einen verschlüsselten Container ermitteln und den Container entschlüsseln.
**Implements:** SFDL-002
**Status:** Stable

## Preconditions

- Ein verschlüsselter SFDL-Container wurde geparst (SFDL-001).
- Auto-Passwort-Liste ist aus Einstellungen geladen (CFG-001).

## Main Success Scenario

1. System probiert jedes Passwort der Auto-Liste (-> SFDL-002).
2. Ein Passwort passt.
3. System entschlüsselt den Container.
4. System gibt den entschlüsselten Container zurück.

## Alternative Flows

### A1: --password Flag angegeben

**Trigger:** Benutzer hat `--password <pw>` angegeben (vor Schritt 1)
**Flow:**

1. System verwendet das angegebene Passwort.
2. System entschlüsselt den Container (-> SFDL-002).
3. Use Case endet erfolgreich.

### A2: Kein Auto-Passwort passt — interaktiver Prompt

**Trigger:** Kein Passwort aus der Liste passt, stderr ist ein Terminal (Schritt 1)
**Flow:**

1. System zeigt Passwort-Prompt (keine Echo-Ausgabe).
2. Benutzer gibt Passwort ein.
3. System entschlüsselt den Container (-> SFDL-002).
4. Use Case endet erfolgreich.

### A3: Kein Passwort verfügbar (nicht-interaktiv)

**Trigger:** Kein Passwort passt, kein Terminal (Schritt 1)
**Flow:**

1. System meldet Fehler auf stderr.
2. Exit-Code 3.

### A4: Falsches Passwort

**Trigger:** Angegebenes oder eingegebenes Passwort ist falsch (A1 Schritt 2 / A2 Schritt 3)
**Flow:**

1. System meldet "Invalid password" auf stderr.
2. Exit-Code 4.

## Postconditions

### Success Postconditions

- Container ist entschlüsselt.

### Failure Postconditions

- Container bleibt verschlüsselt.
- Fehlermeldung auf stderr.

## Business Rules

### BR-CLI-018: Passwort-Priorität

- Priorität: `--password` Flag > Auto-Passwort-Liste (CFG) > Interaktiver Prompt > Fehler.
- Interaktiver Prompt nur wenn stderr ein Terminal ist.
- Passwort-Eingabe ohne Echo (rpassword).

Weitere Regeln: -> CLI-CC (Cross-Cutting): Exit-Codes (BR-CLI-007).
