#!/usr/bin/env python3
"""Get the garden's program files from GitHub and check every one (Ubuntu).

The download from Google Drive (junsgarden-for-jun.zip) carries only what is
NOT on GitHub: the starter backup, the music file, the handbook, the file list.
The program itself (garden.sh, the website, the guides, ...) is public code at
https://github.com/<repo>, so this fetches it from there, at ONE fixed version
(the commit written in CODE-SOURCE.txt), never "whatever is newest".

Every file is then checked against FILES-FROM-AUTHOR.txt, the list of
fingerprints (SHA-256) that came in the zip. Only when ALL of them match are
the files put in this folder; files that came in the zip are never replaced.
If anything does not match, nothing is changed.

    python3 tools/get-code.py            fetch, check, unpack (Start-Here-Ubuntu.sh runs this)
    python3 tools/get-code.py --check ARCHIVE   only check a downloaded archive (used by the release build)

Needs only Python 3, which Ubuntu has. No git, no extra programs.
"""
import hashlib, io, os, sys, tarfile, tempfile, urllib.request

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # the garden folder
CRLF_EXT = (".cmd", ".ps1", ".bat")      # Windows files get Windows line endings, like the zip had
NOT_CHECKED = {"DOWNLOADS-SHA256.txt"}  # cannot hold its own new fingerprint; the copy on GitHub is the one to read
ZIP_OWN = {"FILES-FROM-AUTHOR.txt"}     # the list itself always comes from the zip


def say(en, ko):
    print("   " + en)
    print("   " + ko)


def read_source():
    """CODE-SOURCE.txt: repo=owner/name and commit=<40 hex>."""
    values = {}
    for line in open(os.path.join(HERE, "CODE-SOURCE.txt"), encoding="utf-8"):
        if "=" in line and not line.startswith("#"):
            k, v = line.strip().split("=", 1)
            values[k.strip()] = v.strip()
    repo, commit = values.get("repo", ""), values.get("commit", "")
    if not repo or len(commit) != 40 or any(c not in "0123456789abcdef" for c in commit):
        sys.exit("CODE-SOURCE.txt is damaged (repo or commit missing). Download the zip again.")
    return repo, commit


def read_list():
    """FILES-FROM-AUTHOR.txt -> {relative path: sha256}."""
    want = {}
    for line in open(os.path.join(HERE, "FILES-FROM-AUTHOR.txt"), encoding="utf-8"):
        line = line.rstrip("\n")
        if len(line) > 68 and line[64:68] == "  ./":
            want[line[68:]] = line[:64]
    if len(want) < 100:
        sys.exit("FILES-FROM-AUTHOR.txt is damaged. Download the zip again.")
    return want


def normalize(rel, data):
    if rel.lower().endswith(CRLF_EXT):
        return data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    return data


def code_files(archive_bytes):
    """The archive's files as {relative path: bytes}, the top folder removed."""
    files = {}
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:gz") as t:
        for m in t.getmembers():
            if not m.isfile():
                continue
            parts = m.name.split("/", 1)
            if len(parts) < 2 or not parts[1] or ".." in parts[1].split("/") or parts[1].startswith("/"):
                continue
            files[parts[1]] = normalize(parts[1], t.extractfile(m).read())
    return files


def check(files, want, present):
    """Problems as a list of text lines (empty = everything matches)."""
    problems = []
    for rel, data in files.items():
        if rel in present or rel in NOT_CHECKED or rel in ZIP_OWN:
            continue                       # the zip's own copy is the one used
        if rel not in want:
            problems.append(f"not in the list: {rel}")
        elif hashlib.sha256(data).hexdigest() != want[rel]:
            problems.append(f"different from the list: {rel}")
    for rel in want:
        if rel not in files and rel not in present:
            problems.append(f"missing: {rel}")
    return problems


def zip_files_present(want):
    return {rel for rel in want if os.path.isfile(os.path.join(HERE, rel))}


def main():
    try:
        sys.stdout.reconfigure(errors="replace")   # a window that cannot show Korean shows ? instead of stopping
    except AttributeError:
        pass
    if len(sys.argv) == 3 and sys.argv[1] == "--check":
        want = read_list()
        problems = check(code_files(open(sys.argv[2], "rb").read()), want, zip_files_present(want))
        print("\n".join(problems[:40]) if problems else "ALL MATCH")
        sys.exit(1 if problems else 0)

    repo, commit = read_source()
    want = read_list()
    present = zip_files_present(want)
    # The files that came in the zip must themselves match the list.
    bad = [rel for rel in present if hashlib.sha256(open(os.path.join(HERE, rel), "rb").read()).hexdigest() != want[rel]]
    if bad:
        say(f"These files from the zip do not match their fingerprints: {', '.join(bad[:5])}",
            f"zip에서 나온 이 파일들의 지문이 맞지 않아요: {', '.join(bad[:5])}")
        sys.exit("Download the zip again. / zip을 다시 내려받으세요.")

    url = f"https://codeload.github.com/{repo}/tar.gz/{commit}"
    say(f"Getting the program from GitHub (version {commit[:12]}): {url}",
        f"GitHub에서 프로그램을 가져와요 (버전 {commit[:12]}).")
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "garden-get-code"})
        with urllib.request.urlopen(req, timeout=120) as r:
            archive = r.read()
    except Exception as e:
        say(f"Could not reach GitHub ({e.__class__.__name__}). Check the internet connection and run this again.",
            "GitHub에 연결할 수 없어요. 인터넷 연결을 확인하고 다시 실행하세요.")
        sys.exit(1)

    files = code_files(archive)
    problems = check(files, want, present)
    if problems:
        say("The program from GitHub does not match the file list, so nothing was changed:",
            "GitHub의 프로그램이 파일 목록과 맞지 않아서 아무것도 바꾸지 않았어요:")
        for p in problems[:20]:
            print("     " + p)
        sys.exit(1)

    # Everything matches: write the files (into a temporary name first, then into place).
    n = 0
    for rel, data in sorted(files.items()):
        if rel in present or rel in ZIP_OWN or (rel in NOT_CHECKED and os.path.exists(os.path.join(HERE, rel))):
            continue
        dst = os.path.join(HERE, *rel.split("/"))
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        fd, tmp = tempfile.mkstemp(dir=os.path.dirname(dst))
        with os.fdopen(fd, "wb") as f:
            f.write(data)
        os.chmod(tmp, 0o755 if rel.endswith(".sh") else 0o644)
        os.replace(tmp, dst)
        n += 1
    say(f"Done: {n} program files from GitHub, every fingerprint checked.",
        f"끝: GitHub에서 프로그램 파일 {n}개, 모든 지문을 확인했어요.")


if __name__ == "__main__":
    main()
