#!/usr/bin/env bash
# Mechanical checks for common TUI anti-patterns. Each rule is deterministic;
# design quality is a separate, human judgment recorded in references/slop-tells.md.
#
# usage: lint-slop.sh [PROJECT_DIR] [--help]
#
# Exit status: 0 when no rule fires, N when N rules fire, 2 on a tooling or
# usage error. Only non-test code is checked. Scan scope: *.rs under PROJECT_DIR
# excluding target/, tests/, examples/ and code after a top-level #[cfg(test)].

set -u

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  printf '%s\n' \
    "usage: lint-slop.sh [PROJECT_DIR]" "" \
    "Scan Rust sources for mechanical anti-patterns." \
    "Rules, why they matter, and how to suppress a line are in references/slop-tells.md." \
    "Exit 0 clean, 1 per failing rule, 2 on usage or missing GNU grep -P."
  exit 0
fi

root="${1:-.}"
[ -d "$root" ] || { printf 'not a directory: %s\n' "$root" >&2; exit 2; }

# A silent pass is worse than no check: fail clearly when the required tool is
# unavailable instead of swallowing grep's non-zero status.
if ! printf 'x\n' | grep -P '^x$' >/dev/null 2>&1; then
  echo "lint-slop: GNU grep with -P (PCRE) is required" >&2
  exit 2
fi

fail=0

source_lines() {
  find "$root" -name '*.rs' -type f \
    -not -path '*/target/*' -not -path '*/tests/*' -not -path '*/examples/*' \
    -print0 2>/dev/null | sort -z | while IFS= read -r -d '' f; do
      # Anchored so a #[cfg(test)] inside a string or comment does not hide code.
      awk -v F="$f" '/^#\[cfg\(test\)\]/ { exit } { print F ":" FNR ": " $0 }' "$f"
    done
}

lines="$(source_lines)"

rule() { # name, pattern, [exclude-pattern]
  local name="$1" pattern="$2" exclude="${3:-^$}" hits count
  # Do not use `|| true`: a grep failure must be visible, not reported as clean.
  hits="$(printf '%s\n' "$lines" | grep -P -- "$pattern" | grep -Pv -- "$exclude")"
  if [ -n "$hits" ]; then
    count="$(printf '%s\n' "$hits" | wc -l | tr -d ' ')"
    echo "FAIL  $name ($count)"
    printf '%s\n' "$hits" | head -8 | sed 's/^/        /'
    [ "$count" -gt 8 ] && echo "        ... and $((count - 8)) more"
    fail=$((fail + 1))
  else
    echo "ok    $name"
  fi
}

# Colors defined away from theme.rs make a theme impossible to audit in one
# place. This is a location rule, not a ban on bright colors.
rule "colors only in theme.rs" \
  'Color::(Rgb|Indexed|Red|Green|Blue|Cyan|Yellow|Magenta|White|Black|Gray|DarkGray|Light[A-Za-z]+)\b|Color::from_u32' \
  '^[^:]*theme\.rs:'
# Control characters in rendered text can move the cursor or inject escapes.
rule "no emoji in source" '[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}\x{2B50}\x{2B55}]'
# An unwrap in production code is a panic waiting for bad input. Tests may unwrap.
rule "no unwrap outside tests" '\.unwrap\(\)'
# Unfinished markers shipped as if complete.
rule "no placeholders" '\b(TODO|FIXME|XXX|HACK)\b|todo!\(|unimplemented!\(|lorem ipsum|placeholder'
# Discarded Results hide failures. A deliberate default (`unwrap_or_default`)
# is explicit error handling and is not flagged here.
rule "no swallowed errors" 'let\s+_\s*=\s*[^;]*(Result|Option|try_|read|write|remove_|send|recv)[^;]*;|\.ok\(\);|Err\(_\) => \{\}'
# Debug output left in production. A per-line marker is an explicit, reviewable
# exception; the usage banner in main.rs is the other allowed case.
rule "no debug leftovers" 'dbg!\(|eprintln!\(|println!\(' \
  'lint-slop: allow print|println!\("\{USAGE\}"\)'
# Chat leftovers in source text.
rule "no chat artifacts" '(?i)as an ai|here is the|here'"'"'s the|let me know|certainly!|i hope this'
# Marketing words inside user-visible strings, not in comments.
rule "no marketing in strings" '(?i)"[^"]*\b(blazing|lightning|supercharge|next-gen|powered by)\b[^"]*"'

# A key with no help text cannot be discovered from the generated ? overlay.
rule "every key binding is documented" 'help:\s*""|label:\s*""'

# README headers must not use emoji.
if [ -f "$root/README.md" ]; then
  readme_hits="$(head -20 "$root/README.md" | grep -P '^#+ .*[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]')"
  if [ -n "$readme_hits" ]; then
    echo "FAIL  README has no emoji headers"
    printf '%s\n' "$readme_hits" | sed 's/^/        /'
    fail=$((fail + 1))
  else
    echo "ok    README has no emoji headers"
  fi
else
  echo "skip  README not present (not checked)"
fi

echo
if [ "$fail" -eq 0 ]; then echo "clean"; else echo "$fail rule(s) failed"; fi
exit "$fail"
