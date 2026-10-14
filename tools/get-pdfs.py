#!/usr/bin/env python3
"""Get the garden's PDF guides (an optional, separate download) and check them.

The PDF guides are not part of the program: they are two small downloads, one
per language, attached to a release on GitHub. Each one is checked against the
fingerprint (SHA-256) and size in PDF-GUIDES-SHA256.txt, which is part of the
program. Only when the fingerprint matches and every name inside is a plain
"<language>/<step folder>/<name>.pdf" are the files put in the folder
"PDF guides/<language>"; otherwise NOTHING is written.

    python3 tools/get-pdfs.py [en|ko|both]     fetch, check, unpack (default: en)
        --dest FOLDER        where "PDF guides" goes (default: the garden folder)
        --url-base URL       another place for the zips (a test; file:// works)
        --check ZIP en|ko    only check a zip against the list (used by the release build)

The garden runs without them. A failure is not an install failure: it prints
what happened in plain words and exits 1; the garden menu (P) tries again.
Needs only Python 3. No git, no extra programs.
"""
import hashlib, os, shutil, sys, tempfile, urllib.request, zipfile

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # the garden folder
LANGS = {"en": "English", "ko": "Korean"}


def say(en, ko):
    print("   " + en)
    print("   " + ko)


def read_list(path):
    """PDF-GUIDES-SHA256.txt -> (url base, {lang code: (file name, sha256, bytes)})."""
    base, files = "", {}
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if line.startswith("url="):
            base = line[4:].strip().rstrip("/")
        elif len(line) > 66 and line[64:66] == "  " and not line.startswith("#"):
            sha, name, size, code = (line[:64], *line[66:].split())[:4]
            files[code] = (name, sha, int(size))
    if not base or set(files) != set(LANGS):
        sys.exit("PDF-GUIDES-SHA256.txt is damaged. / PDF-GUIDES-SHA256.txt 파일이 손상됐어요.")
    return base, files


def problems_in(zip_path, folder, want):
    """Reasons this zip must not be used (empty = fine)."""
    name, sha, size = want
    bad = []
    if os.path.getsize(zip_path) != size:
        bad.append("wrong size")
    h = hashlib.sha256()
    with open(zip_path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    if h.hexdigest() != sha:
        bad.append("fingerprint does not match")
    if bad:
        return bad
    try:
        with zipfile.ZipFile(zip_path) as z:
            names = [i.filename for i in z.infolist() if not i.is_dir()]
            for n in names:
                parts = n.split("/")
                if (n.startswith("/") or "\\" in n or ".." in parts or parts[0] != folder
                        or len(parts) < 3 or not n.lower().endswith(".pdf")):
                    bad.append("unexpected file name inside")
                    break
            if not names:
                bad.append("empty")
            elif not bad and z.testzip() is not None:
                bad.append("damaged zip")
    except zipfile.BadZipFile:
        bad.append("not a zip")
    return bad


MENU_HINT = "use menu item P to get them"


def set_status(dest, folder, state, detail=""):
    """Keep "PDF guides/STATUS.txt" (one line per language) and a small placeholder note in
    the language folder: _arriving.txt while the download runs; it is removed when the set is in
    (or replaced by _not-downloaded.txt). These are status notes only, never part of the check."""
    root = os.path.join(dest, "PDF guides")
    os.makedirs(root, exist_ok=True)
    text = {"arriving": "arriving (about a minute)", "ready": f"ready ({detail})",
            "failed": f"not downloaded - {MENU_HINT}", "none": f"not downloaded - {MENU_HINT}"}[state]
    lines = {}
    path = os.path.join(root, "STATUS.txt")
    if os.path.isfile(path):
        for ln in open(path, encoding="utf-8").read().splitlines():
            if ": " in ln and ln.split(": ", 1)[0] in LANGS.values():
                lines[ln.split(": ", 1)[0]] = ln.split(": ", 1)[1]
    if state == "none" and lines.get(folder, "").startswith("ready"):
        return                              # a set that is already in stays "ready"
    lines[folder] = text
    body = "".join(f"{k}: {lines[k]}\n" for k in ("English", "Korean") if k in lines)
    note = {"arriving": "The PDF guides are arriving, about a minute. This file is replaced when they are in.\n",
            "failed": f"The PDF guides were not downloaded. The garden works without them. {MENU_HINT.capitalize()}.\n"}
    d = os.path.join(root, folder)
    if state in note:
        os.makedirs(d, exist_ok=True)
    for name, keep in (("_arriving.txt", state == "arriving"), ("_not-downloaded.txt", state == "failed")):
        f = os.path.join(d, name)
        if keep:
            tmp = f + ".part"
            open(tmp, "w", encoding="utf-8", newline="\n").write(note[state])
            os.replace(tmp, f)
        elif os.path.isfile(f):
            os.remove(f)
    tmp = path + ".part"
    open(tmp, "w", encoding="utf-8", newline="\n").write(body)
    os.replace(tmp, path)


def fetch(url, dst, label):
    req = urllib.request.Request(url, headers={"User-Agent": "garden-get-pdfs"})
    with urllib.request.urlopen(req, timeout=120) as r, open(dst, "wb") as out:
        total = int(r.headers.get("Content-Length") or 0)
        got, shown = 0, -1
        while True:
            block = r.read(1 << 20)
            if not block:
                break
            out.write(block)
            got += len(block)
            pct = int(got * 100 / total) // 25 * 25 if total else -1
            if total and pct != shown:
                shown = pct
                print(f"   {label}: {got // (1 << 20)} / {total // (1 << 20)} MB")


def install(zip_path, folder, dest):
    """Unpack into dest/PDF guides/<folder>; each file lands whole (temporary name, then in place)."""
    root = os.path.join(dest, "PDF guides")
    n = 0
    with zipfile.ZipFile(zip_path) as z:
        for info in z.infolist():
            if info.is_dir():
                continue
            target = os.path.join(root, *info.filename.split("/"))
            os.makedirs(os.path.dirname(target), exist_ok=True)
            fd, tmp = tempfile.mkstemp(dir=os.path.dirname(target))
            with os.fdopen(fd, "wb") as f, z.open(info) as src:
                shutil.copyfileobj(src, f)
            os.chmod(tmp, 0o644)
            os.replace(tmp, target)
            n += 1
    return n


def main(argv):
    try:
        sys.stdout.reconfigure(errors="replace")
    except AttributeError:
        pass
    args, opts = [], {}
    it = iter(argv)
    for a in it:
        if a in ("--dest", "--url-base", "--list"):
            opts[a] = next(it, "")
        elif a == "--check":
            opts[a] = (next(it, ""), next(it, ""))
        else:
            args.append(a)
    listfile = opts.get("--list") or os.path.join(HERE, "PDF-GUIDES-SHA256.txt")
    base, files = read_list(listfile)
    if "--check" in opts:
        zp, code = opts["--check"]
        bad = problems_in(zp, LANGS[code], files[code])
        print("ALL MATCH" if not bad else "; ".join(bad))
        return 1 if bad else 0

    pick = (args[0] if args else "en").lower()
    dest = opts.get("--dest") or HERE
    if pick in ("none", "no"):
        for folder in LANGS.values():
            set_status(dest, folder, "none")
        return 0
    codes = ["en", "ko"] if pick == "both" else [pick]
    if any(c not in LANGS for c in codes):
        print("Choose en, ko or both. / en, ko, both 중에서 고르세요.")
        return 1
    for c in LANGS:
        if c not in codes:
            set_status(dest, LANGS[c], "none")
    base = (opts.get("--url-base") or base).rstrip("/")
    failed = False
    with tempfile.TemporaryDirectory(prefix="garden-pdfs-") as tmp:
        for code in codes:
            name, sha, size = files[code]
            folder = LANGS[code]
            say(f"PDF guides arriving ({folder}, {size // (1 << 20)} MB)...", f"PDF 안내서를 받고 있어요 ({folder}, {size // (1 << 20)} MB)...")
            zp = os.path.join(tmp, name)
            set_status(dest, folder, "arriving")
            try:
                fetch(f"{base}/{name}", zp, folder)
            except Exception as e:
                say(f"Could not download the {folder} PDF guides ({e.__class__.__name__}). Nothing was written. "
                    "The garden works without them; try again later with menu P.",
                    f"{folder} PDF 안내서를 받지 못했어요. 아무것도 쓰지 않았어요. 안내서 없이도 정원은 잘 돌아가요. 나중에 메뉴 P로 다시 해 보세요.")
                failed = True
                set_status(dest, folder, "failed")
                continue
            bad = problems_in(zp, folder, files[code])
            if bad:
                say(f"The {folder} PDF guides did not pass the check ({'; '.join(bad)}), so nothing was written. "
                    "Try again later with menu P.",
                    f"{folder} PDF 안내서가 확인을 통과하지 못해서 아무것도 쓰지 않았어요. 나중에 메뉴 P로 다시 해 보세요.")
                failed = True
                set_status(dest, folder, "failed")
                continue
            n = install(zp, folder, dest)
            set_status(dest, folder, "ready", f"{n} PDFs")
            say(f"Done: {n} {folder} PDF guides in the folder \"PDF guides/{folder}\", fingerprint checked.",
                f"끝: \"PDF guides/{folder}\" 폴더에 PDF {n}개, 지문을 확인했어요.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
