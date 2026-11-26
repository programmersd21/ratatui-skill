# Agent compatibility

`skills/ratatui/` is the canonical skill source in this repository. It follows the open Agent Skills directory format: one directory containing `SKILL.md` and optional `references/`, `scripts/`, and template assets.

Do not maintain separate copies of the instructions for each agent. Install the same canonical directory at the discovery path for the client you use.

## Project-level discovery paths

| Agent | Project-level path | Notes |
|---|---|---|
| OpenAI Codex | `.agents/skills/ratatui/` | Codex also supports user-level `~/.agents/skills/ratatui/`. |
| Claude Code | `.claude/skills/ratatui/` | Claude Code also supports `~/.claude/skills/ratatui/`. |
| Cursor | `.agents/skills/ratatui/` or `.cursor/skills/ratatui/` | Cursor also recognizes Claude and Codex skill directories. |
| Google Antigravity | `.agents/skills/ratatui/` | Shared workspace path for Antigravity IDE and CLI. |
| OpenCode | `.opencode/skills/ratatui/` | OpenCode also discovers `.agents/skills/` and `.claude/skills/` as compatibility locations. |

The shared `.agents/skills/ratatui/` path is the simplest project-level choice when using Codex, Cursor, Antigravity, and OpenCode together. Claude Code currently uses its own `.claude/skills/` project path, so install a copy there as well when Claude Code is part of the workflow.

## Global discovery paths

| Agent | User-level path |
|---|---|
| OpenAI Codex | `~/.agents/skills/ratatui/` |
| Claude Code | `~/.claude/skills/ratatui/` |
| Cursor | `~/.agents/skills/ratatui/` or `~/.cursor/skills/ratatui/` |
| Google Antigravity | `~/.gemini/config/skills/ratatui/` |
| OpenCode | `~/.config/opencode/skills/ratatui/` |

OpenCode additionally accepts global compatibility directories `~/.agents/skills/` and `~/.claude/skills/`.

## Install from this repository

Run these commands from the root of the cloned `ratatui-skill` repository. Replace `/path/to/project` with the root of the project where you want the skill available.

### Shared location

For Codex, Cursor, Antigravity, and OpenCode:

```bash
mkdir -p /path/to/project/.agents/skills
cp -a skills/ratatui /path/to/project/.agents/skills/ratatui
```

### Claude Code

```bash
mkdir -p /path/to/project/.claude/skills
cp -a skills/ratatui /path/to/project/.claude/skills/ratatui
```

### OpenCode native location

```bash
mkdir -p /path/to/project/.opencode/skills
cp -a skills/ratatui /path/to/project/.opencode/skills/ratatui
```

### Global install

For a user-level install, replace the project directory above with the global path from the tables. For example, Codex:

```bash
mkdir -p ~/.agents/skills
cp -a skills/ratatui ~/.agents/skills/ratatui
```

Avoid maintaining multiple copies of the same skill in overlapping discovery directories for one agent unless you intentionally want to test precedence or overrides. Some clients load more than one source and resolve duplicate skill names using product-specific precedence rules.

## What "compatible" means

There are separate levels of compatibility:

1. **Format-compatible:** the directory contains the standard `SKILL.md` frontmatter and supporting files.
2. **Discovery-documented:** the agent's official documentation lists the installation path.
3. **Runtime-tested:** the skill has been installed in that product and verified to appear, trigger, load references, and run its scripts.

This repository uses the standard skill directory format and documents the official discovery paths linked below. The paths are not a claim that every listed client has been smoke-tested with this repository. Live discovery, trigger behavior, reference loading, and script execution still need product-by-product validation.

The skill's portable content is shared. Product-specific features, invocation controls, and context-loading behavior may differ.

## Official documentation

- [OpenAI Codex skill locations and configuration](https://learn.chatgpt.com/docs/build-skills) and [OpenAI's skill-based OSS maintenance workflow](https://developers.openai.com/blog/skills-agents-sdk)
- [Claude Code skills](https://code.claude.com/docs/en/skills)
- [Cursor Agent Skills](https://cursor.com/docs/skills)
- [Google Antigravity agent skills](https://antigravity.google/docs/skills)
- [OpenCode skills](https://opencode.ai/docs/skills/)
- [Agent Skills specification](https://agentskills.io/specification)
