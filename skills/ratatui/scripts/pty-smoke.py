#!/usr/bin/env python3
"""Run a TUI binary in a real pseudo-terminal, send keys, check it exits cleanly.

usage: pty-smoke.py BINARY [ARGS...]

Checks: it enters and leaves the alternate screen, exits 0 after `q`, exits 0
on Ctrl-C, survives a tiny terminal, and drops color under NO_COLOR.
"""
import fcntl, os, pty, select, struct, subprocess, sys, termios, time

def run(cmd, keys, size=(30, 100), env=None):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", size[0], size[1], 0, 0))
    e = dict(os.environ, TERM="xterm-256color")
    e.pop("NO_COLOR", None)
    e.update(env or {})
    proc = subprocess.Popen(cmd, stdin=slave, stdout=slave, stderr=slave, env=e)
    os.close(slave)
    out = b""

    def drain(seconds):
        nonlocal out
        end = time.time() + seconds
        while time.time() < end:
            ready, _, _ = select.select([master], [], [], 0.05)
            if ready:
                try:
                    out += os.read(master, 65536)
                except OSError:
                    return

    drain(0.8)
    for key in keys:
        os.write(master, key.encode())
        drain(0.25)
    try:
        code = proc.wait(timeout=3)
    except subprocess.TimeoutExpired:
        proc.kill()
        code = "TIMEOUT"
    drain(0.2)
    return code, out

def main():
    cmd = sys.argv[1:]
    if not cmd:
        sys.exit(__doc__)
    failures = []

    def check(name, ok):
        print(("ok    " if ok else "FAIL  ") + name)
        if not ok:
            failures.append(name)

    code, out = run(cmd, ["j", "?", "\x1b", "/", "a", "\r", "q"])
    check("quits with q", code == 0)
    check("enters alternate screen", b"\x1b[?1049h" in out)
    check("restores terminal", b"\x1b[?1049l" in out)
    code, _ = run(cmd, ["\x03"])
    check("quits with Ctrl-C", code == 0)
    code, out = run(cmd, ["q"], size=(5, 20))
    # The terminal diff skips blank cells, so words arrive separately; look for
    # the one line that is written in a single piece.
    check("survives a tiny terminal", code == 0 and b"have 20x5" in out)
    _, color = run(cmd, ["q"])
    _, mono = run(cmd, ["q"], env={"NO_COLOR": "1"})
    # ratatui writes ANSI cyan as 38;5;6, possibly combined with other codes.
    def has_cyan(data):
        return b"38;5;6" in data or b"[36m" in data

    check("emits color by default", has_cyan(color))
    check("no color under NO_COLOR", not has_cyan(mono))
    sys.exit(1 if failures else 0)

main()
