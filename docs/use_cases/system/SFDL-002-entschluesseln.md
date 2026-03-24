# Use Case: Container entschlüsseln

## Overview

**Use Case ID:** SFDL-002
**Use Case Name:** Container entschlüsseln
**Primary Actor:** Benutzer
**Goal:** Einen verschlüsselten SFDL-Container entschlüsseln, sodass alle Klartextwerte verfügbar sind.
**Requirements:** FR-02
**Status:** Stable

## Preconditions

- Ein geparster, verschlüsselter Container liegt vor.
- Verschlüsselte Felder sind als Base64-Strings vorhanden.

## Main Success Scenario

1. System prüft, ob gespeicherte Passwörter vorhanden sind (Auto-Passwort-Liste aus Einstellungen).
2. **[Passwort-Liste vorhanden]** System probiert jedes Passwort der Liste:
   2a. System leitet den Schlüssel ab (-> BR-SFDL-003).
   2b. System entschlüsselt ein Testfeld (z.B. Host).
   2c. Wenn das Ergebnis valides UTF-8 ist und plausibel erscheint -> Passwort gefunden, weiter bei Schritt 5.
3. **[Kein Passwort gefunden]** System signalisiert: "Passwort erforderlich."
4. Actor übergibt ein Passwort.
5. System leitet den AES-Schlüssel aus dem Passwort ab (-> BR-SFDL-003).
6. System entschlüsselt alle verschlüsselten Felder:
    - Host, Port, Username, Password
    - Dateinamen, Pfade, Beschreibung
7. System ersetzt die verschlüsselten Werte im Container-Objekt durch die Klartextwerte.
8. System gibt den entschlüsselten Container zurück.

## Alternative Flows

### A1: Kein Passwort aus der Liste passt

**Trigger:** System hat alle Passwörter der Liste durchprobiert, keines passt (Schritt 2c)
**Flow:**

1. System hat alle Passwörter der Liste durchprobiert, keines passt.
2. Weiter bei Schritt 3 (manuelles Passwort anfordern).

### A2: Falsches Passwort

**Trigger:** Actor gibt ein Passwort an, das nicht korrekt ist (Schritt 4)
**Flow:**

1. System entschlüsselt das Testfeld.
2. Ergebnis ist kein valides UTF-8 oder offensichtlich kein Klartext.
3. System meldet: "Falsches Passwort."
4. Use Case kann mit neuem Passwort wiederholt werden (zurück zu Schritt 4).

### A3: Entschlüsselung teilweise fehlgeschlagen

**Trigger:** Einzelne Felder ergeben nach der Entschlüsselung keinen sinnvollen Wert (Schritt 6)
**Flow:**

1. Einzelne Felder ergeben nach der Entschlüsselung keinen sinnvollen Wert.
2. System meldet: "Container möglicherweise beschädigt: [betroffene Felder]."
3. Use Case endet mit Fehler.

## Postconditions

### Success Postconditions

- Alle verschlüsselten Felder sind entschlüsselt und im Container-Objekt ersetzt.

### Failure Postconditions

- Container bleibt verschlüsselt.
- Fehlermeldung mit Ursache liegt vor.

## Business Rules

### BR-SFDL-003: AES-Verschlüsselung

- Algorithmus: AES-128-CBC
- Schlüssel: MD5-Hash des Passworts (16 Bytes)
- IV: Zufällig generiert (16 Bytes), dem Ciphertext vorangestellt
- Padding: PKCS7
- Encoding: Base64(IV || Ciphertext)
- Bei der Entschlüsselung werden die ersten 16 Bytes als IV extrahiert, der Rest als Ciphertext

### BR-SFDL-004: Passwort-Validierung

- Ein Passwort gilt als korrekt, wenn das entschlüsselte Testfeld (Host) valides UTF-8 ergibt und mindestens einen Punkt enthält (Hostname-Heuristik).
- Es gibt keinen expliziten Passwort-Hash im SFDL-Format -- die Validierung ist immer heuristisch.

### BR-SFDL-005: Sicherheit

- Das Passwort wird nicht persistiert (ausser wenn der Actor es explizit zur Passwort-Liste hinzufügt).
- Entschlüsselte Credentials existieren nur im Speicher (NFR-07).

## Input

- `container`: Verschlüsselter Container
- `password`: Klartext-Passwort (manuell oder aus Auto-Liste)

## Output

- `Container` mit entschlüsselten Feldern
