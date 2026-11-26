# Version-aware skill distribution: investigation

## Question

Can the ratatui skill be distributed alongside Ratatui itself so that the skill
version always matches the crate version the agent is working with?

## What Cargo supports today

Cargo's `include` and `exclude` fields in `Cargo.toml` control which files ship
in the published crate archive. A skill directory (Markdown + scripts) can be
included in the archive without affecting compilation, since only `.rs` files
under `src/` are compiled.

Proof of concept: adding `include = ["src/**", "skills/ratatui/**"]` to a
crate's `Cargo.toml` would ship the skill files in the `.crate` tarball. The
skill would be discoverable at `~/.cargo/registry/src/*/ratatui-*/skills/`.

## What Cargo does not support

- Automatic skill discovery by AI agents. There is no standard mechanism for an
  agent to find skills inside a Cargo registry cache.
- Version-specific skill selection. An agent would need to know which crate
  version the project uses, then find the matching skill in the registry cache.
- Cross-crate skill composition. If ratatui ships a skill and crossterm ships
  one, the agent must merge them.

## library-skills.io applicability

library-skills.io is a proposed standard for shipping skills with libraries. It
is not yet adopted by the Rust ecosystem. Cargo has no native support for it.
Implementing it would require:

1. A Cargo build script or post-install hook to extract skills.
2. A standard discovery path (e.g., `~/.config/agent-skills/`).
3. Agent-side support for discovering skills from that path.

This is premature for a single skill. The overhead of maintaining a distribution
mechanism exceeds the benefit until multiple Rust crates adopt the standard.

## Recommendation

**Do not distribute via Cargo yet.** Instead:

1. Keep the skill as a standalone repository (current approach).
2. Pin the skill's tested Ratatui version in SKILL.md and references.
3. Add a `rust-version` check to the research step so the agent detects version
   mismatches early.
4. Revisit when library-skills.io or an equivalent standard gains Rust ecosystem
   adoption.

## Trade-offs

| Approach | Pros | Cons |
|---|---|---|
| Standalone repo (current) | Simple, version-independent, easy to update | Skill version can drift from crate version |
| Cargo include | Automatic version matching | No discovery mechanism, bloats crate archive |
| library-skills.io | Standardized | Not adopted by Rust, requires new tooling |

## Minimal proof of concept

Add to a crate's `Cargo.toml`:

```toml
[package]
include = ["src/**", "skills/**", "Cargo.toml", "Cargo.lock", "README.md"]
```

This ships the skill but does not make it discoverable. It is a packaging
experiment, not a distribution solution.
