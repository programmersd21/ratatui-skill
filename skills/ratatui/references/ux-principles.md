# TUI design principles

Drawn from lazygit's published design principles (discoverability, simplicity, safety, power, speed) and from terminal design systems that agree on spatial consistency, keyboard-first input, layered color, and a UI that never freezes. Where a rule below is this skill's own judgment rather than something those sources say, it is marked **(judgment)**.

## Layout

Pick one archetype and keep it.

| Archetype | Use when | Example |
|---|---|---|
| List + preview | One collection, details of the selected item | file browser, log viewer |
| Multi-panel | Several related collections the user moves between | lazygit |
| Drill-down | Hierarchy where depth matters, one level at a time | settings, k8s resources |
| Miller columns | Hierarchy where seeing the parents helps | ranger |
| Dashboard | Glanceable live numbers, little input | system monitor |
| Header + list | One collection, nothing else | process picker |
| Popup / overlay | A short question or a help screen over any of the above | confirm, `?` |

Rules:

- **A fixed skeleton.** Header, body, footer stay where they are. Panels do not move unless the user asks.
- **Responsive by dropping, not squashing.** Below a width, remove the preview pane; below a minimum size, show "terminal too small" with the needed size.
- **Borders group, whitespace separates.** One border style. No box inside a box.
- **Lists show only the rows on screen** and keep the selection visible with minimal scrolling.

## Keys

- **Keyboard complete.** Every action has a key. Mouse is optional; if something looks clickable it must work **(judgment: prefer none over fake)**.
- **Three tiers of discovery:** footer hints for the current mode, `?` overlay for everything, README for the rest. Generate the first two from the keymap so they cannot drift.
- **Vim-style plus arrows** (`j k h l g G`, Ctrl-u/d) unless the domain says otherwise. Keep `/` for filter, `?` for help, `q` and Ctrl-C to quit, Esc to cancel.
- **Never reassign standard keys** to something surprising.
- **Modes are visible.** The footer changes with the mode; a text prompt shows a cursor.
- **Fewest keypresses for the common path.**

## Color

Design in three layers:

1. **Monochrome works.** Bold, dim and reverse carry the meaning. This is also the `NO_COLOR` theme.
2. **16 ANSI colors look right.** They follow the user's terminal palette, so the app fits their setup.
3. **True color is an opt-in extra,** never the only way things are readable.

Define colors **by role** (`text`, `muted`, `accent`, `selected`, `error`) in one file. Use one accent. Do not force a background. Neon palettes are a legitimate theme but they are the look people recognize as generated, so make them opt-in **(judgment)**.

## Responsiveness

- Never freeze the UI. Slow work runs elsewhere and reports progress.
- Esc always returns control.
- Startup is fast; do the minimum before the first frame.
- Handle resize without panics, at every size including 1x1.

## Safety

- Destructive actions confirm, and the confirmation says what will be lost.
- Show errors with their reason and keep the previous state usable.
- Treat all external text as hostile: strip control characters before drawing.

## States every screen needs

| State | Show |
|---|---|
| Empty | Why it is empty and what to do about it |
| Loading | That work is happening, with progress if known |
| Error | The reason, in plain words |
| Too small | The size needed and the size you have |
| No results (filtered) | The filter text, so the user can see why |
