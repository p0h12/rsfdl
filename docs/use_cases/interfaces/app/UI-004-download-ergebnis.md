# UI-004: Download-Ergebnis

**Interface Spec ID:** UI-004
**Interface:** GUI (Dioxus Desktop)
**Implementiert:** POST-001, POST-002, POST-003

---

## Beschreibung

Zusammenfassung nach Abschluss einer DownloadSession. Zeigt Ergebnis, optionale Hash-Verifikation, Extraktion und Speed-Report.

## Layout

### Zusammenfassung

- Status-Banner: Grün (alle erfolgreich), Gelb (teilweise), Rot (alle fehlgeschlagen)
- Statistiken: Dateien OK / Failed / Skipped, Gesamtgrösse, Dauer, Ø Geschwindigkeit

### Hash-Verifikation (wenn Hashes verfügbar)

- Liste: Dateiname + Hash-Ergebnis (✅ Valid, ❌ Invalid, ⚪ No Hash)
- Zusammenfassung: „X von Y Dateien verifiziert"

### Extraktion (wenn aktiviert und Archive vorhanden)

- Liste: Archivname + Extraktions-Status
- Bei Fehler: Fehlermeldung pro Archiv

### Speed-Report

- Button „Speed-Report generieren"
- Textfeld mit gerendertem BBCode (read-only)
- Button „Kopieren" → BBCode in Zwischenablage

### Aktionen

- „Ordner öffnen" → Zielverzeichnis im Dateimanager öffnen
- „Neuen Container öffnen" → Zurück zum Hauptfenster
