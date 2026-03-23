# UI-003: Download-Fortschritt

**Interface Spec ID:** UI-003
**Interface:** GUI (Dioxus Desktop)
**Implementiert:** DL-004, DL-005, DL-006, DL-007, DL-008

---

## Beschreibung

Panel, das während einer aktiven DownloadSession den Fortschritt anzeigt. Wird unterhalb oder anstelle der Dateiliste eingeblendet.

## Layout

### Globaler Fortschritt

- Fortschrittsbalken: Gesamtbytes heruntergeladen / total
- Text: „X von Y Dateien (Z MB / W MB) — Ø V MB/s"
- Geschätzte Restzeit: „ca. MM:SS verbleibend"
- Aktive Threads: „3 von 3 aktiv"

### Pro-Datei-Fortschritt

- Liste der aktiven und abgeschlossenen Tasks:
    - Icon: Status (⏳ Pending, ⬇ Active, ✅ Completed, ❌ Failed, ⏸ Cancelled, ⏭ Skipped)
    - Dateiname
    - Fortschrittsbalken (nur bei Active)
    - Geschwindigkeit (nur bei Active)
    - Bei Failed: Fehlertyp + Retry-Info („Retry 2/3 in 10s…")

### Aktions-Buttons

- „Alle abbrechen" (→ DL-006 Variante B)
- Pro Datei: „Abbrechen" bei Active-Tasks (→ DL-006 Variante A)

## Zustandsübergänge

| Event                   | UI-Reaktion                                        |
|-------------------------|----------------------------------------------------|
| Progress-Event (Core)   | Fortschrittsbalken + Geschwindigkeit aktualisieren |
| Task → Completed        | Zeile aktualisieren, Icon wechseln                 |
| Task → Failed           | Rote Markierung, Fehlertext anzeigen               |
| Task → Retry            | Countdown-Anzeige: „Retry in Xs…"                  |
| Session → Completed     | Zusammenfassung anzeigen (→ UI-004)                |
| Session → Cancelled     | „Download abgebrochen" Meldung                     |
| Bandbreiten-Limit aktiv | Anzeige: „🔽 Limit: X MB/s" im Header              |

## Performance

- UI-Updates maximal 10× pro Sekunde (Debouncing der Progress-Events).
- Geschwindigkeit wird als gleitender Durchschnitt der letzten 3 Sekunden berechnet.
