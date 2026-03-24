# Use Case: Container verschlüsseln

## Overview

**Use Case ID:** CR-004
**Use Case Name:** Container verschlüsseln
**Primary Actor:** System
**Goal:** Alle sensitiven Felder eines Containers mit AES-128-CBC verschlüsseln.
**Requirements:** FR-23
**Status:** Stable

## Preconditions

- Ein vollständig aufgebauter Container mit Klartextwerten liegt vor.
- Ein Passwort ist verfügbar.
- Wird von CR-001 aufgerufen, wenn ein Verschlüsselungspasswort angegeben wurde.

## Main Success Scenario

1. System leitet den AES-Schlüssel aus dem Passwort ab (-> BR-SFDL-003).
2. Für jedes zu verschlüsselnde Feld:
   2a. System generiert einen kryptographisch zufälligen IV.
   2b. System verschlüsselt den Klartextwert mit AES-128-CBC.
   2c. System kodiert das Ergebnis als Base64.
3. System ersetzt die Klartextwerte im Container durch die verschlüsselten Werte.
4. System setzt `Encrypted=true` im Container.
5. System gibt den verschlüsselten Container an CR-001 zurück.

## Alternative Flows

### A1: Leeres Passwort

**Trigger:** Actor hat ein leeres Passwort angegeben (Schritt 1)
**Flow:**

1. Actor hat ein leeres Passwort angegeben.
2. System meldet: "Passwort darf nicht leer sein."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Alle sensitiven Felder sind AES-128-CBC-verschlüsselt.
- Container-Flag `Encrypted=true` ist gesetzt.

### Failure Postconditions

- Container bleibt unverschlüsselt.
- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-CR-007: Verschlüsselte Felder

Folgende Felder werden verschlüsselt:

- Host, Username, Password
- Dateinamen, Pfade, Beschreibung

Folgende Felder bleiben unverschlüsselt:

- Port, FileSize, HashType

### BR-CR-008: Schlüsselableitung

- Identischer Algorithmus wie bei der Entschlüsselung (BR-SFDL-003):
    - AES-128-CBC, MD5-Key-Derivation, PKCS7-Padding
- Round-Trip mit SFDL-002: Verschlüsselter Container muss mit demselben Passwort entschlüsselt werden können.

### BR-CR-009: IV-Generierung

- Jedes Feld erhält einen eigenen kryptographisch zufälligen IV.
- Der IV wird nicht separat gespeichert -- er ist als Präfix im verschlüsselten Wert enthalten (SFDL-Konvention).

## Input

- `container`: Unverschlüsselter Container
- `password`: Klartext-Passwort

## Output

- `Container` mit verschlüsselten Feldern und `Encrypted=true`
