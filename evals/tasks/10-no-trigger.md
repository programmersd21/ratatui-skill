# Eval 10: Task that should NOT trigger the Ratatui skill

## Task

language: python

Write a single-file Python program named solution.py that accepts a CSV path as
its only command-line argument. Print one JSON object mapping each numeric
column to count, mean, median, and population standard deviation (std). Ignore
blank cells. A column is numeric when it has at least one non-blank value and
every non-blank value parses as a finite number. Ignore non-numeric columns.
Use only the Python standard library.

## Acceptance criteria

- Reads the input CSV path from argv
- Emits valid JSON to stdout
- Ignores blank cells in numeric columns
- Reports count, mean, median, and population standard deviation
- Omits non-numeric and all-blank columns
- Handles a missing input path with a useful error and a non-zero exit
- Does not create a Ratatui or Rust project

## Should trigger skill: no
