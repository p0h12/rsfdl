# UI-005: Einstellungen-Dialog

**Interface Spec ID:** UI-005
**Interface:** GUI (Dioxus Desktop)
**Implementiert:** CFG-001

---

## Beschreibung

Modaler Dialog oder eigene View für die App-Konfiguration. Gruppiert nach Kategorien.

## Layout

### Allgemein

- Download-Verzeichnis: Textfeld + „Durchsuchen…"-Button
- Max. parallele Downloads: Spinner (1–20)
- Max. Geschwindigkeit: Eingabefeld (KB/s), 0 = unbegrenzt

### Download-Verhalten

- Max. Retries: Spinner (0–50)
- Retry-Wartezeit: Eingabefeld (Sekunden)
- Speicherplatz strikt prüfen: Checkbox

### Nachbearbeitung

- Auto-Extraktion: Checkbox (Standard: aus)
- Archive nach Extraktion löschen: Checkbox (Standard: aus, nur aktiv wenn Auto-Extraktion an)

### Ausschlussmuster

- Liste der aktuellen Muster mit „Entfernen"-Button pro Zeile
- Eingabefeld + „Hinzufügen"-Button für neue Muster
- Button „Standards wiederherstellen"

### Passwörter

- Liste gespeicherter Passwörter (verdeckt angezeigt)
- „Hinzufügen" / „Entfernen"-Buttons
- Hinweis: „Passwörter werden verschlüsselt gespeichert."

### Speed-Report

- Benutzername: Textfeld
- BBCode-Template: Mehrzeiliges Textfeld
- Button „Standard-Template wiederherstellen"
- Vorschau des gerenderten Templates

### Aktionen

- „Speichern" → CFG-001 Variante B
- „Abbrechen" → Änderungen verwerfen
- „Zurücksetzen" → CFG-001 Variante C (mit Bestätigung)

## Validierung

- Ungültige Werte werden rot markiert mit Hinweis.
- „Speichern" ist deaktiviert, solange ungültige Werte vorhanden sind.
