# TUI design guidance

This document contains design heuristics for terminal interfaces, not Ratatui requirements. Product requirements, user expectations, existing project conventions, terminal capabilities, and accessibility needs take precedence.

The examples describe common patterns. Combine, adapt, or ignore them when the application calls for something else.

## Classification

- **Correctness:** behavior the application must preserve, such as terminal restoration, safe text handling, responsive input, and accurate information.
- **Recommended:** patterns that often improve usability, but have legitimate exceptions.
- **Optional:** visual and interaction choices that depend on the product.

Do not turn an optional preference into a hard requirement in a new or existing project.

## Layout

Choose a layout from the interaction model and the information users need.

| Pattern | Often useful when | Example |
|---|---|---|
| List + preview | One collection with details for a selected item | File browser, log viewer |
| Multi-panel | Users move among several related collections | Git client |
| Drill-down | Hierarchical navigation where depth matters | Settings, resource browser |
| Miller columns | Users benefit from seeing hierarchy and parents | File manager |
| Dashboard | Users monitor a small set of changing values | System monitor |
| Header + list | One collection is the main task | Process picker |
| Popup / overlay | A short decision or contextual help belongs above the current screen | Confirmation, help |

### Useful layout heuristics

- Keep primary content visually stable when that helps spatial memory; let layouts change when mode or available space requires it.
- Prefer dropping secondary information or panes over compressing important content into unreadable widths.
- Use borders to group related content and whitespace to separate it. Nested borders are a visual choice, not inherently wrong.
- Render only visible rows when rendering the full dataset would be expensive.
- Keep selection visible when navigation depends on it.
- Choose breakpoints from the content's minimum useful dimensions, not from a universal 40/80-column rule.

## Keyboard and interaction

Keyboard-first interaction is a strong default for TUIs, but the correct bindings depend on the domain and audience.

- Make important actions discoverable through context hints, help, documentation, or clear labels.
- Keep key behavior, footer hints, and help text consistent when they share bindings. A shared binding table can help when it reduces duplication.
- Vim-style bindings and arrow keys are both common. Support the conventions appropriate to the application; do not impose one set on every project.
- Preserve familiar terminal conventions when possible, but do not sacrifice a required domain interaction to follow a generic key map.
- Make the current mode or focus visible when it changes how input is interpreted.
- Ensure important functionality is reachable without a mouse when the application is intended to be keyboard-driven.
- Do not advertise a key or control that does nothing.
- Esc is often useful for backing out or cancelling, but its exact behavior should be clear and consistent.

A help overlay, footer hints, and mode-specific key tables are useful patterns, not mandatory components for every small utility.

## Color and visual hierarchy

Design for the terminal environments the application actually supports.

- Do not rely on color alone to communicate an important state.
- Maintain readable contrast and a clear distinction between selected, focused, disabled, and normal content.
- Respect `NO_COLOR` when the application claims to support it.
- A role-based theme can make colors easier to maintain. A single theme module is useful when several components share semantic styles; a smaller app may not need a separate theme abstraction.
- The user's terminal palette, ANSI colors, true color, and monochrome output are all valid choices within the product's compatibility goals.
- One accent color, a restrained palette, and consistent border styles are useful defaults, not universal rules.
- Avoid forced backgrounds when terminal transparency is part of the intended experience.

## Responsiveness and performance

### Correctness requirements

- Do not perform unbounded or avoidable blocking work in the draw path when it prevents input or redraws.
- Do not leave the terminal in an unusable state after normal errors or shutdown.
- Handle resize events without panicking in the dimensions the application claims to support.
- Sanitize untrusted text before writing it to the terminal so embedded control sequences cannot manipulate the display.
- Measure strings by terminal display width when alignment or truncation depends on columns.

### Recommended techniques

- Keep expensive work bounded, cached, incremental, or off the draw path when measurements or expected workload justify it.
- Preserve useful state across resize when possible.
- Prefer vertical scrolling over horizontal scrolling when it fits the data.
- Show progress for genuinely long-running operations.
- Test small terminal sizes that are relevant to the application. A 1x1 terminal can be a useful stress case, but an application may explicitly define a larger minimum usable size.

Do not introduce async tasks, worker threads, caches, or complex rendering abstractions without a concrete need.

## States and feedback

Consider which states can actually occur:

| State | Useful feedback |
|---|---|
| Empty | Explain why there is no content and what action is available |
| Loading | Indicate that work is in progress; show progress when it is known |
| Error | Give a useful reason and a recovery path |
| No filter results | Make the active filter visible and explain that nothing matched |
| Too small | Explain the minimum useful dimensions when the layout cannot work |
| Success | Confirm meaningful operations without inventing status |

Not every screen needs every state. Do not add loading indicators to operations that complete immediately or confirmation messages for actions whose result is already obvious.

For destructive actions, use confirmation, undo, trash, or another safeguard proportionate to the consequences. Do not add confirmation to every action.

## Architecture and design tokens

A small style struct, a theme module, or semantic role names can all work. Choose the smallest approach that avoids duplicated or inconsistent styling.

Semantic roles such as `text.primary`, `text.secondary`, `accent`, `state.error`, `selection`, and `border.focus` can clarify intent in larger interfaces. They do not require a formal design-token framework.

Likewise, spacing scales and named layout regions can improve consistency when a project has many screens. Do not force a fixed spacing scale, token taxonomy, or module structure onto a small application.

## Component patterns

These are starting points, not recipes:

- **Header:** useful for a title, current location, or summary. Omit it if it duplicates information.
- **Footer:** useful for contextual shortcuts or status. Keep it compact.
- **List row:** emphasize the primary label; include metadata only when it helps a decision.
- **Selected item:** use a treatment visible in the application's supported color modes.
- **Focused pane:** distinguish focus from selection when both concepts exist.
- **Help view:** list the actions relevant to the current interaction model.
- **Empty state:** explain the absence of content and the next useful action.
- **Error state:** preserve context and provide a useful recovery path when possible.
- **Confirmation:** name the consequences and avoid making the destructive action the default.

## Motion

Motion is optional in terminal interfaces.

- Never let decorative animation delay input.
- Animate only when it communicates meaningful progress or state.
- Keep important information available without animation.
- Prefer stable, readable output over movement added only for novelty.

## Review questions

- Is the primary task clear?
- Does the layout reflect the actual information and interaction model?
- Are important actions discoverable and functional?
- Can users distinguish focus, selection, and status?
- Does the application behave well at its supported terminal sizes?
- Are errors useful and is recovery possible?
- Is color used appropriately for the supported terminals?
- Does every panel or indicator provide useful information?
- Is any complexity or decoration present only because a template suggested it?

A deliberate interface is one in which each decision has a reason. It does not have to look like any single visual system.
