#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  printf '%s\n' "usage: lint-slop.sh [PROJECT_DIR]" "Runs heuristic source-style checks; compiler-visible lints belong in Clippy."
  exit 0
fi

root="${1:-.}"
[[ -d "$root" ]] || { printf 'not a directory: %s\n' "$root" >&2; exit 2; }

python3 - "$root" <<'PY'
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1]).resolve()
excluded = {".git", "target"}
files = sorted(
    path
    for path in root.rglob("*.rs")
    if not excluded.intersection(path.relative_to(root).parts)
    and "tests" not in path.relative_to(root).parts
)
rules = [
    (
        "colors are defined in theme modules",
        re.compile(r"Color::(?:Rgb|Indexed|Red|Green|Blue|Cyan|Yellow|Magenta|White|Black|Gray|DarkGray|Light[A-Za-z]+)\b|Color::from_u32"),
        lambda path: path.name != "theme.rs",
    ),
    (
        "key labels and help text are non-empty",
        re.compile(r"(?:label|help):\s*\x22\x22"),
        lambda path: True,
    ),
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
            if pattern.search(line):
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
    hits = [
        (number, line)
        for number, line in enumerate(readme.read_text(encoding="utf-8").splitlines()[:20], 1)
        if line.startswith("#") and emoji.search(line)
    ]
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
print("clean" if failed == 0 else f"{failed} heuristic rule(s) failed")
sys.exit(failed)
PY
