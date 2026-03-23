# CLI-005: `rsfdl config`

**Interface Spec ID:** CLI-005
**Interface:** CLI (headless)
**Implementiert:** CFG-001

---

## Beschreibung

Zeigt die aktuelle Konfiguration an oder öffnet sie im Editor zur Bearbeitung.

## Syntax

```
rsfdl config show [--json]
rsfdl config edit
rsfdl config path
```

## Subcommands

| Subcommand | Beschreibung                                         |
|------------|------------------------------------------------------|
| `show`     | Zeigt die aktuelle Konfiguration an (Standard: TOML) |
| `edit`     | Öffnet die Konfigurationsdatei in `$EDITOR`          |
| `path`     | Gibt den Pfad zur Konfigurationsdatei aus            |

## Parameter

| Parameter | Pflicht | Beschreibung                                 |
|-----------|---------|----------------------------------------------|
| `--json`  | nein    | Ausgabe als JSON statt TOML (nur bei `show`) |

## Verhalten

### `config show`

1. System lädt die Konfigurationsdatei (→ CFG-001, Variante A).
2. Falls Datei nicht existiert: Standardwerte werden angezeigt.
3. System gibt die Einstellungen auf stdout aus.

### `config edit`

1. System prüft, ob `$EDITOR` gesetzt ist.
2. Falls nicht: Fehlermeldung „Umgebungsvariable EDITOR ist nicht gesetzt."
3. System erstellt die Konfigurationsdatei mit Standardwerten, falls nicht vorhanden.
4. System öffnet die Datei im Editor.
5. Nach Schliessen des Editors: System validiert die Datei (→ CFG-001, BR-CFG-003).
6. Bei Validierungsfehler: Warnung auf stderr.

### `config path`

1. System gibt den Pfad zur Konfigurationsdatei auf stdout aus.

## Exit-Codes

| Code | Bedeutung                             |
|------|---------------------------------------|
| 0    | Erfolg                                |
| 1    | Editor nicht gefunden / nicht gesetzt |
| 2    | Validierungsfehler nach Bearbeitung   |
