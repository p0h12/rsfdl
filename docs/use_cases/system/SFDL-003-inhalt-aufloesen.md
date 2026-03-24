# Use Case: Container-Inhalt auflösen

## Overview

**Use Case ID:** SFDL-003
**Use Case Name:** Container-Inhalt auflösen
**Primary Actor:** System (automatisch nach SFDL-001)
**Goal:** Alle Pakete eines Containers in eine vollständige Liste von FileEntries mit Pfaden und Grössen auflösen.
**Requirements:** FR-03
**Status:** Stable

## Preconditions

- Ein geparster und ggf. entschlüsselter Container liegt vor.

## Main Success Scenario

1. System iteriert über alle Pakete des Containers.
2. Für jedes Paket prüft das System den Modus:
    - **FileList-Modus:** Dateien sind explizit im XML aufgelistet -> weiter bei Schritt 3.
    - **BulkFolder-Modus:** Nur Verzeichnispfade sind angegeben -> weiter bei Schritt 4.
3. **[FileList]** System übernimmt die FileEntries direkt aus dem XML.
   -> Weiter bei Schritt 6.
4. **[BulkFolder]** System stellt eine FTP-Verbindung mit den ConnectionInfo-Daten her.
5. System führt für jeden BulkFolder ein rekursives Verzeichnis-Listing durch:
   5a. System listet alle Dateien im Verzeichnis (und Unterverzeichnissen).
   5b. System erstellt für jede gefundene Datei einen FileEntry mit remote_path und size_bytes.
   5c. System markiert den BulkFolder als `resolved=true`.
6. -> **include** DL-002 (Ausschlussmuster anwenden) auf die FileEntry-Liste.
7. System berechnet die Gesamtgrösse pro Paket und für den gesamten Container.
8. System gibt den aufgelösten Container mit vollständiger Dateiliste zurück.

## Alternative Flows

### A1: FTP-Verbindung fehlgeschlagen

**Trigger:** System kann keine Verbindung zum FTP-Server herstellen (Schritt 4)
**Flow:**

1. System kann keine Verbindung zum FTP-Server herstellen.
2. System markiert betroffene BulkFolders als `resolved=false`.
3. System meldet: "Verzeichnis konnte nicht aufgelöst werden: [Fehler]. Dateien können beim Download-Start erneut aufgelöst werden."
4. Container wird mit teilweise aufgelösten Paketen zurückgegeben.

### A2: Leeres Verzeichnis

**Trigger:** FTP-Listing gibt keine Dateien zurück (Schritt 5a)
**Flow:**

1. FTP-Listing gibt keine Dateien zurück.
2. BulkFolder wird als aufgelöst markiert mit 0 FileEntries.
3. System meldet: "Verzeichnis [Pfad] ist leer."

### A3: Zugriff verweigert

**Trigger:** FTP-Server verweigert den Zugriff auf ein Verzeichnis (Schritt 5)
**Flow:**

1. FTP-Server verweigert den Zugriff auf ein Verzeichnis.
2. BulkFolder wird als `resolved=false` markiert mit Fehlermeldung.
3. Verarbeitung weiterer BulkFolders wird fortgesetzt.

## Postconditions

### Success Postconditions

- Alle Pakete enthalten eine vollständige Liste von FileEntries mit Pfaden und (wenn verfügbar) Grössen.

### Failure Postconditions

- Container mit teilweise aufgelösten Paketen liegt vor.
- Nicht aufgelöste BulkFolders sind markiert.

## Business Rules

### BR-SFDL-006: BulkFolder-Auflösung

- BulkFolders werden rekursiv aufgelöst (inklusive Unterverzeichnisse).
- Symbolische Links werden nicht verfolgt.
- Die FTP-Verbindung für die Auflösung wird nach Abschluss geschlossen.

### BR-SFDL-007: Grössen-Berechnung

- Wenn Dateigrössen im XML vorhanden sind, werden diese verwendet.
- Wenn nicht (typisch bei BulkFolder), wird die Grösse aus dem FTP-Listing genommen.
- Ist keine Grösse verfügbar, wird `size_bytes=None` gesetzt. Die Gesamtgrösse wird als "mindestens X" angezeigt.

## Input

- `container`: Geparster Container mit Paketen

## Output

- `Container` mit aufgelösten FileEntries pro Paket
- `total_files: int` -- Gesamtanzahl Dateien
- `total_bytes: Option<int>` -- Gesamtgrösse (None wenn nicht alle Grössen bekannt)
