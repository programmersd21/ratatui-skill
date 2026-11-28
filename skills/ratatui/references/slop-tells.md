# Review signals for generated-looking code and interfaces

"Slop" is not a technical category and cannot be reliably detected by a universal regex. This guide collects signals that deserve review. They are not automatic defects: assess them against the product, audience, existing codebase, and explicit requirements.

Using AI is not itself a quality problem. The concern is shipping code or interfaces whose claims, behavior, and implementation do not agree.

The scanner in `scripts/lint-slop.sh` checks only a narrow set of mechanical conventions. It does not detect most of the issues in this guide. Human review and behavioral tests remain necessary.

## High-confidence problems

These are defects when they contradict the requested behavior:

| Signal | Why it matters | Better approach |
|---|---|---|
| Fake data presented as live or real | Misleads users | Connect real data or label examples clearly |
| A documented key or visible control that does nothing | The interface promises behavior it does not provide | Implement it, remove it, or explain why it is unavailable |
| Placeholder output shipped as completed functionality | The requested work is incomplete | Implement the behavior or report the limitation |
| Unsupported claims about tests, compatibility, or performance | Reviewers cannot trust the report | State exactly what was measured or run |
| Ignored errors that can affect correctness | Failures become invisible | Propagate, handle, or report the error |
| APIs or dependencies chosen without checking the project's versions | Code may not compile or may introduce compatibility problems | Inspect the manifest, lockfile, and version-matched documentation |
| Destructive behavior without safeguards appropriate to its consequences | Users can lose data unexpectedly | Use confirmation, undo, backups, or another proportionate safeguard |

## Design signals to question

These are common generic defaults, not universal prohibitions:

| Signal | Why it may be a problem | Questions to ask |
|---|---|---|
| Neon gradients, glow, or high-saturation accents | Can obscure hierarchy or look unrelated to the product | Does this palette serve the product and remain readable? |
| Emoji used as generic icons | Meaning and rendering vary by terminal | Does the symbol communicate clearly in supported terminals? |
| Decorative banners, taglines, mascots, or badges | May consume space without helping the task | Does this element add information or improve navigation? |
| Panels with no useful content | Adds visual structure without value | What user decision does this panel support? |
| Nested borders and repeated separators | Can make dense interfaces harder to scan | Would spacing or a simpler divider work better? |
| Excessive animation or status indicators | Adds noise and can distract from input | Does motion communicate a real state or progress? |
| A fixed palette, layout, or spacing scale everywhere | Ignores product-specific constraints | Is this convention useful in this application? |
| Too much empty space or too much density | Either can slow comprehension | What information should be visible at this size? |

## Code and architecture signals to question

| Signal | Why it may be a problem | Better approach |
|---|---|---|
| Comments that merely narrate syntax | Adds maintenance noise | Explain intent, invariants, or non-obvious trade-offs |
| Assistant-style conversational leftovers | Belongs in chat, not source code | Remove it |
| Unfinished `TODO`, `todo!`, or `unimplemented!` paths | May indicate incomplete behavior | Finish, track, or explicitly scope the work |
| `.unwrap()` or `.expect()` in application code | May turn recoverable failures into panics | Use error propagation or document a genuine invariant |
| Broad generic names such as `data` or `helper` | Can hide meaning when used without context | Prefer names that describe the domain or operation |
| A very large `main.rs` | Can obscure responsibilities in a complex app | Split modules when real boundaries improve understanding |
| Many tiny modules or wrappers | Can make simple behavior harder to follow | Keep related code together until a useful boundary appears |
| Hex colors scattered through rendering | Can make visual consistency harder to maintain | Use shared semantic styles when multiple views benefit |
| Repeated widget boilerplate | Can cause drift | Extract a helper when behavior is genuinely shared |
| A generic abstraction used only once | Can hide rather than simplify | Keep the direct implementation until reuse or complexity justifies abstraction |

These are prompts for inspection, not a ban list. For example, an `unwrap` may be justified after a proven invariant, and a single-file program may be the clearest design for a small utility.

## Documentation and communication signals

- Feature lists made mostly of adjectives rather than verifiable behavior.
- Screenshots that do not match the current application.
- No runnable command or no explanation of key interactions when users need them.
- Claims of compatibility or production readiness without corresponding checks.
- Repeated sections that restate the same information.
- Unclear disclosure of generated or AI-assisted work when provenance matters to the project.

Prefer specific capabilities, real commands, current screenshots, and honest limitations.

## Before and after

A status message should report a real state, not decorative confidence:

```rust
if let Some(message) = state.status() {
    frame.render_widget(Line::styled(message, theme.error), area);
}
```

A collection view should represent actual data and have an honest empty state:

```rust
if entries.is_empty() {
    render_empty(frame, area, "empty directory");
}
```

## Human review checklist

- Does the implementation satisfy the stated task?
- Are displayed values and status messages truthful?
- Does every documented interaction work?
- Are dependencies and APIs compatible with the project?
- Are errors handled intentionally?
- Is the architecture proportionate to the application's complexity?
- Is the interface readable and useful at the sizes it claims to support?
- Can any decoration, abstraction, module, or instruction be removed without losing value?
- Do tests demonstrate behavior rather than merely mirror implementation details?
- Does the final report distinguish completed checks from unverified work?

A clean lint scan does not prove design quality. A polished screenshot does not prove behavior. Review the implementation and the evidence together.
