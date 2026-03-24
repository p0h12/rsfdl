# Use Case: Container verschluesseln

## Overview

**Use Case ID:** CR-004
**Use Case Name:** Container verschluesseln
**Primary Actor:** System
**Goal:** Alle sensitiven Felder eines Containers mit AES-128-CBC verschluesseln.
**Requirements:** FR-23
**Status:** Stable

## Preconditions

- Ein vollstaendig aufgebauter Container mit Klartextwerten liegt vor.
- Ein Passwort ist verfuegbar.
- Wird von CR-001 aufgerufen, wenn ein Verschluesselungspasswort angegeben wurde.

## Main Success Scenario

1. System leitet den AES-Schluessel aus dem Passwort ab (-> BR-SFDL-003).
2. Fuer jedes zu verschluesselnde Feld:
   2a. System generiert einen kryptographisch zufaelligen IV.
   2b. System verschluesselt den Klartextwert mit AES-128-CBC.
   2c. System kodiert das Ergebnis als Base64.
3. System ersetzt die Klartextwerte im Container durch die verschluesselten Werte.
4. System setzt `Encrypted=true` im Container.
5. System gibt den verschluesselten Container an CR-001 zurueck.

## Alternative Flows

### A1: Leeres Passwort

**Trigger:** Actor hat ein leeres Passwort angegeben (Schritt 1)
**Flow:**

1. Actor hat ein leeres Passwort angegeben.
2. System meldet: "Passwort darf nicht leer sein."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Alle sensitiven Felder sind AES-128-CBC-verschluesselt.
- Container-Flag `Encrypted=true` ist gesetzt.

### Failure Postconditions

- Container bleibt unverschluesselt.
- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-CR-007: Verschluesselte Felder

Folgende Felder werden verschluesselt:

- Host, Username, Password
- Dateinamen, Pfade, Beschreibung

Folgende Felder bleiben unverschluesselt:

- Port, FileSize, HashType

### BR-CR-008: Schluesselableitung

- Identischer Algorithmus wie bei der Entschluesselung (BR-SFDL-003):
    - AES-128-CBC, MD5-Key-Derivation, PKCS7-Padding
- Round-Trip mit SFDL-002: Verschluesselter Container muss mit demselben Passwort entschluesselt werden koennen.

### BR-CR-009: IV-Generierung

- Jedes Feld erhaelt einen eigenen kryptographisch zufaelligen IV.
- Der IV wird nicht separat gespeichert -- er ist als Praefix im verschluesselten Wert enthalten (SFDL-Konvention).

## Input

- `container`: Unverschluesselter Container
- `password`: Klartext-Passwort

## Output

- `Container` mit verschluesselten Feldern und `Encrypted=true`
