# UI-002: Passwort-Dialog

**Interface Spec ID:** UI-002
**Interface:** GUI (Dioxus Desktop)
**Implementiert:** SFDL-002

---

## Beschreibung

Modaler Dialog, der erscheint, wenn ein verschlüsselter Container geöffnet wird und kein Passwort aus der Auto-Liste passt.

## Layout

- Titel: „Passwort erforderlich"
- Hinweis: „Die SFDL-Datei [Dateiname] ist verschlüsselt."
- Passwort-Eingabefeld (verdeckt, mit Toggle für Sichtbarkeit)
- Checkbox: „Passwort für zukünftige Dateien merken" (→ Passwort-Liste in CFG-001)
- Buttons: „Entschlüsseln" (Primary), „Abbrechen"

## Interaktionen

| Aktion            | Auslöst                                                         |
|-------------------|-----------------------------------------------------------------|
| „Entschlüsseln"   | SFDL-002 mit eingegebenem Passwort                              |
| Enter-Taste       | Wie „Entschlüsseln"                                             |
| „Abbrechen"       | Dialog schliesst, kein Container geladen                        |
| Falsches Passwort | Fehlermeldung im Dialog, Feld wird geleert, Dialog bleibt offen |

## Fehlerdarstellung

- Falsches Passwort: Roter Hinweis unter dem Eingabefeld: „Falsches Passwort. Bitte erneut versuchen."
- Beschädigter Container: Fehlermeldung, Dialog schliesst, Rückkehr zum Hauptfenster.

## Hinweise

- Bei Auto-Passwort-Treffer wird dieser Dialog übersprungen (SFDL-002 Schritt 2).
- Während der Auto-Passwort-Prüfung kann ein kurzer Lade-Indikator erscheinen.
