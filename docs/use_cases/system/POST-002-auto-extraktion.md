# POST-002: Archive extrahieren

**Use Case ID:** POST-002
**Requirements:** FR-16
**Primary Actor:** System (automatisch nach Download)
**Trigger:** Alle DownloadTasks eines Pakets haben Status `Completed` und `auto_extract=true` in den Einstellungen.
**Preconditions:** Alle Dateien eines Pakets sind vollständig heruntergeladen. Auto-Extraktion ist aktiviert.
**Postconditions (Erfolg):** Archive sind entpackt. Optional: Archiv-Dateien gelöscht.
**Postconditions (Fehlschlag):** Fehlermeldung. Archiv-Dateien bleiben erhalten. Download-Erfolg wird nicht beeinflusst.

---

## Main Success Scenario

1. System scannt die heruntergeladenen Dateien des Pakets nach Archiven (→ BR-POST-003).
2. Für jedes erkannte Archiv erstellt das System eine ExtractionTask.
3. System extrahiert das Archiv in das Zielverzeichnis:
    - **[ZIP]** System entpackt mit Standard-ZIP-Bibliothek.
    - **[RAR]** System entpackt mit `unrar`-Bibliothek.
    - **[Multi-Part RAR]** System erkennt den ersten Teil und entpackt das gesamte Archiv.
4. System emittiert Fortschritts-Events während der Extraktion.
5. ExtractionTask-Status → `Completed`.
6. **[delete_archives_after_extract=true]** System löscht die Archiv-Dateien.

## Alternative Paths

**1a. Keine Archive gefunden:**
1a.1. Keine Dateien passen auf Archiv-Muster.
1a.2. Use Case endet ohne Aktion.

**3a. Extraktion fehlgeschlagen:**
3a.1. Archiv ist beschädigt oder passwortgeschützt.
3a.2. ExtractionTask-Status → `Failed` mit Fehlermeldung.
3a.3. Archiv-Dateien bleiben erhalten.
3a.4. System meldet: „Extraktion fehlgeschlagen für [Archiv]: [Fehler]."
3a.5. **Der Download-Gesamterfolg wird nicht beeinflusst.**

**3b. Multi-Part unvollständig:**
3b.1. Nicht alle Teile eines Multi-Part-RAR sind vorhanden.
3b.2. System meldet: „Archiv unvollständig: Teil X von Y fehlt."
3b.3. ExtractionTask-Status → `Failed`.

## Business Rules

**BR-POST-003: Archiv-Erkennung**

- ZIP: Dateiendung `.zip`
- RAR: Dateiendung `.rar`
- Multi-Part RAR: `.part01.rar`, `.part001.rar`, `.r01`, `.r001`
- Bei Multi-Part: Nur der erste Teil löst die Extraktion aus.
- Erkennung ist case-insensitiv.

**BR-POST-004: Extraktions-Ziel**

- Archive werden in das gleiche Verzeichnis entpackt, in dem sie liegen.
- Enthält das Archiv einen Root-Ordner, wird dieser beibehalten.
- Bei Namenskonflikten: bestehende Dateien werden nicht überschrieben.

**BR-POST-005: Feature-Toggle**

- Auto-Extraktion ist in den Einstellungen deaktivierbar (Standard: aus).
- `delete_archives_after_extract` ist separat schaltbar (Standard: aus).

## Input

- `package_files[]`: Heruntergeladene Dateien eines Pakets
- `target_directory: String`
- `delete_after: bool`

## Output

- `ExtractionTask[]` mit Status pro Archiv
