#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  printf '%s\n' "usage: lint-slop.sh [PROJECT_DIR]" "Scans Rust source and README conventions; Clippy owns compiler-visible lints."
  exit 0
fi

root="${1:-.}"
[[ -d "$root" ]] || { printf 'not a directory: %s\n' "$root" >&2; exit 2; }

python3 - "$root" <<'PY'
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1]).resolve()
files = sorted(
    p for p in root.rglob("*.rs")
    if not any(part in {"target", "tests", "examples"} for part in p.parts)
)
rules = [
    ("colors only in theme.rs", re.compile(r"Color::(?:Rgb|Indexed|Red|Green|Blue|Cyan|Yellow|Magenta|White|Black|Gray|DarkGray|Light[A-Za-z]+)\b|Color::from_u32"), lambda p: p.name != "theme.rs"),
    ("no emoji in source", re.compile(r"[\U0001F300-\U0001FAFF\u2600-\u27BF\u2B50\u2B55]"), lambda p: True),
    ("no unwrap outside tests", re.compile(r"\.unwrap\(\)"), lambda p: True),
    ("no placeholders", re.compile(r"\b(?:TODO|FIXME|XXX|HACK)\b|todo!\(|unimplemented!\(|(?i:lorem ipsum|placeholder)"), lambda p: True),
    ("no swallowed errors", re.compile(r"let\s+_\s*=\s*[^;]*(?:Result|Option|try_|read|write|remove_|send|recv)[^;]*;|\.ok\(\);|Err\(_\)\s*=>\s*\{\}"), lambda p: True),
    ("no debug leftovers", re.compile(r"dbg!\(|eprintln!\(|println!\("), lambda p: True),
    ("no chat artifacts", re.compile(r"(?i)as an ai|here is the|here's the|let me know|certainly!|i hope this"), lambda p: True),
    ("no marketing in strings", re.compile(r'(?i)"[^"]*\b(?:blazing|lightning|supercharge|next-gen|powered by)\b[^"]*"'), lambda p: True),
    ("every key binding is documented", re.compile(r'(?:help|label):\s*""'), lambda p: True),
]
failed = 0
for name, pattern, include in rules:
    hits = []
    for path in files:
        if not include(path):
            continue
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except UnicodeDecodeError:
            continue
        for number, line in enumerate(lines, 1):
            if number > 1 and line.startswith("#[cfg(test)]"):
                break
            if pattern.search(line):
                if name == "no debug leftovers" and "allow print" in line:
                    continue
                if name == "no debug leftovers" and 'println!("{USAGE}")' in line:
                    continue
                hits.append(f"{path.relative_to(root)}:{number}: {line.strip()}")
    if hits:
        failed += 1
        print(f"FAIL  {name} ({len(hits)})")
        for hit in hits[:8]:
            print(f"        {hit}")
        if len(hits) > 8:
            print(f"        ... and {len(hits) - 8} more")
    else:
        print(f"ok    {name}")

readme = root / "README.md"
if readme.is_file():
    emoji = re.compile(r"[\U0001F300-\U0001FAFF\u2600-\u27BF\u2B50\u2B55]")
    hits = [(n, line) for n, line in enumerate(readme.read_text(encoding="utf-8").splitlines()[:20], 1) if line.startswith("#") and emoji.search(line)]
    if hits:
        failed += 1
        print("FAIL  README has no emoji headers")
        for number, line in hits:
            print(f"        README.md:{number}: {line}")
    else:
        print("ok    README has no emoji headers")
else:
    print("skip  README not present")

print()
print("clean" if failed == 0 else f"{failed} rule(s) failed")
sys.exit(failed)
PY
