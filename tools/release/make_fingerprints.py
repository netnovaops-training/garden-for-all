#!/usr/bin/env python3
"""Make (or check) CODE-FINGERPRINTS.txt, the PUBLIC list that the first start checks the program against.

    python tools/release/make_fingerprints.py            write CODE-FINGERPRINTS.txt from what is staged (the git index)
    python tools/release/make_fingerprints.py --check    only check that the list in the index is exactly right (exit 1 if not)
    python tools/release/make_fingerprints.py --tree HEAD   use a commit instead of the index (with --check)

The list has one line per program file: the SHA-256 and the path, two spaces between,
sorted by path. It names every tracked file EXCEPT the list itself and the files that are
kept off GitHub on purpose (OFF_GITHUB below). tools/get-code.py and tools/get-code.ps1
fetch the program from GitHub and refuse it (write nothing) if a file differs from this
list, is missing, or if the download holds a file that is not on the list.

The fingerprint is of the file exactly as get-code.* will write it: the same bytes git
archive (GitHub's download) gives, with Windows line endings on .cmd, .ps1 and .bat files.
The list is read from `git archive` itself, so it matches what GitHub serves.
"""
import argparse, hashlib, io, os, subprocess, sys, tarfile

LIST_NAME = "CODE-FINGERPRINTS.txt"
CRLF_EXT = (".cmd", ".ps1", ".bat")
# Kept off the public GitHub on purpose (or only a different copy is public). Never in the list.
OFF_GITHUB = {"README.md", "site/Boogie.mp3", "FILES-FROM-AUTHOR.txt", "DOWNLOADS-SHA256.txt",
              "PHOTO-PARTS.json", "photos.key", "photos-access.token"}
NOT_LISTED = OFF_GITHUB | {LIST_NAME}


def normalized(rel, data):
    """The bytes get-code.* write for this file (and so the bytes that are fingerprinted)."""
    if rel.lower().endswith(CRLF_EXT):
        return data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    return data


def git(*args, capture=True):
    r = subprocess.run(["git", "-c", "core.quotepath=off", *args], capture_output=capture)
    if r.returncode != 0:
        sys.exit("git " + " ".join(args) + " failed: " + (r.stderr or b"").decode("utf-8", "replace").strip())
    return r.stdout


def tree_files(tree):
    """{path: bytes} of every regular file in the tree, as `git archive` (and so GitHub) gives them."""
    raw = git("archive", "--format=tar", tree)
    files = {}
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as t:
        for m in t.getmembers():
            if m.isfile():
                files[m.name] = t.extractfile(m).read()
    return files


def make_list(files):
    lines = []
    for rel in sorted(files):
        if rel in NOT_LISTED:
            continue
        if "\n" in rel or "\r" in rel:
            sys.exit(f"a path with a line break cannot be listed: {rel!r}")
        lines.append(f"{hashlib.sha256(normalized(rel, files[rel])).hexdigest()}  {rel}\n")
    return "".join(lines)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="do not write; exit 1 if the list in the tree is stale")
    ap.add_argument("--tree", default="", help="a commit or tree (default: the staged files, the git index)")
    A = ap.parse_args()
    os.chdir(git("rev-parse", "--show-toplevel").decode().strip())
    tree = A.tree or git("write-tree").decode().strip()
    files = tree_files(tree)
    want = make_list(files)
    if A.check:
        have = files.get(LIST_NAME)
        if have is None:
            print(f"{LIST_NAME} is not in the tree. Run: python tools/release/make_fingerprints.py")
            sys.exit(1)
        if have.decode("utf-8") == want:
            print(f"{LIST_NAME}: up to date ({want.count(chr(10))} files)")
            return
        a = dict(l.split("  ", 1)[::-1] for l in have.decode("utf-8").splitlines() if "  " in l)
        b = dict(l.split("  ", 1)[::-1] for l in want.splitlines())
        for p in sorted(set(a) | set(b)):
            if p not in a:
                print(f"missing from the list: {p}")
            elif p not in b:
                print(f"in the list but not a file: {p}")
            elif a[p] != b[p]:
                print(f"stale: {p}")
        print(f"{LIST_NAME} is stale. Run: python tools/release/make_fingerprints.py")
        sys.exit(1)
    if A.tree:
        sys.exit("--tree needs --check (the list is written from the staged files)")
    with open(LIST_NAME, "wb") as f:
        f.write(want.encode("utf-8"))
    print(f"wrote {LIST_NAME}: {want.count(chr(10))} files")


if __name__ == "__main__":
    main()
