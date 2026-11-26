# Slop tells

"Slop" here means output that reads as generated: recognizable on sight, with a gap between what it claims and what it does. Using an AI is not the problem; shipping filler is. `skills/ratatui/scripts/lint-slop.sh` checks the mechanical ones; the rest need judgment.

Sources for these tells are public discussions of AI-generated projects (a large study of complaints about AI-built sites, Rust forum and project AI policies, a NixOS forum review of a Rust TUI, Hacker News threads). The common thread across them: effort used to be proof of quality, now it is free, so people look for evidence that someone decided things.

## UI tells

| Tell | Why it reads as slop | Instead |
|---|---|---|
| Purple/cyan neon gradient on dark, glow, thick borders | The look every model reaches for | One ANSI accent, dim for secondary |
| Emoji as icons or in headers | Decoration with no information | Plain text, or a Unicode symbol that carries meaning |
| A "Ready" dot, a banner, a tagline, a mascot | Marketing inside the tool | Nothing. The tool is the interface |
| Panels that show nothing real (fake stats, sample rows) | Looks finished, is not | Show real data or no panel |
| Keys or buttons that do not respond | The most-cited complaint about generated TUIs | Remove it, or make it work |
| Everything boxed, borders inside borders | No hierarchy | Borders group; whitespace separates |
| Swapping one default for another | Still a default | State a choice in the design brief and justify it from the data |
| Polish on the README, none in the program | Effort in the wrong place | Put the effort in the interaction |

## Code tells

| Tell | Instead |
|---|---|
| Comments that narrate (`// increment counter`) | Comment why, or delete |
| Chat leftovers ("Here's the updated code", "Certainly!") | Delete |
| `TODO`, `unimplemented!()`, placeholder strings | Finish it or cut it |
| `.unwrap()` in non-test code | `?`, or handle the case |
| `let _ = fallible();`, empty `Err(_) => {}` | Handle or report the error |
| Generic names (`data`, `item`, `helper`, `utils.rs`) | Name the thing |
| Hex colors scattered through draw code | Colors by role in `theme.rs` |
| One 1,500-line `main.rs` | Modules that follow the data flow |
| Hallucinated crates or methods | Research, then compile (see the checklist) |
| Boilerplate repeated per widget | One helper, used |

## README tells

- Emoji headers, a "Features" list of adjectives ("blazing fast", "beautiful").
- No real command, no real key list.
- No statement of how it was made. Say plainly if AI helped; one line is enough.
- Claims the code does not back up.

## Before and after

```rust
// before: decorative, dishonest
let status = Paragraph::new("● Ready — all systems operational")
    .style(Style::new().fg(Color::Rgb(0, 255, 136)));

// after: shows something true, in a themed role, only when there is something to say
if let Some(message) = state.status() {
    frame.render_widget(Line::styled(message, theme.error), area);
}
```

```rust
// before: invented placeholder
let rows = vec!["Item 1", "Item 2", "Item 3"];

// after: real data, with an honest empty state
if entries.is_empty() {
    render_empty(frame, area, "empty directory");
}
```

## Formal anti-slop design standard

### Prohibited decorative defaults

Reject unless justified by a real product need:

- Neon purple/cyan gradients
- Glowing borders
- Decorative banners
- Fake status indicators
- Fake statistics or activity feeds
- Placeholder rows
- Empty panels
- Unnecessary mascots
- Emoji used as generic icons
- Nested borders everywhere
- Excessive separators
- Redundant headings
- Marketing copy inside developer tools
- Unnecessary welcome screens
- Fake loading indicators
- Decorative animations
- Unnecessary badges
- Redundant status labels

### Information honesty

Every visible value must be real. No fabricated data, no invented progress, no
misleading success messages, no hardcoded metrics presented as live information.
When there is no data, show an honest empty state.

### Typography and copy

- Short, clear labels.
- Consistent terminology.
- Sentence-case labels.
- Concise error messages.
- No unnecessary slogans.
- No assistant-style conversational leftovers.
- No exaggerated marketing adjectives.
- No redundant explanatory text.

### Product-specific decisions

The agent must decide what information matters most, what users do most
frequently, what needs immediate attention, and what can be removed. Every
design choice should have a product reason, not a template default.

### Human review checklist

- Can a first-time user understand the screen?
- Is the primary content immediately obvious?
- Does every panel contain useful information?
- Does every key binding work?
- Can the interface be used without color?
- Does resizing preserve usability?
- Are empty and error states honest?
- Is the screen too dense or too padded?
- Can any element be removed without losing information?
- Does the UI look like a considered product rather than a generic template?

A clean lint scan is not proof of design quality. It is a mechanical guardrail.
