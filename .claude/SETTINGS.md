# Settings (issue #63)

Scope for now: cosmetic settings only: palette, border (width, focused/unfocused color), gap, radius, font size. Cursor dropped for now.

## Source

- Config dir: `$XDG_CONFIG_HOME/sonamu` (default `~/.config/sonamu`). The repo's `examples/config` is the reference config; the dev machine symlinks `~/.config/sonamu -> examples/config`.
- If `settings` in the config dir is executable, sonamu runs it and parses its stdout as JSON. Otherwise it reads `settings.json`.
- `examples/config/settings` is a python script that builds the palette and border colors from DankMaterialShell (`~/.cache/DankMaterialShell/dms-colors.json`, current `mode`) and radius from `cornerRadius` in `~/.config/DankMaterialShell/settings.json`, and watches both. Gap, border width and font size are fixed in the script (DMS has none).
- The program can be anything; the intended Rust form is a small binary depending on a `sonamu_settings` crate that builds `Settings` and prints it as JSON.
- Not a dylib: JSON over a pipe has no ABI coupling and a crashing/hanging program can't take down the compositor.

## Format

- `Settings` derives `serde` + `schemars::JsonSchema`; the schema is written out so `"$schema"` gives editor autocomplete.
- Every field is `#[serde(default)]`; partial files are fine.
- Optional top-level `"watch": [paths]`: extra files to watch (eg the file colors are generated from). Not part of the stored settings.

## Reload

- Watch the settings file/program and every `watch` path (`notify`). On change: re-run/re-read.
- Program gets a timeout. On spawn failure, timeout, non-zero exit or parse error: keep the previous settings and log.
- Also a "reload settings" command (`settings::reload()` exists; no command wired yet).
- Settings live in a global in `singularity_common` readable from every applet thread; a settings-changed event is broadcast so applets redraw.

## Implemented

- `singularity_common::settings`: `Settings` (palette, border, gap, radius, font_size), `load(dir)`, `config_dir()` (`$SONAMU_CONFIG` overrides), global `get()`/`reload()`/`subscribe()`. `load_and_watch()` (called in `singularity_sde` main) loads once, then a `notify` thread reloads on changes. Nothing reads `get()` yet.
- Watching goes through parent dirs (non-recursive), so atomically replaced files are seen. Watched: `settings`, `settings.json` (only while there is no program), and `watch` paths (relative ones resolve against the config dir). Changes are debounced 100 ms.
- Loop detection: after a program run, any `watch` path that changed during the run (+100 ms) is assumed written by the program; it's logged and ignored until restart. Changes to other watched paths in that window cause another reload.
- Broadcast: on an actual change (`!=`), each `subscribe`d calloop sender is pinged; dead ones are dropped. `AppletClient` and the root applet subscribe and damage their surface. No applet hook yet; applets that cache settings would need one.
- `Color` parses/deserializes `#rrggbb`/`#rrggbbaa`.
- Defaults match the current hardcoded colors (border focused = yellow, unfocused = green).
- Known gap: if the program leaves a background child holding stdout open, reading blocks after it exits (timeout only covers the program itself).

## Palette roles

- Colors are currently hardcoded `Color::*` constants (~100 call sites). Settings need semantic roles (`bg`, `fg`, `accent`, `focused_border`, `inactive`, ...) and call sites moved to them.
- Migrate incrementally: treeview, borders, project manager first.

## Later

- Commands/keybinds by name from the same program. Only things that must run code in-process would need a dylib.
- Settings editor: deferred; schema autocomplete covers most of it.
