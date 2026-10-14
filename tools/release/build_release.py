"""Build a release of Jun's Garden (the original package is never written to).

    python tools/release/build_release.py OUT_FOLDER EMBLEM.png --commit <40-hex public commit>
                                          [--repo owner/name] [--allow-differences]

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
   on GitHub, plus CODE-SOURCE.txt (the commit) and FILES-FROM-AUTHOR.txt (the
   fingerprint of every file of the finished folder).
6. Checks the main zip the way Jun's computer will: tools/get-code.py --check
   against the downloaded code. It must say ALL MATCH.
FILES-FROM-AUTHOR.txt (and the add-on fingerprints) are written in the copy;
copy FILES-FROM-AUTHOR.txt into the package for the final commit.
The PDFs are part of the program ("PDF guides/", English and Korean), so there
is no separate PDF upload any more.
"""
import argparse, hashlib, io, os, pathlib, shutil, subprocess, sys, tarfile, urllib.request, zipfile

HERE = pathlib.Path(__file__).resolve().parent
PKG = HERE.parents[1]
ap = argparse.ArgumentParser()
ap.add_argument("out")
ap.add_argument("logo", nargs="?", default="")
ap.add_argument("--commit", required=True)
ap.add_argument("--repo", default="netnovaops-training/garden-for-all")
ap.add_argument("--allow-differences", action="store_true")
A = ap.parse_args()
OUT = pathlib.Path(A.out).resolve()
STAGE = OUT / "build" / "junsgarden-for-jun"
CRLF_EXT = (".cmd", ".ps1", ".bat")
OFF_GITHUB = {"README.md", "site/Boogie.mp3", "FILES-FROM-AUTHOR.txt", "DOWNLOADS-SHA256.txt"}
STARTERS = ["Start-Here-Ubuntu.sh", "Start-Here-Windows.cmd", "Garden-Menu-Windows.cmd",
            "tools/get-code.py", "tools/get-code.ps1"]


def run(args, **env):
    print("  $", " ".join(str(a) for a in args))
    subprocess.run(args, check=True, env={**os.environ, **{k: str(v) for k, v in env.items()}})


def normalized(rel, data):
    if rel.lower().endswith(CRLF_EXT):
        return data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    return data


print(f"1. The public code at {A.commit[:12]} ({A.repo})")
assert len(A.commit) == 40, "--commit must be the full 40-character commit"
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
print(f"   {2 + len(STARTERS)} files and {len(data)} data files")

print("4. The handbook")
run([sys.executable, HERE / "make_handbook.py", STAGE], GARDEN_LOGO=A.logo)

print("5. The three downloads")
run([sys.executable, HERE / "make_zip.py", OUT / "junsgarden-for-jun.zip"],
    GARDEN_SRC=STAGE, GARDEN_CODE_COMMIT=A.commit, GARDEN_CODE_REPO=A.repo)

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
print(f"done: {OUT}")
