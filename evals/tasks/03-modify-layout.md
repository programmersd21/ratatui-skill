# Eval 03: Modify an existing application's layout

## Context

language: rust
fixture: todo

Given the todo app, add a sidebar showing completed vs pending counts and a
progress bar. The main list keeps its current behavior.

## Acceptance criteria

- Existing todo tests still pass
- Sidebar disappears below 80 columns (responsive)
- Progress bar reflects real completion ratio
- No regression in add/toggle/delete
- Renders at 40x8 without panic
- No unrelated restructuring

## Should trigger skill: yes
