# Use Case: Container entschluesseln

## Overview

**Use Case ID:** SFDL-002
**Use Case Name:** Container entschluesseln
**Primary Actor:** Benutzer
**Goal:** Einen verschluesselten SFDL-Container entschluesseln, sodass alle Klartextwerte verfuegbar sind.
**Requirements:** FR-02
**Status:** Stable

## Preconditions

- Ein geparster, verschluesselter Container liegt vor.
- Verschluesselte Felder sind als Base64-Strings vorhanden.

## Main Success Scenario

1. System prueft, ob gespeicherte Passwoerter vorhanden sind (Auto-Passwort-Liste aus Einstellungen).
2. **[Passwort-Liste vorhanden]** System probiert jedes Passwort der Liste:
   2a. System leitet den Schluessel ab (-> BR-SFDL-003).
   2b. System entschluesselt ein Testfeld (z.B. Host).
   2c. Wenn das Ergebnis valides UTF-8 ist und plausibel erscheint -> Passwort gefunden, weiter bei Schritt 5.
3. **[Kein Passwort gefunden]** System signalisiert: "Passwort erforderlich."
4. Actor uebergibt ein Passwort.
5. System leitet den AES-Schluessel aus dem Passwort ab (-> BR-SFDL-003).
6. System entschluesselt alle verschluesselten Felder:
    - Host, Port, Username, Password
    - Dateinamen, Pfade, Beschreibung
7. System ersetzt die verschluesselten Werte im Container-Objekt durch die Klartextwerte.
8. System gibt den entschluesselten Container zurueck.

## Alternative Flows

### A1: Kein Passwort aus der Liste passt

**Trigger:** System hat alle Passwoerter der Liste durchprobiert, keines passt (Schritt 2c)
**Flow:**

1. System hat alle Passwoerter der Liste durchprobiert, keines passt.
2. Weiter bei Schritt 3 (manuelles Passwort anfordern).

### A2: Falsches Passwort

**Trigger:** Actor gibt ein Passwort an, das nicht korrekt ist (Schritt 4)
**Flow:**

1. System entschluesselt das Testfeld.
2. Ergebnis ist kein valides UTF-8 oder offensichtlich kein Klartext.
3. System meldet: "Falsches Passwort."
4. Use Case kann mit neuem Passwort wiederholt werden (zurueck zu Schritt 4).

### A3: Entschluesselung teilweise fehlgeschlagen

**Trigger:** Einzelne Felder ergeben nach der Entschluesselung keinen sinnvollen Wert (Schritt 6)
**Flow:**

1. Einzelne Felder ergeben nach der Entschluesselung keinen sinnvollen Wert.
2. System meldet: "Container moeglicherweise beschaedigt: [betroffene Felder]."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Alle verschluesselten Felder sind entschluesselt und im Container-Objekt ersetzt.

### Failure Postconditions

- Container bleibt verschluesselt.
- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-SFDL-003: AES-Verschluesselung

- Algorithmus: AES-128-CBC
- Schluessel: MD5-Hash des Passworts (16 Bytes)
- IV: Zufaellig generiert (16 Bytes), dem Ciphertext vorangestellt
- Padding: PKCS7
- Encoding: Base64(IV || Ciphertext)
- Bei der Entschluesselung werden die ersten 16 Bytes als IV extrahiert, der Rest als Ciphertext

### BR-SFDL-004: Passwort-Validierung

- Ein Passwort gilt als korrekt, wenn das entschluesselte Testfeld (Host) valides UTF-8 ergibt und mindestens einen Punkt enthaelt (Hostname-Heuristik).
- Es gibt keinen expliziten Passwort-Hash im SFDL-Format -- die Validierung ist immer heuristisch.

### BR-SFDL-005: Sicherheit

- Das Passwort wird nicht persistiert (ausser wenn der Actor es explizit zur Passwort-Liste hinzufuegt).
- Entschluesselte Credentials existieren nur im Speicher (NFR-07).

## Input

- `container`: Verschluesselter Container
- `password`: Klartext-Passwort (manuell oder aus Auto-Liste)

## Output

- `Container` mit entschluesselten Feldern
