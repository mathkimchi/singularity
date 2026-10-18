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
- Also a "reload settings" command.
- Settings live in a global in `singularity_common` readable from every applet thread; a settings-changed event is broadcast so applets redraw.

## Palette roles

- Colors are currently hardcoded `Color::*` constants (~100 call sites). Settings need semantic roles (`bg`, `fg`, `accent`, `focused_border`, `inactive`, ...) and call sites moved to them.
- Migrate incrementally: treeview, borders, project manager first.

## Later

- Commands/keybinds by name from the same program. Only things that must run code in-process would need a dylib.
- Settings editor: deferred; schema autocomplete covers most of it.
