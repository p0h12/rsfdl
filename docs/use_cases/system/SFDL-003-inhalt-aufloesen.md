# Use Case: Container-Inhalt aufloesen

## Overview

**Use Case ID:** SFDL-003
**Use Case Name:** Container-Inhalt aufloesen
**Primary Actor:** System (automatisch nach SFDL-001)
**Goal:** Alle Pakete eines Containers in eine vollstaendige Liste von FileEntries mit Pfaden und Groessen aufloesen.
**Requirements:** FR-03
**Status:** Stable

## Preconditions

- Ein geparster und ggf. entschluesselter Container liegt vor.

## Main Success Scenario

1. System iteriert ueber alle Pakete des Containers.
2. Fuer jedes Paket prueft das System den Modus:
    - **FileList-Modus:** Dateien sind explizit im XML aufgelistet -> weiter bei Schritt 3.
    - **BulkFolder-Modus:** Nur Verzeichnispfade sind angegeben -> weiter bei Schritt 4.
3. **[FileList]** System uebernimmt die FileEntries direkt aus dem XML.
   -> Weiter bei Schritt 6.
4. **[BulkFolder]** System stellt eine FTP-Verbindung mit den ConnectionInfo-Daten her.
5. System fuehrt fuer jeden BulkFolder ein rekursives Verzeichnis-Listing durch:
   5a. System listet alle Dateien im Verzeichnis (und Unterverzeichnissen).
   5b. System erstellt fuer jede gefundene Datei einen FileEntry mit remote_path und size_bytes.
   5c. System markiert den BulkFolder als `resolved=true`.
6. -> **include** DL-002 (Ausschlussmuster anwenden) auf die FileEntry-Liste.
7. System berechnet die Gesamtgroesse pro Paket und fuer den gesamten Container.
8. System gibt den aufgeloesten Container mit vollstaendiger Dateiliste zurueck.

## Alternative Flows

### A1: FTP-Verbindung fehlgeschlagen

**Trigger:** System kann keine Verbindung zum FTP-Server herstellen (Schritt 4)
**Flow:**

1. System kann keine Verbindung zum FTP-Server herstellen.
2. System markiert betroffene BulkFolders als `resolved=false`.
3. System meldet: "Verzeichnis konnte nicht aufgeloest werden: [Fehler]. Dateien koennen beim Download-Start erneut aufgeloest werden."
4. Container wird mit teilweise aufgeloesten Paketen zurueckgegeben.

### A2: Leeres Verzeichnis

**Trigger:** FTP-Listing gibt keine Dateien zurueck (Schritt 5a)
**Flow:**

1. FTP-Listing gibt keine Dateien zurueck.
2. BulkFolder wird als aufgeloest markiert mit 0 FileEntries.
3. System meldet: "Verzeichnis [Pfad] ist leer."

### A3: Zugriff verweigert

**Trigger:** FTP-Server verweigert den Zugriff auf ein Verzeichnis (Schritt 5)
**Flow:**

1. FTP-Server verweigert den Zugriff auf ein Verzeichnis.
2. BulkFolder wird als `resolved=false` markiert mit Fehlermeldung.
3. Verarbeitung weiterer BulkFolders wird fortgesetzt.

## Postconditions

### Success Postconditions

- Alle Pakete enthalten eine vollstaendige Liste von FileEntries mit Pfaden und (wenn verfuegbar) Groessen.

### Failure Postconditions

- Container mit teilweise aufgeloesten Paketen liegt vor.
- Nicht aufgeloeste BulkFolders sind markiert.

## Business Rules

### BR-SFDL-006: BulkFolder-Aufloesung

- BulkFolders werden rekursiv aufgeloest (inklusive Unterverzeichnisse).
- Symbolische Links werden nicht verfolgt.
- Die FTP-Verbindung fuer die Aufloesung wird nach Abschluss geschlossen.

### BR-SFDL-007: Groessen-Berechnung

- Wenn Dateigroessen im XML vorhanden sind, werden diese verwendet.
- Wenn nicht (typisch bei BulkFolder), wird die Groesse aus dem FTP-Listing genommen.
- Ist keine Groesse verfuegbar, wird `size_bytes=None` gesetzt. Die Gesamtgroesse wird als "mindestens X" angezeigt.

## Input

- `container`: Geparster Container mit Paketen

## Output

- `Container` mit aufgeloesten FileEntries pro Paket
- `total_files: int` -- Gesamtanzahl Dateien
- `total_bytes: Option<int>` -- Gesamtgroesse (None wenn nicht alle Groessen bekannt)
