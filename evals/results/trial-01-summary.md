# Eval 01 trial summary (partial)

## Trial: with skill, 1 run

- **Task**: Create a new Ratatui application (scrollable log viewer)
- **Model**: beam-free (opencode)
- **Duration**: ~180s (timed out before completion)
- **Result**: Agent generated a full dirview-structured project (14 source files) but did not compile because `main.rs` retained `use dirview::App;` instead of renaming to the new crate name.
- **Root cause**: Template copying pitfall (now documented in research-checklist.md).
- **Compilation**: FAIL (unresolved import)
- **Skill adherence**: Agent followed the skill's structure (action/keymap/state/ui/theme split) and included tests, but did not rename the crate reference.
- **Key finding**: The skill's "copy structure" instruction needs to explicitly mention renaming the crate reference in main.rs. This has been added.

## Limitations

- Only 1 trial attempted; timed out.
- No no-skill comparison run completed.
- Results are preliminary and not statistically significant.
