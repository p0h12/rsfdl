# UC-04: Dateien auswählen

## Scope

Dateiauswahl in der GUI vor dem Download:

- Per-File Checkbox (einzelne Datei an-/abwählen)
- Per-Package Checkbox (ganzes Paket an-/abwählen)
- Select All Checkbox (alle Dateien an-/abwählen)
- Grössen- und Anzahl-Berechnung der Auswahl
- Deaktiviert während laufendem Download

## Beteiligte Module

- `gui/src/state.rs` — `selected_files: Signal<Vec<bool>>`, `selected_size()`, `selected_count()`
- `gui/src/components/file_list.rs` — `FileList` Komponente, `render_package_group()`
- `gui/src/components/file_row.rs` — `FileRow` Komponente (Per-File Checkbox)
- `gui/src/views/main_view.rs` — `start_download()` filtert nach Selection

## API Design

```rust
// --- AppState Helpers ---

/// Signal: Flat-Liste von Booleans, ein Eintrag pro Datei über alle Packages.
pub selected_files: Signal<Vec<bool>>;

/// Summe der Grössen nur ausgewählter Dateien.
pub fn selected_size(&self) -> u64;

/// Anzahl ausgewählter Dateien.
pub fn selected_count(&self) -> usize;
```

## Implementierungsdetails

### Datenstruktur

- `selected_files: Signal<Vec<bool>>` — flacher Vektor, indexiert über alle Packages
- Initialisierung: `vec![true; file_count]` in `finish_container_load()`
- Nach BulkFolder-Auflösung: wird neu initialisiert mit `vec![true; new_file_count]`

### Index-Mapping

```
Package 0: files[0], files[1], files[2]    → selected_files[0..3]
Package 1: files[0], files[1]              → selected_files[3..5]
Package 2: files[0]                        → selected_files[5..6]
```

- `FileList` iteriert über alle Packages und baut `(global_idx, file_name, file_size)` Tripel
- `FileRow` erhält seinen `index` (global) als Prop

### Select All

```rust
// In FileList Komponente:
let all_selected = !selected.is_empty() && selected.iter().all(|&s| s);

onchange: {
    let new_val = !all_selected;
    state.selected_files.set(vec![new_val; total_count]);
}
```

### Per-Package Select

```rust
// In render_package_group():
let indices: Vec<usize> = files.iter().map(|(i, _, _)| *i).collect();
let pkg_all_selected = indices.iter().all(|i| selected.get(*i).unwrap_or(false));

onchange: {
    let new_val = !pkg_all_selected;
    let mut sel = state.selected_files.write();
    for &idx in &indices {
        if let Some(v) = sel.get_mut(idx) { *v = new_val; }
    }
}
```

### Per-File Select

```rust
// In FileRow Komponente:
onchange: {
    let mut sel = state.selected_files.write();
    if let Some(v) = sel.get_mut(index) { *v = !*v; }
}
```

### Deaktivierung während Download

- Alle Checkboxes haben `disabled: downloading`
- `downloading = *state.download_phase.read() != DownloadPhase::Idle`

### Download-Filterung

```rust
// In start_download():
let selected = state.selected_files.read().clone();
let mut idx = 0;
for package in &mut container.packages {
    package.file_list.retain(|_| {
        let keep = selected.get(idx).copied().unwrap_or(true);
        idx += 1;
        keep
    });
}
```

- Nicht-ausgewählte Dateien werden aus dem Container entfernt bevor er an `DownloadManager::new()` geht
- `unwrap_or(true)`: Bei Index-Mismatch wird die Datei trotzdem heruntergeladen

### CLI

- In MVP: Alle Dateien werden heruntergeladen (keine Auswahl)
- Geplant für später: `--include` / `--exclude` Pattern
