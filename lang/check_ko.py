#!/usr/bin/env python3
"""Check lang/ko.tsv against garden.sh.

  python lang/check_ko.py          # file format, orphan keys, coverage
  python lang/check_ko.py --run    # also show the menu in Korean and in "both"

What it checks:
 1. ko.tsv is UTF-8 without BOM, LF line endings, one TAB per line, no
    duplicate English keys.
 2. Every English key is a sentence garden.sh really shows (no orphans).
 3. Coverage: how many of garden.sh's fixed sentences have Korean words
    (lines marked "#=" count too: they are the same in both languages).
 4. With --run: copies this folder (without backups/) to a temporary folder,
    writes "ko" and then "both" into .garden-lang there, runs
    printf '0\\n' | bash garden.sh   (show the menu, then 0 = Quit), and
    deletes the copy. Nothing in the real folder changes.
"""
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
TSV = HERE / "ko.tsv"
sys.path.insert(0, str(HERE))
import extract_strings  # noqa: E402


def read_tsv():
    raw = TSV.read_bytes()
    problems = []
    if raw.startswith(b"\xef\xbb\xbf"):
        problems.append("file starts with a BOM")
    if b"\r" in raw:
        problems.append("file has CR (Windows) line endings")
    text = raw.decode("utf-8")
    keys, neutral = {}, []
    for n, line in enumerate(text.split("\n"), 1):
        if not line:
            continue
        if line.startswith("#=\t"):
            neutral.append(line[3:])
            continue
        if line.startswith("#"):
            continue
        if line.count("\t") != 1:
            problems.append(f"line {n}: needs exactly one TAB: {line[:60]!r}")
            continue
        en, ko = line.split("\t")
        if not ko.strip():
            problems.append(f"line {n}: empty Korean for {en!r}")
        if en in keys:
            problems.append(f"line {n}: duplicate key {en!r}")
        if en != en.lstrip(" "):
            problems.append(f"line {n}: key starts with spaces (garden.sh strips them): {en!r}")
        keys[en] = ko
    return keys, neutral, problems


def find_bash():
    for p in (os.environ.get("GARDEN_BASH"),
              r"C:\Program Files\Git\bin\bash.exe",
              r"C:\Program Files\Git\usr\bin\bash.exe"):
        if p and Path(p).exists():
            return p
    return shutil.which("bash")   # on Linux: the normal bash


def run_menu():
    bash = find_bash()
    tmp = Path(tempfile.mkdtemp(prefix="garden-lang-test-"))
    try:
        copy = tmp / "junsgarden-for-jun"
        shutil.copytree(ROOT, copy, ignore=shutil.ignore_patterns("backups", ".git", "target", "local", "logs"))
        for mode in ("ko", "both"):
            (copy / ".garden-lang").write_text(mode + "\n", encoding="utf-8", newline="\n")
            r = subprocess.run([bash, "garden.sh"], cwd=copy, input=b"0\n",
                               capture_output=True, timeout=60)
            out = r.stdout.decode("utf-8", "replace")
            print(f"\n===== menu with .garden-lang = {mode}  (exit code {r.returncode}) =====")
            print(out)
            err = r.stderr.decode("utf-8", "replace").strip()
            if err:
                print("----- stderr (harmless on Windows: commands like 'hostname -I') -----")
                print(err)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
        print(f"\n(temporary copy deleted: {not tmp.exists()})")


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    keys, neutral, problems = read_tsv()
    static, dynamic = extract_strings.collect()
    found = {t for t, _ in static}

    orphans = [k for k in list(keys) + neutral if k not in found]
    covered = [t for t, _ in static if t in keys or t in neutral]
    missing = [t for t, _ in static if t not in keys and t not in neutral]

    print(f"ko.tsv: {len(keys)} translations, {len(neutral)} marked same-in-both-languages")
    print(f"garden.sh: {len(static)} fixed sentences, {len(dynamic)} with $variables (stay English)")
    print(f"Coverage: {len(covered)}/{len(static)} = {100 * len(covered) / max(1, len(static)):.1f}%")
    for p in problems:
        print("PROBLEM:", p)
    for o in orphans:
        print("ORPHAN (not in garden.sh):", repr(o))
    for m in missing:
        print("NOT TRANSLATED:", repr(m))
    ok = not problems and not orphans
    print("RESULT:", "OK" if ok and not missing else ("OK, but some sentences are not translated" if ok else "FIX THE LINES ABOVE"))

    if "--run" in sys.argv:
        run_menu()
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
