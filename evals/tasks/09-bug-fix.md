# Eval 09: Fix a bug without restructuring

## Context

language: rust
fixture: dirview-filter-reset

The task fixture deliberately resets selection to index 0 on every filter
keystroke. Fix it so filtering preserves the selected entry when it remains
visible, and otherwise selects the nearest valid entry.

## Acceptance criteria

- Smallest possible diff (no restructuring, no new abstractions)
- Existing tests still pass
- New test specifically for the bug
- No changes to unrelated code
- Key bindings unchanged

## Should trigger skill: yes
