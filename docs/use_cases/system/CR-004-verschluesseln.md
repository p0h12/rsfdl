# CR-004: Container verschlüsseln

**Use Case ID:** CR-004
**Requirements:** FR-23
**Primary Actor:** System
**Trigger:** Wird von CR-001 aufgerufen, wenn ein Verschlüsselungspasswort angegeben wurde.
**Preconditions:** Ein vollständig aufgebauter Container mit Klartextwerten liegt vor. Ein Passwort ist verfügbar.
**Postconditions (Erfolg):** Alle sensitiven Felder sind AES-128-CBC-verschlüsselt. Container-Flag `Encrypted=true`.
**Postconditions (Fehlschlag):** Container bleibt unverschlüsselt. Fehlermeldung mit Ursache.

---

## Main Success Scenario

1. System leitet den AES-Schlüssel aus dem Passwort ab (→ BR-SFDL-003).
2. Für jedes zu verschlüsselnde Feld:
   2a. System generiert einen kryptographisch zufälligen IV.
   2b. System verschlüsselt den Klartextwert mit AES-128-CBC.
   2c. System kodiert das Ergebnis als Base64.
3. System ersetzt die Klartextwerte im Container durch die verschlüsselten Werte.
4. System setzt `Encrypted=true` im Container.
5. System gibt den verschlüsselten Container an CR-001 zurück.

## Alternative Paths

**1a. Leeres Passwort:**
1a.1. Actor hat ein leeres Passwort angegeben.
1a.2. System meldet: „Passwort darf nicht leer sein."
1a.3. Use Case endet mit Fehler.

## Business Rules

**BR-CR-007: Verschlüsselte Felder**

Folgende Felder werden verschlüsselt:

- Host, Username, Password
- Dateinamen, Pfade, Beschreibung

Folgende Felder bleiben unverschlüsselt:

- Port, FileSize, HashType

**BR-CR-008: Schlüsselableitung**

- Identischer Algorithmus wie bei der Entschlüsselung (BR-SFDL-003):
    - AES-128-CBC, MD5-Key-Derivation, PKCS7-Padding
- Round-Trip mit SFDL-002: Verschlüsselter Container muss mit demselben Passwort entschlüsselbar sein.

**BR-CR-009: IV-Generierung**

- Jedes Feld erhält einen eigenen kryptographisch zufälligen IV.
- Der IV wird nicht separat gespeichert — er ist als Präfix im verschlüsselten Wert enthalten (SFDL-Konvention).

## Input

- `container`: Unverschlüsselter Container
- `password`: Klartext-Passwort

## Output (Erfolg)

- `Container` mit verschlüsselten Feldern und `Encrypted=true`
