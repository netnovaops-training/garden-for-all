#!/usr/bin/env python3
"""Get the garden's program files from GitHub and check every one (Ubuntu).

The download from Google Drive (junsgarden-for-jun.zip) carries only what is
NOT on GitHub: the starter backup, the music file, the handbook, the file list.
The program itself (garden.sh, the website, the guides, ...) is public code at
https://github.com/<repo>, so this fetches it from there.

CODE-SOURCE.txt says which version:
  ref=main        follow the public page's main branch (what new zips say). Every
                  file is checked against CODE-FINGERPRINTS.txt, the PUBLIC list
                  that main carries; a file that differs, is missing or is extra
                  (not on the list) stops everything. The zip's FILES-FROM-AUTHOR.txt
                  then covers only the files that are not on GitHub.
  commit=<40 hex> (or ref=<40 hex>) ONE fixed version, never "whatever is newest":
                  the way to stay put. A version from before CODE-FINGERPRINTS.txt
                  existed is checked file by file against FILES-FROM-AUTHOR.txt (the
                  old zips); a newer one against its CODE-FINGERPRINTS.txt.

Only when ALL files match are they put in this folder; files that came in the
zip are never replaced. If anything does not match, nothing is changed.

    python3 tools/get-code.py            fetch, check, unpack (Start-Here-Ubuntu.sh runs this)
    python3 tools/get-code.py --check ARCHIVE   only check a downloaded archive (used by the release build)

Needs only Python 3, which Ubuntu has. No git, no extra programs.
"""
import hashlib, io, os, re, sys, tarfile, tempfile, urllib.request

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # the garden folder
CRLF_EXT = (".cmd", ".ps1", ".bat")      # Windows files get Windows line endings, like the zip had
NOT_CHECKED = {"DOWNLOADS-SHA256.txt"}  # cannot hold its own new fingerprint; the copy on GitHub is the one to read
ZIP_OWN = {"FILES-FROM-AUTHOR.txt"}     # the list itself always comes from the zip
SETTINGS = {"CODE-SOURCE.txt"}          # the person's own choice of version; may be edited (guide 18), so a changed copy is not an error
LIST_NAME = "CODE-FINGERPRINTS.txt"     # the public list of the program's fingerprints (on GitHub, in the download)
# Public but not on that list (the private copies come in the zip); never written from GitHub in main mode.
NOT_LISTED_OK = {"README.md", "FILES-FROM-AUTHOR.txt", "DOWNLOADS-SHA256.txt"}
# Test-only: where the download comes from (a local test server). Real runs never set it.
BASE = os.environ.get("GARDEN_CODE_BASE", "https://codeload.github.com").rstrip("/")


def say(en, ko):
    print("   " + en)
    print("   " + ko)


def read_source():
    """CODE-SOURCE.txt -> (repo, ref, fixed). `ref=main` (or another branch name) or a 40-hex commit
    (`commit=<hex>`, the old one-line format, or `ref=<hex>`). fixed is True for a commit."""
    values = {}
    for line in open(os.path.join(HERE, "CODE-SOURCE.txt"), encoding="utf-8"):
        if "=" in line and not line.startswith("#"):
            k, v = line.strip().split("=", 1)
            values[k.strip()] = v.strip()
    repo, commit, ref = values.get("repo", ""), values.get("commit", ""), values.get("ref", "")
    if not re.fullmatch(r"[A-Za-z0-9._-]+/[A-Za-z0-9._-]+", repo) or (commit and ref):
        sys.exit("CODE-SOURCE.txt is damaged (repo, or both commit and ref). Download the zip again.")
    pick = commit or ref
    if re.fullmatch(r"[0-9a-f]{40}", pick):
        return repo, pick, True
    if commit or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", pick) or ".." in pick:
        sys.exit("CODE-SOURCE.txt is damaged (the version is neither ref=main nor a 40-character commit). Download the zip again.")
    return repo, pick, False


def read_list(minimum=100):
    """FILES-FROM-AUTHOR.txt -> {relative path: sha256}."""
    want = {}
    for line in open(os.path.join(HERE, "FILES-FROM-AUTHOR.txt"), encoding="utf-8"):
        line = line.rstrip("\n")
        if len(line) > 68 and line[64:68] == "  ./":
            want[line[68:]] = line[:64]
    if len(want) < minimum:
        sys.exit("FILES-FROM-AUTHOR.txt is damaged. Download the zip again.")
    return want


def normalize(rel, data):
    if rel.lower().endswith(CRLF_EXT):
        return data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    return data


def code_files(archive_bytes, odd=None):
    """The archive's files as {relative path: bytes}, the top folder removed.
    If `odd` is a list, anything strange is added to it as text (a link or device, a path that
    leaves the folder, a second top folder, the same file twice) instead of being skipped."""
    files, tops = {}, set()
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:gz") as t:
        for m in t.getmembers():
            parts = m.name.split("/", 1)
            tops.add(parts[0])
            if not m.isfile():
                if odd is not None and not m.isdir():
                    odd.append(f"not a plain file: {m.name}")
                continue
            if len(parts) < 2 or not parts[1] or ".." in parts[1].split("/") or parts[1].startswith("/"):
                if odd is not None:
                    odd.append(f"strange path: {m.name}")
                continue
            if odd is not None and parts[1] in files:
                odd.append(f"twice: {parts[1]}")
            files[parts[1]] = normalize(parts[1], t.extractfile(m).read())
    if odd is not None and len(tops) > 1:
        odd.append("more than one top folder in the download")
    return files, tops


def check(files, want, present):
    """Problems as a list of text lines (empty = everything matches). The old way: every file against FILES-FROM-AUTHOR.txt."""
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


def read_fingerprints(files):
    """CODE-FINGERPRINTS.txt from the download -> ({path: sha256}, problems). Same line shape as FILES-FROM-AUTHOR.txt,
    but without the leading ./ ("<sha256>  <path>")."""
    raw = files.get(LIST_NAME)
    if raw is None:
        return {}, [f"{LIST_NAME} is not in the download"]
    fp, problems = {}, []
    for n, line in enumerate(raw.decode("utf-8", "replace").split(chr(10)), 1):
        line = line.rstrip(chr(13))
        if not line:
            continue
        if len(line) < 67 or line[64:66] != "  " or any(c not in "0123456789abcdef" for c in line[:64]):
            problems.append(f"{LIST_NAME} line {n} is not a fingerprint line")
            continue
        rel = line[66:]
        if rel in fp or rel.startswith("/") or ".." in rel.split("/") or rel in (LIST_NAME,):
            problems.append(f"{LIST_NAME} line {n}: unusable path")
            continue
        fp[rel] = line[:64]
    if not fp and not problems:
        problems.append(f"{LIST_NAME} is empty")
    return fp, problems


def check_listed(files, fp):
    """The new way: the download against its own public list. Every file must be on the list with the right
    fingerprint, every file on the list must be there, and nothing else may be there."""
    problems = []
    for rel, data in files.items():
        if rel == LIST_NAME or rel in NOT_LISTED_OK:
            continue
        if rel not in fp:
            problems.append(f"extra, not on the list: {rel}")
        elif hashlib.sha256(data).hexdigest() != fp[rel]:
            problems.append(f"different from the list: {rel}")
    for rel in fp:
        if rel not in files:
            problems.append(f"missing: {rel}")
    return problems


def zip_files_present(want):
    return {rel for rel in want if os.path.isfile(os.path.join(HERE, rel))}


def verify(files, odd, want, present, fixed):
    """-> (problems, listed). `listed` is True when the download was checked against its own public list
    (CODE-FINGERPRINTS.txt), False when against FILES-FROM-AUTHOR.txt (the old way, for old fixed versions)."""
    if fixed and LIST_NAME not in files:
        if len(want) < 100:        # a zip that follows main lists only its own files: it cannot vouch for an older version
            say("That version of the program has no CODE-FINGERPRINTS.txt, and the file list that came in this zip",
                "그 버전의 프로그램에는 CODE-FINGERPRINTS.txt가 없고, 이 zip에 들어 있는 파일 목록으로는")
            say("does not cover it, so nothing was changed. Choose a version that has CODE-FINGERPRINTS.txt (guide 18).",
                "확인할 수 없어서 아무것도 바꾸지 않았어요. CODE-FINGERPRINTS.txt가 있는 버전을 고르세요(안내서 18).")
            sys.exit(1)
        return check(files, want, present), False
    fp, problems = read_fingerprints(files)
    if not problems:
        problems = check_listed(files, fp)
    return odd + problems, True


def check_archive(path):
    """Problems (empty = ALL MATCH) of a downloaded archive, checked the way the first start checks it.
    Used by `--check` and by the release build."""
    _, _, fixed = read_source()
    want = read_list(5)
    odd = []
    files, _tops = code_files(open(path, "rb").read(), odd)
    return verify(files, odd, want, zip_files_present(want), fixed)[0]


def main():
    try:
        sys.stdout.reconfigure(errors="replace")   # a window that cannot show Korean shows ? instead of stopping
    except AttributeError:
        pass
    if len(sys.argv) == 3 and sys.argv[1] == "--check":
        problems = check_archive(sys.argv[2])
        print(chr(10).join(problems[:40]) if problems else "ALL MATCH")
        sys.exit(1 if problems else 0)

    repo, ref, fixed = read_source()
    want = read_list(5)
    present = zip_files_present(want)
    # The files that came in the zip must themselves match the list.
    bad = [rel for rel in present if rel not in SETTINGS and hashlib.sha256(open(os.path.join(HERE, rel), "rb").read()).hexdigest() != want[rel]]
    if bad:
        say(f"These files from the zip do not match their fingerprints: {', '.join(bad[:5])}",
            f"zip에서 나온 이 파일들의 지문이 맞지 않아요: {', '.join(bad[:5])}")
        sys.exit("Download the zip again. / zip을 다시 내려받으세요.")

    url = f"{BASE}/{repo}/tar.gz/{ref}"
    if fixed:
        say(f"Getting the program from GitHub (version {ref[:12]}): {url}",
            f"GitHub에서 프로그램을 가져와요 (버전 {ref[:12]}).")
    else:
        say(f"Getting the program from GitHub ({ref} branch): {url}",
            f"GitHub에서 프로그램을 가져와요 ({ref} 브랜치).")
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "garden-get-code"})
        with urllib.request.urlopen(req, timeout=120) as r:
            archive = r.read()
    except Exception as e:
        say(f"Could not reach GitHub ({e.__class__.__name__}). Check the internet connection and run this again.",
            "GitHub에 연결할 수 없어요. 인터넷 연결을 확인하고 다시 실행하세요.")
        sys.exit(1)

    odd = []
    try:
        files, _tops = code_files(archive, odd)
    except (tarfile.TarError, EOFError, OSError) as e:
        files, odd = {}, [f"the download could not be read ({e.__class__.__name__})"]
    problems, listed = verify(files, odd, want, present, fixed) if files else (odd, True)
    if problems:
        if listed:
            say("The program on GitHub is being updated, or it does not match its own list of fingerprints.",
                "GitHub의 프로그램이 업데이트되는 중이거나, 자체 지문 목록과 맞지 않아요.")
            say("Nothing was changed. Try again in a few minutes. If it keeps happening, do not go on: tell the author.",
                "아무것도 바꾸지 않았어요. 몇 분 뒤에 다시 해 보세요. 계속 그러면 더 진행하지 말고 제작자에게 알려 주세요.")
            say("(Guide 18 shows how to stay on one fixed version of the program.)",
                "(18번 안내서에 프로그램의 한 버전에 머무는 방법이 있어요.)")
        else:
            say("The program from GitHub does not match the file list, so nothing was changed:",
                "GitHub의 프로그램이 파일 목록과 맞지 않아서 아무것도 바꾸지 않았어요:")
        for p in problems[:20]:
            print("     " + p)
        sys.exit(1)

    # Everything matches: write the files (into a temporary name first, then into place).
    n = 0
    for rel, data in sorted(files.items()):
        if rel in present or rel in ZIP_OWN:
            continue
        if listed and rel in NOT_LISTED_OK:
            continue                        # public, but not on the list: the zip carries the real ones
        if not listed and rel in NOT_CHECKED and os.path.exists(os.path.join(HERE, rel)):
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
