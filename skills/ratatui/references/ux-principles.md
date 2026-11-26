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

## Semantic design tokens (terminal-native)

Tokens are role names, not colors. The theme maps each role to a style. Default
to the 16 ANSI colors so the user's palette applies; monochrome keeps bold, dim
and reverse video.

### Foreground roles

- `text.primary`: main content.
- `text.secondary`: supporting information.
- `text.subtle`: metadata, timestamps, hints.
- `text.disabled`: inactive or unavailable items.

### Accent

- `accent.primary`: one restrained accent for the most important emphasis.
- `accent.focus`: focus indicator, usually the same as primary.

### Semantic states

- `state.success`: only for genuine success.
- `state.warning`: only for genuine warnings.
- `state.danger`: only for genuine errors or destructive actions.
- `state.info`: only for neutral informational status.

### Selection

- `selection.foreground` and `selection.background`, or reverse video as a
  fallback on terminals without color.

### Borders

- `border.default`: normal panel borders.
- `border.subtle`: dividers and separators.
- `border.focus`: focused pane border.

### Typography roles

- `type.title`: screen or panel title.
- `type.section`: section header.
- `type.body`: normal content.
- `type.secondary`: supporting text.
- `type.emphasis`: highlighted inline text.

Terminals do not provide arbitrary font weights. Use bold, dim and normal
intensity only.

### Spacing scale (cell units)

- `space.none`: 0 cells.
- `space.xs`: 1 cell.
- `space.sm`: 2 cells.
- `space.md`: 4 cells.
- `space.lg`: 8 cells.

Use these consistently. No arbitrary padding values.

### Layout regions

- `layout.header`: one row, title and summary.
- `layout.content`: fills remaining space.
- `layout.footer`: one row, hints or prompt.
- `layout.gutter`: 1-2 cells between panes.

### Interaction states

- `interaction.normal`: default.
- `interaction.focused`: has keyboard focus.
- `interaction.selected`: currently selected item.
- `interaction.disabled`: cannot be acted on.

Selection and focus must not be confused. A selected item in an unfocused pane
looks different from a focused item.

## Visual hierarchy rules

1. Primary content dominates. Metadata is subtle.
2. One accent color. Secondary information is dim, not a second color.
3. Borders group; whitespace separates. One border style.
4. No box inside a box.
5. No forced background. Terminal transparency survives.
6. Color is never the only signal. Bold, dim and reverse carry meaning.
7. Every visible element earns its place. If it can be removed without losing
   information, remove it.

## Terminal-native geometry

- The cell is the unit. Everything is measured in columns and rows.
- Measure text with `unicode-width`. CJK characters occupy two cells.
- Truncate with an ellipsis, never clip silently.
- Below the minimum size, show a message with the required and actual dimensions.
- Drop secondary panes instead of crushing everything.
- Prefer vertical scrolling over horizontal scrolling.

## Component patterns

### Header

One row. Title on the left, summary on the right. No border.

### Footer

One row. Contextual key hints that change with mode. Filter prompt shows a
cursor. Status messages use `text.subtle` for success, `state.danger` for errors.

### List row

Checkbox or icon, then text, then optional metadata right-aligned. Truncate the
text, never the metadata.

### Selected row

Reverse video or a distinct background. Must be visible in monochrome.

### Focused pane

Border changes from `border.default` to `border.focus`. Only one pane is focused
at a time.

### Help overlay

Generated from the keymap. Shows all bindings grouped by mode. Closes with Esc,
? or q. Must fit or scroll at 40x8.

### Empty state

Say why it is empty and what to do. Centered, muted.

### Error state

Show the reason in plain words. Keep the previous state usable.

### Too-small state

Show required and actual dimensions. Centered.

### Confirmation prompt

Name what will be lost. Offer y/n or the specific keys. Never default to
destructive.

## Motion principles

- No animation that delays keyboard input.
- No decorative movement.
- Use animation only to communicate real state or progress.
- Provide stable non-animated behavior.
- A static interface with excellent hierarchy beats a noisy animated one.
