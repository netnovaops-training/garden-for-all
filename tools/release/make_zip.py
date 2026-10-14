"""Build Jun's three downloads (they go side by side in the Google Drive folder):

  junsgarden-for-jun.zip        ONLY what is not on GitHub: the starter backup
                                WITHOUT photos (plants database, comments, AI
                                cache), the music file, the handbook, the private
                                read-me, the file list, and the small starters
                                that fetch the program itself (tools/get-code.*)
  junsgarden-photos-addon.zip   the photos (images.tar), big, can be fetched again
  junsgarden-wiki-addon.zip     the wiki's pages (only if Jun wants the wiki)

The program (garden.sh, the website, the guides) is public code on
GitHub. The first start (Start-Here-Ubuntu.sh / Start-Here-Windows.cmd) fetches
it from there at ONE fixed version, GARDEN_CODE_COMMIT (written into
CODE-SOURCE.txt), and checks every file against FILES-FROM-AUTHOR.txt.

    GARDEN_CODE_COMMIT=<40-hex public commit> GARDEN_SRC=<release copy> python make_zip.py OUT.zip

- Text files get the right line endings (LF for Linux files, CRLF for .cmd/.ps1);
  .sh files are marked executable. tools/get-code.* do the same to what they
  fetch, so the fingerprints match.
- Writes FILES-FROM-AUTHOR.txt: sha256 of EVERY file of the finished folder (the
  program from GitHub plus the zip), exactly as it will be on disk, so Jun can
  check nothing was changed (garden.sh menu M, 11) and get-code refuses anything
  that does not match.
- The main zip carries the add-ons' fingerprints (docker/tools/photos.sha256,
  docker/wiki/seed/wiki-pages.sha256), so garden.sh refuses a wrong add-on.
- Skips local state: .env, local/, logs/, help/, .git, build output.

Test-only: SMALL_DIR swaps in a small backup and a small wiki seed, so a full
install test is quick. The program files are exactly the same.
"""
import hashlib, io, os, shutil, sys, tarfile, zipfile

# Run from anywhere:  python tools/release/make_zip.py [OUTPUT.zip]
# The three zips are written next to OUTPUT (default: next to this package folder).
# GARDEN_SRC: build from a COPY of the package (the original is then never written to;
# the fingerprint files below are saved in the copy).
SRC = os.environ.get("GARDEN_SRC") or os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(SRC), "junsgarden-for-jun.zip")
TAG = "-TEST-small" if "TEST" in os.path.basename(OUT) else ""
PHOTOS = os.path.join(os.path.dirname(OUT), f"junsgarden-photos-addon{TAG}.zip")
WIKI = os.path.join(os.path.dirname(OUT), f"junsgarden-wiki-addon{TAG}.zip")
TOP = "junsgarden-for-jun/"
SKIP_DIRS = {"target", ".git", "local", "logs", "help", "__pycache__", "node_modules", "from-garden-computer", "PDF-src", "PDF guides"}
SKIP_FILES = {".env", ".garden-remote", ".garden-target-windows.txt", ".garden-remote-windows.txt", ".garden-mode",
              ".garden-lang", ".garden-pdfs", ".garden-built", "MY-ADDRESSES.txt", "CHECKLIST.txt", "CHECKLIST-laptop.txt",
              "CHECKLIST-windows.txt", "checklist-history.log", "FILES-FROM-AUTHOR.txt",
              "junsgarden-wiki-addon.zip", "junsgarden-photos-addon.zip"}
TEXT_EXT = {".sh", ".md", ".yml", ".yaml", ".py", ".rs", ".html", ".css", ".js", ".json", ".toml",
            ".conf", ".example", ".txt", ".sql", ".lock", ".template", ".ini", ".cfg", ".csv", ".gitignore",
            ".gitattributes", ".dockerignore", ".svg", ".xml", ".j2", ".sha256"}
CRLF_EXT = {".cmd", ".ps1", ".bat"}
STORE_EXT = {".tar", ".apk", ".jpg", ".jpeg", ".png", ".webp", ".gif", ".mp3", ".mp4", ".gz", ".zip", ".pdf", ".woff", ".woff2"}
SMALL = os.environ.get("SMALL_DIR")
SEED_REL = "docker/wiki/seed/wiki-pages.json.gz"
SEED_SHA_REL = "docker/wiki/seed/wiki-pages.sha256"
PHOTOS_SHA_REL = "docker/tools/photos.sha256"
SEED_SRC = os.path.join(SMALL, "wiki-pages.json.gz") if SMALL else os.path.join(SRC, SEED_REL.replace("/", os.sep))
BACKUP_DIR = SMALL if SMALL else os.path.join(SRC, "backups")
STAMP = (2026, 10, 13, 0, 0, 0)
CODE_REPO = os.environ.get("GARDEN_CODE_REPO", "netnovaops-training/garden-for-all")
CODE_COMMIT = os.environ.get("GARDEN_CODE_COMMIT", "")
assert len(CODE_COMMIT) == 40 and all(c in "0123456789abcdef" for c in CODE_COMMIT), \
    "set GARDEN_CODE_COMMIT to the full public commit (40 hex characters) the download fetches"
# What goes in the main zip: what is NOT on GitHub, and the starters that fetch the rest.
NOT_ON_GITHUB = {"README.md", "site/Boogie.mp3", "00-WELCOME.html"}
STARTERS = {"Start-Here-Ubuntu.sh", "Start-Here-Windows.cmd", "Garden-Menu-Windows.cmd",
            "tools/get-code.py", "tools/get-code.ps1"}


def in_main_zip(rel):
    """True for what the main zip carries; everything else comes from GitHub."""
    return rel in NOT_ON_GITHUB or rel in STARTERS or rel.startswith("handbook/")


def content(full, name):
    data = open(full, "rb").read()
    ext = os.path.splitext(name)[1].lower()
    is_text = ext in TEXT_EXT or name in ("Dockerfile", ".gitignore", ".dockerignore", ".gitattributes") or name.startswith("Dockerfile")
    if ext in CRLF_EXT:
        data = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    elif is_text and b"\0" not in data[:4096]:
        data = data.replace(b"\r\n", b"\n")
    return data


def info_for(path, date_time=STAMP, mode=0o644):
    info = zipfile.ZipInfo(path, date_time=date_time)
    info.create_system = 3  # Unix, so the permission bits below are used
    info.external_attr = (mode | 0o100000) << 16
    return info


# ---- 1. Split the starter backup: photos out, everything else stays ----------
backup_tars = sorted(f for f in os.listdir(BACKUP_DIR) if f.startswith("junsgarden-backup_") and f.endswith(".tar"))
assert len(backup_tars) == 1, f"expected one starter backup in {BACKUP_DIR}, found {backup_tars}"
backup_name = backup_tars[0]
stem = backup_name[:-4]
src_tar = tarfile.open(os.path.join(BACKUP_DIR, backup_name))
# The starter backup in the download says where it came from without naming anyone.
out_stem = stem.replace("_from-jasons-server", "_from-the-author")
out_name = out_stem + ".tar"
pieces = {}
for m in src_tar.getmembers():
    if m.isfile() and not m.name.endswith("/images.tar"):
        pieces[os.path.basename(m.name)] = src_tar.extractfile(m).read()
images_member = src_tar.getmember(f"{stem}/images.tar")

# Photos add-on: images.tar streamed into its own zip, fingerprinted on the way.
if os.path.exists(PHOTOS):
    os.remove(PHOTOS)
h = hashlib.sha256()
with zipfile.ZipFile(PHOTOS, "w", zipfile.ZIP_STORED, allowZip64=True) as z:
    with src_tar.extractfile(images_member) as fin, z.open(info_for("images.tar"), "w", force_zip64=True) as fout:
        while True:
            block = fin.read(1 << 20)
            if not block:
                break
            h.update(block); fout.write(block)
    z.writestr(info_for("README.txt"), (
        "Jun's Garden: the photos (a separate download)\n\n"
        "Leave this file ZIPPED. Put it in your Home folder (Ubuntu) or on your Desktop (Windows).\n"
        "The install finds it by itself. Later, or to load the photos again: menu 6, then 11\n"
        "(or: bash garden.sh photos). Photos you already have are kept.\n"
        "The menu checks its fingerprint before using it.\n\n"
        "정원 사진(따로 내려받는 파일)\n\n"
        "이 파일은 압축을 풀지 마세요. Home 폴더(Ubuntu)나 바탕 화면(Windows)에 두세요.\n"
        "설치가 알아서 찾아요. 나중에, 또는 사진을 다시 넣으려면: 메뉴 6, 그다음 11\n"
        "(또는: bash garden.sh photos). 이미 있는 사진은 그대로 둬요.\n"
        "메뉴가 쓰기 전에 지문(fingerprint)을 확인해요.\n").encode("utf-8"))
photos_sha = h.hexdigest()

# The starter backup without photos: same pieces, new MANIFEST and SHA256SUMS.
manifest = pieces["MANIFEST.txt"].decode().replace(stem, out_stem).replace("Jason's server", "the author's server")
manifest = "\n".join(
    ("  images.tar        the photos: a separate download, junsgarden-photos-addon.zip"
     if line.strip().startswith("images.tar") else line) for line in manifest.split("\n"))
pieces["MANIFEST.txt"] = manifest.encode()
order = [p for p in ("database.sql.gz", "data.tar", "cashe.tar", "wiki.sql.gz", "n8n.tar", "auth.tar") if p in pieces]
sums = "".join(f"{hashlib.sha256(pieces[p]).hexdigest()}  {'' if p.endswith('.gz') else './'}{p}\n" for p in order)
sums += f"{hashlib.sha256(pieces['MANIFEST.txt']).hexdigest()}  MANIFEST.txt\n"
pieces["SHA256SUMS"] = sums.encode()
slim = io.BytesIO()
with tarfile.open(fileobj=slim, mode="w") as out:
    d = tarfile.TarInfo(out_stem); d.type = tarfile.DIRTYPE; d.mode = 0o755; out.addfile(d)
    for p in ["MANIFEST.txt", "SHA256SUMS"] + order:
        ti = tarfile.TarInfo(f"{out_stem}/{p}"); ti.size = len(pieces[p]); ti.mode = 0o644
        out.addfile(ti, io.BytesIO(pieces[p]))
slim_bytes = slim.getvalue()

# ---- 2. The wiki add-on ------------------------------------------------------
seed_bytes = open(SEED_SRC, "rb").read()
seed_sha = hashlib.sha256(seed_bytes).hexdigest()
if os.path.exists(WIKI):
    os.remove(WIKI)
with zipfile.ZipFile(WIKI, "w", zipfile.ZIP_STORED) as z:
    z.writestr(info_for("wiki-pages.json.gz"), seed_bytes)
    z.writestr(info_for("README.txt"), (
        "Jun's Garden: the wiki add-on (the garden's wiki pages)\n\n"
        "Only needed if you switch the wiki on. Leave this file ZIPPED and put it in your\n"
        "Home folder (Ubuntu) or on your Desktop (Windows). Then, in the menu: 9 (Parts) to\n"
        "switch the wiki on, or if it is already on:  bash garden.sh wiki\n"
        "The menu checks its fingerprint before using it.\n\n"
        "정원 위키 추가 파일(정원의 위키 페이지)\n\n"
        "위키를 켤 때만 필요해요. 이 파일은 압축을 풀지 말고 Home 폴더(Ubuntu)나\n"
        "바탕 화면(Windows)에 두세요. 그다음 메뉴에서: 9(Parts)로 위키를 켜거나,\n"
        "이미 켜져 있으면:  bash garden.sh wiki\n"
        "메뉴가 쓰기 전에 지문(fingerprint)을 확인해요.\n").encode("utf-8"))

# ---- 3. The main zip ---------------------------------------------------------
generated = {
    "CODE-SOURCE.txt": (
        "# Where the program comes from: the public code, at this one version.\n"
        "# tools/get-code.py (Ubuntu) and tools/get-code.ps1 (Windows) fetch it and\n"
        "# check every file against FILES-FROM-AUTHOR.txt. To read the same code:\n"
        f"# https://github.com/{CODE_REPO}/tree/{CODE_COMMIT}\n"
        f"repo={CODE_REPO}\ncommit={CODE_COMMIT}\n").encode(),
    SEED_SHA_REL: f"{seed_sha}  wiki-pages.json.gz\n".encode(),
    PHOTOS_SHA_REL: f"{photos_sha}  images.tar\n".encode(),
    f"backups/{out_name}": slim_bytes,
}
entries = []
for root, dirs, files in os.walk(SRC):
    dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
    for name in sorted(files):
        if name in SKIP_FILES or name.endswith(".env") or name.endswith(".pyc"):
            continue
        full = os.path.join(root, name)
        rel = os.path.relpath(full, SRC).replace(os.sep, "/")
        if rel.startswith(SEED_REL) or rel in generated:   # made below, or in the add-ons
            continue
        if rel.startswith("backups/") and name.endswith(".tar"):
            continue
        entries.append((rel, full, name))
if os.path.exists(OUT):
    os.remove(OUT)
lines = []
with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
    for rel, full, name in entries:
        data = content(full, name)
        lines.append(f"{hashlib.sha256(data).hexdigest()}  ./{rel}\n")   # listed: every file of the finished folder
        if not in_main_zip(rel):
            continue                                                     # ...but fetched from GitHub, not zipped
        info = info_for(TOP + rel, zipfile.ZipInfo.from_file(full).date_time, 0o755 if name.endswith(".sh") else 0o644)
        info.compress_type = zipfile.ZIP_STORED if os.path.splitext(name)[1].lower() in STORE_EXT else zipfile.ZIP_DEFLATED
        z.writestr(info, data)
    for rel, data in generated.items():
        lines.append(f"{hashlib.sha256(data).hexdigest()}  ./{rel}\n")
        z.writestr(info_for(TOP + rel), data)
    listing = "".join(sorted(lines, key=lambda l: l.split("  ", 1)[1])).encode()
    z.writestr(info_for(TOP + "FILES-FROM-AUTHOR.txt"), listing)

if not SMALL:
    # Keep the fingerprints next to the source too (git tracks them).
    open(os.path.join(SRC, "FILES-FROM-AUTHOR.txt"), "wb").write(listing)
    open(os.path.join(SRC, PHOTOS_SHA_REL.replace("/", os.sep)), "wb").write(generated[PHOTOS_SHA_REL])
    open(os.path.join(SRC, SEED_SHA_REL.replace("/", os.sep)), "wb").write(generated[SEED_SHA_REL])
for f in (OUT, PHOTOS, WIKI):
    print(f"{os.path.basename(f):40s} {os.path.getsize(f) / 1e6:8.1f} MB")
