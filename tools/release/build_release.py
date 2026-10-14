"""Build a release of Jun's Garden from a COPY of this package (the original is never written to).

    python tools/release/build_release.py OUT_FOLDER  [path\\to\\emblem.png]

1. Copies the package into OUT_FOLDER/build/junsgarden-for-jun (no .git, no local
   state: .env, local/, logs/, the Windows target file).
2. Builds the English PDFs into the copy ("Setup documentation/English/...").
3. Builds the handbook into the copy (00-WELCOME.html, handbook/), with the
   emblem as its logo when one is given (the emblem is never kept in git).
4. Makes the three downloads from the copy (make_zip.py): the garden, the
   photos, the wiki, side by side in OUT_FOLDER.
5. Builds the Korean PDFs and packs them as one zip for the email to Jun
   (OUT_FOLDER/Jun's Garden guides (Korean).zip). They do not go in the download.
The fingerprints (FILES-FROM-AUTHOR.txt and the add-on fingerprints) are written
in the copy; compare them with the original before the final commit.
"""
import os, pathlib, shutil, subprocess, sys, zipfile

HERE = pathlib.Path(__file__).resolve().parent
PKG = HERE.parents[1]
OUT = pathlib.Path(sys.argv[1]).resolve()
LOGO = sys.argv[2] if len(sys.argv) > 2 else ""
STAGE = OUT / "build" / "junsgarden-for-jun"
SKIP_DIRS = {".git", "local", "logs", "help", "target", "node_modules", "__pycache__", "PDF", "PDF-src",
             "handbook", "Setup documentation"}
SKIP_FILES = {".env", ".garden-target-windows.txt", ".garden-lang", ".garden-mode", "00-WELCOME.html"}


def run(args, **env):
    print("  $", " ".join(str(a) for a in args))
    subprocess.run(args, check=True, env={**os.environ, **{k: str(v) for k, v in env.items()}})


print("1. Copy the package (the original is only read)")
if STAGE.exists():
    shutil.rmtree(STAGE)
# Only the files git tracks, plus the garden data the downloads are made from
# (the starter backup and the wiki pages, which are kept out of git on purpose).
# So nothing left over from building or testing (an app's build folder, a local
# settings file) can ever end up in a download.
tracked = subprocess.run(["git", "-C", str(PKG), "ls-files", "-z"], capture_output=True, check=True).stdout.decode("utf-8").split("\0")
data = [p.relative_to(PKG).as_posix() for p in list((PKG / "backups").glob("junsgarden-backup_*.tar")) +
        list((PKG / "docker" / "wiki" / "seed").glob("*.json.gz"))]
for rel in [t for t in tracked if t] + data:
    src, dst = PKG / rel, STAGE / rel
    if src.is_file():
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)
print(f"   {len([t for t in tracked if t])} tracked files and {len(data)} data files")
print("2. English PDFs into the copy")
run([sys.executable, HERE / "make_pdfs.py", STAGE / "Setup documentation"], GARDEN_PDF_LANG="en", GARDEN_WATERMARK=LOGO.replace("emblem-mark", "emblem-full"))
print("3. The handbook into the copy")
run([sys.executable, HERE / "make_handbook.py", STAGE], GARDEN_LOGO=LOGO)
print("4. The three downloads, made from the copy")
run([sys.executable, STAGE / "tools" / "release" / "make_zip.py", OUT / "junsgarden-for-jun.zip"], GARDEN_SRC=STAGE)
print("5. The Korean PDFs, one zip for the email")
ko = OUT / "korean-pdfs"
run([sys.executable, HERE / "make_pdfs.py", ko], GARDEN_PDF_LANG="ko", GARDEN_WATERMARK=LOGO.replace("emblem-mark", "emblem-full"))
kz = OUT / "Jun's Garden guides (Korean).zip"
with zipfile.ZipFile(kz, "w", zipfile.ZIP_STORED) as z:
    for f in sorted((ko / "Korean").rglob("*.pdf")):
        z.write(f, f.relative_to(ko))
print(f"done: {OUT}  (Korean zip {kz.stat().st_size / 1e6:.1f} MB)")
