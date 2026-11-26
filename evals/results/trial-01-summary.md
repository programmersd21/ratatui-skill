# Eval 01 trial summary

## Trial: with skill, 1 run (partial)

- **Task**: Create a new Ratatui application (scrollable log viewer)
- **Model**: beam-free (opencode, OpenRouter-backed)
- **Duration**: 180s (timed out before completion)
- **Result**: Agent generated partial code (action.rs, state.rs, keymap.rs, event.rs, lib.rs, Cargo.toml) following a modular structure. Timed out before generating main.rs, app.rs, ui/, theme.rs.
- **Compilation**: Not reached (incomplete)
- **Skill adherence**: Agent followed a modular structure (action/keymap/state/event split) consistent with the skill's guidance, but the trial timed out before completion.
- **Template copying**: Cannot be determined from partial output. Whether the agent copied the dirview template or wrote from scratch is unknown.
- **Key finding**: The 180s timeout is insufficient for a full app generation. The harness now supports configurable timeouts.

## Limitations

- Only 1 partial trial attempted.
- No no-skill comparison run completed.
- Results are preliminary.
- Skill activation is reported as "unverified".
