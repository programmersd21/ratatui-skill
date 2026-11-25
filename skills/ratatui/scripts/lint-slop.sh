#!/usr/bin/env bash
# Scan a Rust TUI project for the tells listed in references/slop-tells.md.
#
# usage: lint-slop.sh [PROJECT_DIR]
#
# Exit status is the number of rules that found something (0 = clean), so it
# can gate CI. Only non-test code is checked: scanning stops at the first
# `#[cfg(test)]` in each file, and files under tests/ and examples/ are skipped.

set -u
root="${1:-.}"
fail=0

# "file:line: text" for every non-test source line.
source_lines() {
  find "$root/src" -name '*.rs' -print0 2>/dev/null | sort -z | while IFS= read -r -d '' f; do
    awk -v F="$f" '/#\[cfg\(test\)\]/ { exit } { print F ":" FNR ": " $0 }' "$f"
  done
}

lines="$(source_lines)"

rule() { # name, pattern, [exclude-file-pattern]
  local name="$1" pattern="$2" exclude="${3:-^$}" hits
  hits="$(printf '%s\n' "$lines" | LC_ALL=C.UTF-8 grep -P -- "$pattern" | grep -Pv -- "$exclude" || true)"
  if [ -n "$hits" ]; then
    echo "FAIL  $name"
    printf '%s\n' "$hits" | head -8 | sed 's/^/        /'
    fail=$((fail + 1))
  else
    echo "ok    $name"
  fi
}

rule "no emoji in source" '[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}\x{2B50}\x{2B55}]'
rule "colors only in theme.rs" 'Color::(Rgb|Indexed|Red|Green|Blue|Cyan|Yellow|Magenta|White|Black|Gray|DarkGray|Light[A-Za-z]+)\b|Color::from_u32' '^[^:]*theme\.rs:'
rule "no unwrap outside tests" '\.unwrap\(\)'
rule "no placeholders" '\b(TODO|FIXME|XXX|HACK)\b|todo!\(|unimplemented!\(|lorem ipsum|placeholder'
rule "no swallowed errors" 'let _ = [^;]*;|\.ok\(\);|Err\(_\) => \{\}|\.unwrap_or_default\(\)\s*;'
rule "no debug leftovers" 'dbg!\(|eprintln!\(|println!\(' '#\[allow\(clippy::print_stdout\)\]|^[^:]*main\.rs:'
rule "no chat artifacts" '(?i)as an ai|here is the|here'"'"'s the|let me know|certainly!|i hope this'
rule "no marketing in the UI" '(?i)\b(blazing|lightning|supercharge|next-gen|powered by|welcome to)\b'

# Every binding must have help text; the help overlay is generated from the
# keymap, so an empty string means a key the user cannot discover.
if grep -rqP 'help:\s*""|label:\s*""' "$root/src" 2>/dev/null; then
  echo "FAIL  every key binding is documented"
  fail=$((fail + 1))
else
  echo "ok    every key binding is documented"
fi

# README must exist and must not open with an emoji header.
if [ -f "$root/README.md" ]; then
  if head -20 "$root/README.md" | LC_ALL=C.UTF-8 grep -qP '^#+ .*[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]'; then
    echo "FAIL  README has no emoji headers"
    fail=$((fail + 1))
  else
    echo "ok    README has no emoji headers"
  fi
fi

echo
if [ "$fail" -eq 0 ]; then echo "clean"; else echo "$fail rule(s) failed"; fi
exit "$fail"
