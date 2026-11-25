#!/usr/bin/env python3
"""Run a TUI binary in a pseudo-terminal and verify terminal behavior.

The checks are split into two groups.

Universal checks apply to every well-behaved ratatui application: the process
starts, enters and restores the alternate screen, shuts down cleanly when told
to quit, and survives a tiny terminal. They make no assumption about colors,
messages, or which keys the application binds.

Application-specific checks are opt-in through --keys, --quit-keys and
--tiny-text. They test a particular app's keymap and wording, so a valid app
with a different style or keymap is never rejected for those reasons.

usage: pty-smoke.py [OPTIONS] BINARY [ARGS...]
"""
import argparse
import fcntl
import os
import pty
import re
import select
import signal
import struct
import subprocess
import sys
import termios
import time

ALT_SCREEN_ENTER = b"\x1b[?1049h"
ALT_SCREEN_LEAVE = b"\x1b[?1049l"
# Any SGR foreground/background color code: 30-37, 90-97, 38;... and 40-47,
# 100-107, 48;... . Not tied to a particular hue.
COLOR_CODE = re.compile(rb"\x1b\[[0-9;]*(?:3[0-7]|9[0-7]|38;[0-9;]+|4[0-7]|10[0-7]|48;[0-9;]+)m")


def clean_env(extra=None):
    env = {k: os.environ[k] for k in ("PATH", "HOME", "LANG", "LC_ALL") if k in os.environ}
    env["TERM"] = "xterm-256color"
    env.pop("COLORTERM", None)
    for key in ("NO_COLOR", "CLICOLOR", "CLICOLOR_FORCE", "FORCE_COLOR"):
        env.pop(key, None)
    env.update(extra or {})
    return env


class Session:
    def __init__(self, cmd, size=(24, 80), env=None, timeout=5.0, sigint=False):
        self.cmd = cmd
        self.timeout = timeout
        self.out = b""
        self.master = None
        self.proc = None
        self.master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", size[0], size[1], 0, 0))
        try:
            kwargs = {"stdin": slave, "stdout": slave, "stderr": slave, "env": clean_env(env)}
            if sigint and hasattr(os, "setsid"):
                kwargs["start_new_session"] = True
            self.proc = subprocess.Popen(cmd, **kwargs)
        finally:
            os.close(slave)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        if self.proc is not None and self.proc.poll() is None:
            self.proc.kill()
            self.proc.wait()
        if self.master is not None:
            os.close(self.master)
            self.master = None
        return False

    def drain(self, seconds):
        end = time.time() + seconds
        while time.time() < end:
            ready, _, _ = select.select([self.master], [], [], 0.05)
            if ready:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError:
                    return
                if not chunk:
                    return
                self.out += chunk

    def wait_ready(self):
        # Read until the alternate-screen escape appears, instead of guessing a
        # fixed warm-up interval. Fall back to a short drain for apps that use a
        # different viewport.
        end = time.time() + self.timeout
        while time.time() < end:
            self.drain(0.05)
            if ALT_SCREEN_ENTER in self.out or self.proc.poll() is not None:
                break
        # Give the first frame time to draw the body after entering the screen.
        self.drain(0.2)

    def send(self, keys, settle=0.2):
        for key in keys:
            try:
                os.write(self.master, key.encode())
            except OSError:
                return
            self.drain(settle)

    def finish(self):
        try:
            return self.proc.wait(timeout=self.timeout)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait()
            return "TIMEOUT"

    def exit(self, keys, sigint=False):
        self.wait_ready()
        if sigint and hasattr(os, "setsid"):
            self.proc.send_signal(signal.SIGINT)
        else:
            self.send(keys)
        code = self.finish()
        self.drain(0.2)
        return code, self.out


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--quit-keys", default="\x03", help="keys that must quit the app (default: Ctrl-C)")
    parser.add_argument("--keys", default="", help="app-specific interaction sequence to send before quitting")
    parser.add_argument("--tiny-text", default="", help="optional expected text in a tiny terminal")
    parser.add_argument("--width", type=int, default=80, help="normal terminal width")
    parser.add_argument("--height", type=int, default=24, help="normal terminal height")
    parser.add_argument("--timeout", type=float, default=5.0, help="seconds to wait for events")
    parser.add_argument("--check-ctrl-c", action="store_true", help="test the Ctrl-C keymap event")
    parser.add_argument("--check-color", action="store_true", help="test color under NO_COLOR")
    parser.add_argument("binary", nargs=argparse.REMAINDER, help="binary and its arguments")
    args = parser.parse_args()

    if not args.binary:
        parser.error("BINARY is required")

    quit_keys = list(args.quit_keys)
    failures = []

    def check(name, ok, detail=""):
        print(("ok    " if ok else "FAIL  ") + name)
        if not ok:
            failures.append(name)
            if detail:
                print("       " + detail)

    size = (args.height, args.width)
    seq = list(args.keys) + quit_keys
    code, out = Session(args.binary, size=size, timeout=args.timeout).exit(seq if seq else [])
    check("starts and exits cleanly", code == 0, f"exit={code}, output={out[:300]!r}")
    check("enters alternate screen", ALT_SCREEN_ENTER in out, f"output={out[:300]!r}")
    check("restores terminal", ALT_SCREEN_LEAVE in out, f"output={out[:300]!r}")

    code, small = Session(args.binary, size=(5, 20), timeout=args.timeout).exit(quit_keys)
    check("survives a tiny terminal", code == 0, f"exit={code}")
    if args.tiny_text:
        visible = re.sub(rb"\x1b\[[0-9;?]*[ -/]*[@-~]", b"", small)
        words = [w.encode() for w in args.tiny_text.split() if w]
        check(
            "tiny terminal reports a size problem",
            bool(words) and all(w in visible for w in words),
            f"visible={visible[:300]!r}",
        )

    if quit_keys:
        code, _ = Session(args.binary, size=size, timeout=args.timeout).exit(quit_keys)
        check("quit keys work", code == 0, f"exit={code}")

    if args.check_ctrl_c:
        code, _ = Session(args.binary, size=size, timeout=args.timeout).exit(["\x03"])
        check("exits cleanly on the Ctrl-C keymap event", code == 0, f"exit={code}")

    if args.check_color:
        _, color = Session(args.binary, size=size, timeout=args.timeout).exit(quit_keys or ["q"])
        _, mono = Session(args.binary, size=size, timeout=args.timeout, env={"NO_COLOR": "1"}).exit(quit_keys or ["q"])
        check("emits color by default", bool(COLOR_CODE.search(color)), f"output={color[:300]!r}")
        check("no color under NO_COLOR", not COLOR_CODE.search(mono), f"output={mono[:300]!r}")

    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    main()
