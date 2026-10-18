# Project UI (issue #68)

## Decisions

- The overview shows while the project's own tree is focused at its inner root (`focused_node() == root()`), i.e.
  on the project itself rather than on something in it. That is also the state right after Super+P, so a freshly
  focused project node (not inside it) shows it too. The project can't tell whether its outer node is `Focusing`
  (`FocusChanged` is never sent), so it doesn't key off that.
- It replaces the panes there (they were the last child's pane before). Rows, from the top: name, key hint,
  `1  <label>` per child of the inner root, `+ new`. Rows are a fixed height so the project hit-tests clicks itself.
- Keys there, taken before the project's selector: `r` (no modifiers) starts renaming; `n` adds a command hub to the
  inner root and focuses inside it (unlike Ctrl+Shift++, which doesn't focus it). Digits/`d`/`e`/`q` stay the selector's.
  While renaming: text, Backspace, Enter (saves; empty keeps the old name), Esc (cancels); Ctrl/Alt/Super keys still
  go to the selector. Mouse events are all taken (no panes are shown).
- Renaming starts from an empty name (starting from the old one made typing append to `Project`).
- The name is the inner root's treeview label, saved as the new `SavedTree::label` on the project's inner root
  (`#[serde(default, skip_serializing_if)]`, so old sessions load and other nodes don't change in the JSON).
  Chose this over changing `SavedApplet::Project(Box<SavedTree>)`, whose JSON form would break old sessions.

- The overview's background is the command hub's `BG`. `BG` and `BAR_BG` use the treeview's `with_opacity` (0.8 alpha),
  so the help applet, which shares them, is translucent too.

## Questions for the user

- Color: not done. Treeview labels are `WorldTree<String>`, so a color needs a protocol change (eg a label struct
  with an optional color, through `Treeview`, `NodeTree::applet_treeview` and the treeview renderer). Want it?
- Should the overview also show while focus is on a child of the project (eg beside the panes), or only at its root?
- Other spawn buttons besides `+ new` (command hub)? Eg Super+T/N style `run foot`.

## Issues found

- `StandardEvent::FocusChanged`/`Highlighted` are never sent by anything.
- `1-9: select` in the hint: `9` is the last child and digits only reach 8 children; beyond that only clicks/`w`/`s`.
- No hover/pointer cursor on the clickable rows.

## Verified (headless, `singularity_sde/examples/headless.rs hub ...`)

Super+P shows the overview; click on the name / `r`, type, Enter renames (treeview and overview); Esc cancels;
click `+ new` / `n` adds a focused hub; Alt+q `a` back to the overview lists both children; clicking one shows its pane;
the name survives a save and restore (`"label"` in the session JSON). Not checked live.
