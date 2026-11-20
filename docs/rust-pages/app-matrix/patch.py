#!/usr/bin/env python3
"""Change exact text in a file of a THROWAWAY test copy (matrix.sh), and fail unless each old
text is there exactly once, so a test build can never be patched by accident or by half.

    python patch.py FILE OLD NEW [OLD NEW ...]
"""
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
pairs = sys.argv[2:]
if not pairs or len(pairs) % 2:
    sys.exit("patch.py FILE OLD NEW [OLD NEW ...]")
for old, new in zip(pairs[::2], pairs[1::2]):
    n = text.count(old)
    if n != 1:
        sys.exit(f"patch.py: {old!r} found {n} times in {path.name}, expected once")
    text = text.replace(old, new)
path.write_text(text, encoding="utf-8")
