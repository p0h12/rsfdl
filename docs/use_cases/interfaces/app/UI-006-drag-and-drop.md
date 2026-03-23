# UI-006: Drag-and-Drop

**Interface Spec ID:** UI-006
**Interface:** GUI (Dioxus Desktop)
**Implementiert:** SFDL-001
**Requirements:** FR-13

---

## Beschreibung

SFDL-Dateien können per Drag-and-Drop auf das App-Fenster gezogen werden, als Alternative zum Dateidialog.

## Verhalten

| Phase      | UI-Feedback                                                                          |
|------------|--------------------------------------------------------------------------------------|
| Drag-Enter | Fenster zeigt visuellen Drop-Indikator (Rand leuchtet, Overlay „Datei hier ablegen") |
| Drag-Over  | Drop-Indikator bleibt aktiv                                                          |
| Drag-Leave | Drop-Indikator verschwindet                                                          |
| Drop       | System prüft die Dateiendung                                                         |

### Nach Drop

- **`.sfdl`-Datei:** → SFDL-001 mit dem Dateipfad aufrufen. Bisheriger Container wird ersetzt.
- **Andere Datei:** Fehlermeldung: „Nur .sfdl-Dateien werden unterstützt."
- **Mehrere Dateien:** Nur die erste `.sfdl`-Datei wird geöffnet, Rest ignoriert mit Hinweis.

## Hinweise

- Drop funktioniert in allen Fenstern-Zuständen (leer, Container geladen, Download läuft).
- Während eines laufenden Downloads: Bestätigungsdialog „Laufenden Download abbrechen und neue Datei öffnen?"
