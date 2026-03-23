# SFDL-002: Container entschlüsseln

**Use Case ID:** SFDL-002
**Requirements:** FR-02
**Primary Actor:** Benutzer
**Trigger:** Wird von SFDL-001 aufgerufen, wenn `Encrypted=true`.
**Preconditions:** Ein geparster, verschlüsselter Container liegt vor. Verschlüsselte Felder sind als Base64-Strings vorhanden.
**Postconditions (Erfolg):** Alle verschlüsselten Felder sind entschlüsselt und im Container-Objekt ersetzt.
**Postconditions (Fehlschlag):** Container bleibt verschlüsselt. Fehlermeldung mit Ursache.

---

## Main Success Scenario

1. System prüft, ob gespeicherte Passwörter vorhanden sind (Auto-Passwort-Liste aus Einstellungen).
2. **[Passwort-Liste vorhanden]** System probiert jedes Passwort der Liste:
   2a. System leitet den Schlüssel ab (→ BR-SFDL-003).
   2b. System entschlüsselt ein Testfeld (z.B. Host).
   2c. Wenn das Ergebnis valides UTF-8 ist und plausibel erscheint → Passwort gefunden, weiter bei Schritt 5.
3. **[Kein Passwort gefunden]** System signalisiert: „Passwort erforderlich."
4. Actor übergibt ein Passwort.
5. System leitet den AES-Schlüssel aus dem Passwort ab (→ BR-SFDL-003).
6. System entschlüsselt alle verschlüsselten Felder:
    - Host, Port, Username, Password
    - Dateinamen, Pfade, Beschreibung
7. System ersetzt die verschlüsselten Werte im Container-Objekt durch die Klartextwerte.
8. System gibt den entschlüsselten Container zurück.

## Alternative Paths

**2c-alt. Kein Passwort aus der Liste passt:**
2c-alt.1. System hat alle Passwörter der Liste durchprobiert, keines passt.
2c-alt.2. Weiter bei Schritt 3 (manuelles Passwort anfordern).

**5a. Falsches Passwort:**
5a.1. System entschlüsselt das Testfeld.
5a.2. Ergebnis ist kein valides UTF-8 oder offensichtlich kein Klartext.
5a.3. System meldet: „Falsches Passwort."
5a.4. Use Case kann mit neuem Passwort wiederholt werden (zurück zu Schritt 4).

**6a. Entschlüsselung teilweise fehlgeschlagen:**
6a.1. Einzelne Felder ergeben nach der Entschlüsselung keinen sinnvollen Wert.
6a.2. System meldet: „Container möglicherweise beschädigt: [betroffene Felder]."
6a.3. Use Case endet mit Fehler.

## Business Rules

**BR-SFDL-003: AES-Schlüsselableitung**

- Algorithmus: AES-128-CBC
- Schlüssel: MD5-Hash des Passworts (16 Bytes)
- IV: MD5-Hash des Passworts (identisch mit Schlüssel)
- Padding: PKCS7
- Encoding der verschlüsselten Felder: Base64

**BR-SFDL-004: Passwort-Validierung**

- Ein Passwort gilt als korrekt, wenn das entschlüsselte Testfeld (Host) valides UTF-8 ergibt und mindestens einen Punkt enthält (Hostname-Heuristik).
- Es gibt keinen expliziten Passwort-Hash im SFDL-Format — die Validierung ist immer heuristisch.

**BR-SFDL-005: Sicherheit**

- Das Passwort wird nicht persistiert (ausser wenn der Actor es explizit zur Passwort-Liste hinzufügt).
- Entschlüsselte Credentials existieren nur im Speicher (NFR-07).

## Input

- `container`: Verschlüsselter Container
- `password`: Klartext-Passwort (manuell oder aus Auto-Liste)

## Output (Erfolg)

- `Container` mit entschlüsselten Feldern
