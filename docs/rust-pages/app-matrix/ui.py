#!/usr/bin/env python3
"""The emulator's screen, read and tapped through adb + uiautomator (matrix.sh).

    python ui.py has TEXT [TEXT ...]    exit 0 if any of the texts is on screen now
    python ui.py tap TEXT [TEXT ...]    tap the first of the texts found (exit 1 if none)
    python ui.py wait TEXT [...]        like has, but waits up to 40 s
    python ui.py tapfirstcard           tap the first plant card under the search box
Uses the adb found on PATH (matrix.sh puts the SDK's platform-tools first, and sets
ANDROID_SERIAL so adb talks to the test emulator only). The dump is read with plain patterns
(no XML parser): it only ever needs each node's text, description, clickable flag and bounds.
"""
import html
import re
import shutil
import subprocess
import sys
import time

ADB = shutil.which("adb") or sys.exit("ui.py: adb is not on PATH")
NODE = re.compile(r"<node\b[^>]*>")
ATTR = re.compile(r'(\w[\w-]*)="([^"]*)"')


def nodes():
    """Every node on screen as a dict of its attributes."""
    subprocess.run([ADB, "shell", "uiautomator", "dump", "/sdcard/ui.xml"], capture_output=True, check=False)
    out = subprocess.run([ADB, "exec-out", "cat", "/sdcard/ui.xml"], capture_output=True, check=False).stdout
    text = out.decode("utf-8", "replace")
    return [{k: html.unescape(v) for k, v in ATTR.findall(m.group(0))} for m in NODE.finditer(text)]


def box(node):
    nums = [int(n) for n in re.findall(r"\d+", node.get("bounds", ""))]
    return nums if len(nums) == 4 else None


def find(texts):
    for node in nodes():
        label = node.get("text", "") + "|" + node.get("content-desc", "")
        b = box(node)
        if b and any(t and t in label for t in texts):
            return (b[0] + b[2]) // 2, (b[1] + b[3]) // 2
    return None


def tap(x, y):
    subprocess.run([ADB, "shell", "input", "tap", str(x), str(y)], check=False)


def main():
    cmd, texts = sys.argv[1], sys.argv[2:]
    if cmd == "has":
        sys.exit(0 if find(texts) else 1)
    if cmd == "wait":
        for _ in range(20):
            if find(texts):
                return
            time.sleep(2)
        sys.exit(1)
    if cmd == "tap":
        at = find(texts)
        if not at:
            sys.exit(1)
        tap(*at)
        return
    if cmd == "tapfirstcard":
        # The first clickable node well below the top bar and search box: the first plant.
        for node in nodes():
            b = box(node)
            if node.get("clickable") == "true" and b and b[1] > 600 and b[3] - b[1] > 150:
                tap((b[0] + b[2]) // 2, (b[1] + b[3]) // 2)
                return
        sys.exit(1)
    sys.exit(__doc__)


if __name__ == "__main__":
    main()
