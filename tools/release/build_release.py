"""Build a release of Jun's Garden (the original package is never written to).

    python tools/release/build_release.py OUT_FOLDER EMBLEM.png --ref main | --commit <40-hex public commit>
                                          [--repo owner/name] [--allow-differences]
                                          [--pdf-english FOLDER --pdf-korean FOLDER]

The program is public code on GitHub; the download carries only what is not
there. So the release is built from exactly what Jun will get:

1. Downloads the public code at ONE commit (--commit, on --repo, default
   netnovaops-training/garden-for-all) and unpacks it into
   OUT_FOLDER/build/junsgarden-for-jun, with the same line endings the fetch
   scripts give it (tools/get-code.*).
2. Checks it is the same as this package's tracked files (apart from what is
   kept off GitHub on purpose: README.md, site/Boogie.mp3, and the two
   fingerprint lists). A difference stops the build: the commit is not the one
   that matches this package. --allow-differences only for tests.
3. Adds what is not on GitHub: the private read-me, the music file, the
   starters (Start-Here-*, Garden-Menu-Windows.cmd, tools/get-code.*), the
   starter backup and the wiki pages (kept out of git on purpose).
4. Builds the handbook into it (00-WELCOME.html, handbook/), with the emblem as
   its logo (the emblem is never kept in git).
5. Makes the three downloads (make_zip.py): the main zip holds only what is not
   on GitHub, plus CODE-SOURCE.txt and FILES-FROM-AUTHOR.txt.
   With --commit, CODE-SOURCE.txt names that commit (fixed version) and the list
   holds the fingerprint of every file of the finished folder.
   With --ref main (what new zips use), main is first looked up on GitHub and
   resolved to its current commit (that commit is what is compared and tested,
   and the build stops if main moves meanwhile); CODE-SOURCE.txt says ref=main and
   FILES-FROM-AUTHOR.txt lists only what is not on GitHub: the program itself is
   checked on Jun's computer against the public CODE-FINGERPRINTS.txt.
6. Checks the main zip the way Jun's computer will: tools/get-code.py --check
   against the downloaded code. It must say ALL MATCH.
FILES-FROM-AUTHOR.txt (and the add-on fingerprints) are written in the copy;
copy FILES-FROM-AUTHOR.txt into the package for the final commit.
The PDF guides are NOT part of the program or of the Drive downloads: they are two
zips attached to a GitHub release (tools/release/make_pdf_zips.py), fetched by the
installer (tools/get-pdfs.py). With --pdf-english / --pdf-korean (the folders the PDFs
were built in) step 5b builds those two zips into OUT_FOLDER/pdf, prints their SHA-256
and checks they are the ones PDF-GUIDES-SHA256.txt (in the public code) names.
"""
import argparse, hashlib, io, os, pathlib, shutil, subprocess, sys, tarfile, urllib.request, zipfile

HERE = pathlib.Path(__file__).resolve().parent
PKG = HERE.parents[1]
ap = argparse.ArgumentParser()
ap.add_argument("out")
ap.add_argument("logo", nargs="?", default="")
ap.add_argument("--commit", default="", help="a fixed version: the 40-character public commit")
ap.add_argument("--ref", default="", help="follow a branch (main): resolved to its current commit for the build; the zip says ref=main")
ap.add_argument("--repo", default="netnovaops-training/garden-for-all")
ap.add_argument("--allow-differences", action="store_true")
ap.add_argument("--pdf-english", default="")
ap.add_argument("--pdf-korean", default="")
ap.add_argument("--photos-key", default="", help="photos.key (made outside every repo); goes only into the main zip")
ap.add_argument("--photos-token", default="", help="photos-access.token (read-only token of the private backup repository); goes only into the main zip")
ap.add_argument("--photo-parts", default="", help="PHOTO-PARTS.json from make_photo_parts.py; goes into the main zip, not into the public code")
A = ap.parse_args()
if bool(A.commit) == bool(A.ref):
    sys.exit("give exactly one of --ref main or --commit <40 hex>")
OUT = pathlib.Path(A.out).resolve()
STAGE = OUT / "build" / "junsgarden-for-jun"
CRLF_EXT = (".cmd", ".ps1", ".bat")
sys.path.insert(0, str(HERE))
from make_fingerprints import OFF_GITHUB      # the files kept off GitHub on purpose (never on CODE-FINGERPRINTS.txt)
STARTERS = ["Start-Here-Ubuntu.sh", "Start-Here-Windows.cmd", "Garden-Menu-Windows.cmd",
            "tools/get-code.py", "tools/get-code.ps1"]


def run(args, **env):
    print("  $", " ".join(str(a) for a in args))
    subprocess.run(args, check=True, env={**os.environ, **{k: str(v) for k, v in env.items()}})


def normalized(rel, data):
    if rel.lower().endswith(CRLF_EXT):
        return data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    return data


if A.ref:
    # Follow-main mode: look up where the branch is now and build from exactly that commit.
    def resolve(ref):
        r = subprocess.run(["git", "ls-remote", f"https://github.com/{A.repo}.git", f"refs/heads/{ref}"], capture_output=True, text=True, timeout=120)
        out = r.stdout.split()
        if r.returncode != 0 or len(out) != 2 or len(out[0]) != 40:
            sys.exit(f"STOP: could not find branch {ref} of {A.repo} on GitHub ({r.stderr.strip()[:200]})")
        return out[0]
    A.commit = resolve(A.ref)
    print(f"0. {A.ref} of {A.repo} is at {A.commit}")
print(f"1. The public code at {A.commit[:12]} ({A.repo})")
assert len(A.commit) == 40 and all(c in "0123456789abcdef" for c in A.commit), "--commit must be the full 40-character commit"
if STAGE.exists():
    shutil.rmtree(STAGE)
url = f"https://codeload.github.com/{A.repo}/tar.gz/{A.commit}"
archive = urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "garden-release"}), timeout=300).read()
(OUT / "build").mkdir(parents=True, exist_ok=True)
(OUT / "build" / "code.tar.gz").write_bytes(archive)
code = {}
with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as t:
    for m in t.getmembers():
        if m.isfile() and "/" in m.name:
            rel = m.name.split("/", 1)[1]
            code[rel] = normalized(rel, t.extractfile(m).read())
for rel, data in code.items():
    dst = STAGE / rel
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(data)
print(f"   {len(code)} files")

print("2. The same as this package?")
tracked = [t for t in subprocess.run(["git", "-C", str(PKG), "-c", "core.quotepath=off", "ls-files", "-z"],
                                     capture_output=True, check=True).stdout.decode("utf-8").split("\0") if t]
diffs = []
for rel in tracked:
    if rel in OFF_GITHUB:
        continue
    mine = normalized(rel, (PKG / rel).read_bytes()) if (PKG / rel).is_file() else None
    if rel not in code:
        diffs.append(f"only here: {rel}")
    elif mine is not None and hashlib.sha256(mine).digest() != hashlib.sha256(code[rel]).digest():
        diffs.append(f"different: {rel}")
diffs += [f"only on GitHub: {rel}" for rel in code if rel not in tracked and rel not in OFF_GITHUB]
if diffs:
    print("   " + "\n   ".join(diffs[:30]) + (f"\n   ... and {len(diffs) - 30} more" if len(diffs) > 30 else ""))
    if not A.allow_differences:
        sys.exit("STOP: that commit is not the code of this package. Push the package's code first, then build with that commit.")
    print("   (allowed: test build)")
else:
    print("   yes, every file")

if A.photos_key and A.photo_parts:      # the key and the list must belong together (the key id is in the list)
    import json
    kid = pathlib.Path(A.photos_key).read_text(encoding="utf-8").splitlines()[1][3:]
    if json.loads(pathlib.Path(A.photo_parts).read_text(encoding="utf-8"))["encryption"]["key_id"] != kid:
        sys.exit("STOP: photos.key is not the key of this PHOTO-PARTS.json (the key ids differ).")
for leak in ("photos.key", "photos-access.token", "PHOTO-PARTS.json"):
    if leak in code:
        sys.exit(f"STOP: {leak} is in the public code. It must not be: remove it from git.")
print("3. What is not on GitHub")
for rel in ["README.md", "site/Boogie.mp3"] + STARTERS:
    (STAGE / rel).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(PKG / rel, STAGE / rel)
data = [p for p in list((PKG / "backups").glob("junsgarden-backup_*.tar")) +
        list((PKG / "docker" / "wiki" / "seed").glob("*.json.gz"))]
for p in data:
    dst = STAGE / p.relative_to(PKG)
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(p, dst)
if A.photo_parts:
    shutil.copy2(A.photo_parts, STAGE / "PHOTO-PARTS.json")
print(f"   {2 + len(STARTERS)} files and {len(data)} data files" + (" and PHOTO-PARTS.json" if A.photo_parts else ""))

print("4. The handbook")
run([sys.executable, HERE / "make_handbook.py", STAGE], GARDEN_LOGO=A.logo)

print("5. The three downloads")
run([sys.executable, HERE / "make_zip.py", OUT / "junsgarden-for-jun.zip"],
    GARDEN_SRC=STAGE, GARDEN_CODE_COMMIT="" if A.ref else A.commit, GARDEN_CODE_REF=A.ref, GARDEN_CODE_REPO=A.repo, GARDEN_PHOTOS_KEY=A.photos_key, GARDEN_PHOTOS_TOKEN=A.photos_token)
# The add-on fingerprints the zip carries must be the ones the public code carries too.
for rel in ("docker/tools/photos.sha256", "docker/wiki/seed/wiki-pages.sha256"):
    if (STAGE / rel).read_bytes() != code.get(rel):
        print(f"   DIFFERENT from the public code: {rel}")
        if not A.allow_differences:
            sys.exit("STOP: the add-ons built now are not the ones the public code names. Fix the public code first.")

if A.pdf_english and A.pdf_korean:
    print("5b. The two PDF guide downloads (for the GitHub release; not on Drive)")
    run([sys.executable, HERE / "make_pdf_zips.py", OUT / "pdf", "--english", A.pdf_english, "--korean", A.pdf_korean])
    listed = code.get("PDF-GUIDES-SHA256.txt", b"").decode("utf-8")
    for z in sorted((OUT / "pdf").glob("*.zip")):
        sha = hashlib.sha256(z.read_bytes()).hexdigest()
        if f"{sha}  {z.name} {z.stat().st_size} " not in listed:
            print(f"   NOT the one PDF-GUIDES-SHA256.txt names: {z.name}")
            if not A.allow_differences:
                sys.exit("STOP: the PDF zips do not match the list in the public code.")
        else:
            print(f"   {z.name}: matches the list")

print("6. The main zip checked the way Jun's computer will check it")
check = OUT / "check"
if check.exists():
    shutil.rmtree(check)
with zipfile.ZipFile(OUT / "junsgarden-for-jun.zip") as z:
    z.extractall(check)
r = subprocess.run([sys.executable, check / "junsgarden-for-jun" / "tools" / "get-code.py", "--check",
                    OUT / "build" / "code.tar.gz"], capture_output=True, text=True)
print("   " + r.stdout.strip().replace("\n", "\n   "))
shutil.rmtree(check)
if r.returncode != 0 and not A.allow_differences:
    sys.exit("STOP: the download would not install (see above).")
if A.ref:
    now = resolve(A.ref)
    if now != A.commit:
        sys.exit(f"STOP: {A.ref} moved while the release was built ({A.commit[:12]} -> {now[:12]}). Build again.")
    print(f"   {A.ref} is still at {A.commit}")
print(f"done: {OUT}")
