# Scoring rubric

Each trial is scored 0-10 on each dimension. A trial's total is the sum.

| Dimension | 0 | 1 | 2 |
|---|---|---|---|
| Compiles | No | With warnings | Clean |
| API correctness | Stale/invented APIs | Minor issues | Current APIs only |
| Terminal lifecycle | Broken or missing | Partial | Correct init/restore |
| Keyboard | Dead or wrong keys | Partial | All documented keys work |
| Resize | Panics or corrupts | Degrades poorly | Graceful at all sizes |
| States | Missing | Partial | Empty/loading/error/too-small all present |
| Architecture | Monolithic | Mixed | State/ui separated |
| Tests | None or trivial | Superficial | Meaningful behavioral tests |
| Complexity | Over-engineered | Reasonable | Minimal and clear |
| Skill adherence | Ignored skill | Partial | Followed workflow |

## Pass threshold

A trial passes if total >= 14/20 and compiles (score >= 1 on "Compiles").

## Skill vs no-skill comparison

Report:
- Pass rate (skill) vs pass rate (no skill)
- Mean score difference
- Specific failure modes unique to each condition
- Token/cost overhead of skill-enabled runs
