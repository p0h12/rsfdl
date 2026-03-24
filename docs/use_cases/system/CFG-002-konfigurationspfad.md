# Use Case: Konfigurationspfad ermitteln

## Overview

**Use Case ID:** CFG-002
**Use Case Name:** Konfigurationspfad ermitteln
**Primary Actor:** System (automatisch)
**Goal:** Den korrekten Pfad zur Konfigurationsdatei bestimmen.
**Status:** Stable

## Preconditions

- Keine.

## Main Success Scenario

1. System prueft die Umgebungsvariable `RSFDL_CONFIG`.
2. Variable ist nicht gesetzt.
3. System ermittelt den Plattform-Standardpfad (-> BR-CFG-005).
4. System gibt den Pfad zurueck.

## Alternative Flows

### A1: RSFDL_CONFIG gesetzt

**Trigger:** Umgebungsvariable ist gesetzt (Schritt 1)
**Flow:**

1. System verwendet den Wert der Variable als Pfad.
2. Use Case endet.

## Postconditions

### Success Postconditions

- Ein Dateipfad zur Konfigurationsdatei liegt vor.

### Failure Postconditions

- Keine — es gibt immer einen Fallback (Plattform-Standard).

## Business Rules

### BR-CFG-005: Plattform-Standardpfade

- Linux: `~/.config/rsfdl/settings.toml`
- macOS: `~/Library/Application Support/rsfdl/settings.toml`
- Windows: `%APPDATA%\rsfdl\settings.toml`

### BR-CFG-006: Umgebungsvariable

- `RSFDL_CONFIG` ueberschreibt den Plattform-Standardpfad.
- Wird von CLI und Desktop-App respektiert.
- Gedacht fuer User die ihre Config permanent an einem anderen Ort haben wollen.
