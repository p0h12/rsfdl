# UI-001: Hauptfenster

**Interface Spec ID:** UI-001
**Interface:** GUI (Dioxus Desktop)
**Implementiert:** SFDL-001, SFDL-003, DL-001

---

## Beschreibung

Das Hauptfenster ist der zentrale Einstiegspunkt der Desktop-App. Es bildet den Workflow ab:
Datei öffnen → Container anzeigen → Dateien auswählen → Download starten.

## Layout

### Zustand: Kein Container geladen

- Header mit App-Name und Version
- Zentrale Drop-Zone / Fläche mit Aufforderung: „SFDL-Datei öffnen oder hierher ziehen"
- Button „Datei öffnen…" → OS-Dateidialog (Filter: `*.sfdl`)
- Menüleiste / Toolbar mit Zugang zu Einstellungen (→ UI-005)

### Zustand: Container geladen

- **Info-Banner**: Container-Beschreibung, Uploader, Host:Port, Protokoll
- **Dateiliste** (→ DL-001):
    - Baumstruktur: Pakete → Dateien
    - Checkbox pro Datei und pro Paket
    - Spalten: Dateiname, Grösse, Status (bei Ausschluss: durchgestrichen/ausgegraut)
    - Footer: „X von Y Dateien ausgewählt (Z MB)"
- **Aktions-Buttons**:
    - „Download starten" (aktiv wenn Selektion > 0)
    - „Alle auswählen" / „Alle abwählen"
    - „Neuen Container öffnen"

### Zustand: Download läuft

- Dateiliste bleibt sichtbar, Status pro Datei aktualisiert sich
- Progress-Panel wird eingeblendet (→ UI-003)
- „Download starten" wird zu „Download abbrechen" (→ DL-006)

## Interaktionen

| Aktion                     | Auslöst                  | Use Case         |
|----------------------------|--------------------------|------------------|
| „Datei öffnen…" Button     | OS-Dateidialog           | SFDL-001         |
| Datei in Drop-Zone ziehen  | Drag-and-Drop            | SFDL-001, UI-006 |
| Checkbox Datei an/abwählen | Selektion aktualisieren  | DL-001           |
| Checkbox Paket an/abwählen | Alle Dateien im Paket    | DL-001           |
| „Download starten"         | Download-Session starten | DL-004           |
| „Download abbrechen"       | Globaler Abbruch         | DL-006           |
| Zahnrad-Icon / Menü        | Einstellungen öffnen     | UI-005           |

## Fehlerdarstellung

- Fehlermeldungen beim Öffnen: Inline-Banner über der Drop-Zone (rot, dismissable)
- Fehlgeschlagene BulkFolder-Auflösung: Warnung im Info-Banner, betroffene Ordner markiert
