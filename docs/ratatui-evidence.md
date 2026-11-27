# Ratatui implementation evidence

This catalogue links guidance in the skill to primary Ratatui project practice and maintainer discussions. A recommendation is scoped to the evidence shown; it is not a universal requirement for every TUI.

Research checked: 27 November 2026.

## Evidence map

| Guidance | Primary evidence | What it supports | What it does not prove |
|---|---|---|---|
| Separate state/logic tests from rendered-buffer tests | [Ratatui discussion #1381](https://github.com/ratatui/ratatui/discussions/1381) and [discussion #1894](https://github.com/ratatui/ratatui/discussions/1894) | Test state transitions as ordinary logic where possible; use TestBackend when the rendered buffer is part of the behavior under test. | TestBackend does not test real input events, terminal restoration, or an actual terminal emulator. |
| Test interactive journeys with reproducible terminal sessions | [Ratatui Labs](https://github.com/ratatui/ratatui-labs), its [Betamax CI workflow](https://github.com/ratatui/ratatui-labs/blob/main/.github/workflows/ci.yml), and [command-palette tape](https://github.com/ratatui/ratatui-labs/blob/main/tapes/command-palette.tape) | A Ratatui-maintained project uses Betamax tapes to exercise and capture application interactions. | This is evidence that the approach is used, not that every application needs a tape. |
| Assert semantic terminal state, not screenshots alone | [Betamax terminal testing](https://www.joshka.net/betamax/testing/terminal-testing/) | Wait for expected visible content and inspect recorded terminal state; use image snapshots for supplementary review. | Pixel equality alone is not a reliable assertion of application behavior. |
| Keep terminal-session CI unprivileged | [Betamax GitHub Action](https://github.com/joshka/betamax-action) and [GitHub Actions guidance](https://www.joshka.net/betamax/testing/github-action/) | Tapes run shell commands. Use a fresh runner, read-only repository permission, no secrets, and checkout without persisted credentials. | A tape is not harmless arbitrary data; it executes commands with the permissions of the CI job. |
| Handle resize in the application event loop | [Ratatui application development guide](https://ratatui.rs/how-to/develop-apps/terminal-and-event-handler/) and [Terminal API](https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html) | Redraw with the current terminal area after resize and test the application's own declared responsive policy. | Ratatui does not define universal breakpoints or guarantee a specific layout policy. |
| Check APIs against the resolved dependency version | [Ratatui releases](https://github.com/ratatui/ratatui/releases) and [versioned crate documentation](https://docs.rs/ratatui/) | Read the manifest and lockfile, then use API docs/source for the resolved version. | Latest-version docs do not prove an API exists in an older project. |

## How this changes the skill

- State transitions and domain logic can be unit-tested without a terminal.
- TestBackend is appropriate for layout, clipping, styles, and other visible-buffer behavior.
- PTY integration remains useful for actual key input, terminal lifecycle, restoration, and resize survival.
- Betamax is a useful additional tool for selected user journeys, especially when an interaction must be reproducible in CI.
- Resize policy is application-specific. Tests should match declared breakpoints and minimum dimensions.
- Dependency research starts with the project's resolved versions rather than an assumed latest release.

## Evidence limitations

This catalogue is a source map, not a systematic survey of Ratatui applications. It supports the scoped engineering guidance above; it does not prove that the ratatui-skill improves model-generated applications.

That claim requires controlled paired trials with the same model, provider, settings, task text, fixtures, and permissions; multiple repetitions; inspectable agent trajectories; deterministic acceptance checks; and human review. No such complete comparison is claimed by this PR.