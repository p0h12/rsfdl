# Vision: rsfdl

## Problem

SFDL.NET ist der De-facto-Standard-Downloader für SFDL-Container-Dateien — aber:

- **Nur Windows**: Kein Support für macOS oder Linux
- **Veraltete Technologie**: VB.NET / WPF, .NET Framework 4.5/4.6
- **Keine CLI**: Kein headless Betrieb, keine Automatisierung möglich
- **Keine aktive Weiterentwicklung**: Projekt ist weitgehend stagniert

Die alternativen Implementierungen (pySFDLSauger, goSFDLSauger, SFDLSaugerPro) lösen Teilprobleme, sind aber entweder nur CLI, nur Web-UI, oder ebenfalls nicht aktiv gepflegt.

## Vision

**rsfdl** ist ein moderner, plattformübergreifender SFDL-Downloader, der als **Desktop-App und CLI** verfügbar ist. Er ermöglicht es Benutzern, SFDL-Container-Dateien zu öffnen, die enthaltenen Dateien zu browsen und über FTP herunterzuladen — auf macOS, Linux und Windows.

## Zielgruppe

- Benutzer von sfdl.net, die macOS oder Linux verwenden
- Benutzer, die Downloads automatisieren wollen (CLI, Scripting, Cronjobs)
- Power-User, die eine schnelle, ressourcenschonende Alternative zu SFDL.NET suchen

## Kernwerte

| Wert                      | Bedeutung                                            |
|---------------------------|------------------------------------------------------|
| **Plattformübergreifend** | macOS, Linux, Windows — eine Codebasis               |
| **Dual-Interface**        | Desktop-GUI und headless CLI aus demselben Core      |
| **Zuverlässig**           | Resume bei Abbrüchen, Retry-Logik, Hash-Verifikation |
| **Einfach**               | Datei öffnen → Passwort eingeben → Download starten  |
| **Automatisierbar**       | CLI für Scripting, Cronjobs, Server-Betrieb          |

## Abgrenzung

rsfdl ist **kein**:

- Media-Center oder Film-Verwaltung (kein TMDB, keine Metadaten)
- Streaming-Tool (kein Instant-Video wie SFDL.NET)
- Forum-Scraper (kein automatisches SFDL-Extrahieren von Webseiten)

Diese Features können in späteren Phasen evaluiert werden, gehören aber nicht zum Kernprodukt.

## Technologie

- **Rust** — Performance, Sicherheit, Cross-Platform
- **Dioxus** — Native Desktop-GUI mit Web-Technologie
- **Geteilter Core** — GUI und CLI nutzen dieselbe Bibliothek
