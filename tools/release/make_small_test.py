"""Test-only: a small copy of the garden data, so a full install test is quick.
Same database (all plants), ~40 photos per folder, 40 wiki pages.
Writes into the scratchpad; the real package is untouched."""
import gzip, hashlib, io, json, os, sys, tarfile

# Run from anywhere:  python tools/release/make_small_test.py [OUTPUT-FOLDER]
PKG = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FULL = "junsgarden-backup_2026-10-13_1524_from-jasons-server"
SMALL = "junsgarden-backup_2026-10-13_1524_test-small"
OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(PKG), "small-test")
PER_FOLDER, PAGES = 40, 40
os.makedirs(OUT, exist_ok=True)

src = tarfile.open(os.path.join(PKG, "backups", FULL + ".tar"))
pieces = {}
for m in src.getmembers():
    if m.isfile():
        pieces[os.path.basename(m.name)] = src.extractfile(m).read() if m.name.endswith("images.tar") is False else None

# images.tar: keep the first PER_FOLDER photos of every folder.
img_in = tarfile.open(fileobj=src.extractfile(f"{FULL}/images.tar"), mode="r|")
buf = io.BytesIO(); kept = {}; count = 0
with tarfile.open(fileobj=buf, mode="w") as img_out:
    for m in img_in:
        folder = m.name.lstrip("./").split("/")[0]
        if m.isdir():
            img_out.addfile(m); continue
        if kept.get(folder, 0) >= PER_FOLDER:
            continue
        kept[folder] = kept.get(folder, 0) + 1; count += 1
        img_out.addfile(m, img_in.extractfile(m))
pieces["images.tar"] = buf.getvalue()

manifest = pieces["MANIFEST.txt"].decode().replace(FULL, SMALL)
manifest = manifest.replace("Jun's Garden backup\n",
    "Jun's Garden backup -- SMALL TEST COPY (only for testing the install)\n")
manifest = manifest.replace("7723 files, 781M", f"{count} files (a sample of each folder)")
pieces["MANIFEST.txt"] = manifest.encode()

order = ["database.sql.gz", "data.tar", "cashe.tar", "images.tar"]
names = {"database.sql.gz": "database.sql.gz", "data.tar": "./data.tar", "cashe.tar": "./cashe.tar", "images.tar": "./images.tar"}
sums = "".join(f"{hashlib.sha256(pieces[p]).hexdigest()}  {names[p]}\n" for p in order)
sums += f"{hashlib.sha256(pieces['MANIFEST.txt']).hexdigest()}  MANIFEST.txt\n"
pieces["SHA256SUMS"] = sums.encode()

tar_path = os.path.join(OUT, SMALL + ".tar")
with tarfile.open(tar_path, "w") as out:
    d = tarfile.TarInfo(SMALL); d.type = tarfile.DIRTYPE; d.mode = 0o755; out.addfile(d)
    for p in ["MANIFEST.txt", "SHA256SUMS"] + order:
        ti = tarfile.TarInfo(f"{SMALL}/{p}"); ti.size = len(pieces[p]); ti.mode = 0o644
        out.addfile(ti, io.BytesIO(pieces[p]))

# Wiki seed: the first PAGES pages, everything else the same.
seed_path = os.path.join(PKG, "docker", "wiki", "seed", "wiki-pages.json.gz")
seed = json.load(gzip.open(seed_path, "rt", encoding="utf-8"))
seed["pages"] = seed["pages"][:PAGES]
with gzip.open(os.path.join(OUT, "wiki-pages.json.gz"), "wt", encoding="utf-8") as fh:
    json.dump(seed, fh)
print("photos kept:", count, kept, "| backup MB:", round(os.path.getsize(tar_path) / 1e6, 1), "| wiki pages:", len(seed["pages"]))
