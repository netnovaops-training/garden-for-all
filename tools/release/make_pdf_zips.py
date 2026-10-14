"""Make the two PDF guide downloads (one per language) and their fingerprints.

    python tools/release/make_pdf_zips.py OUT_FOLDER --english "PDF guides/English" --korean "PDF guides/Korean"
                                          [--tag pdf-guides-2026-10-14b] [--list-file PDF-GUIDES-SHA256.txt]

The PDF guides are not in the program (not in git). They are attached to a
release on GitHub, one zip per language, and the installer fetches them
(tools/get-pdfs.py, tools/get-pdfs.ps1). This makes the zips from the folders
the PDFs were built in (make_pdfs.py) and prints each zip's size and SHA-256.
--list-file writes PDF-GUIDES-SHA256.txt, the list the installer checks the
downloads against; that file IS in the program.

The zips are reproducible: sorted names, a fixed date, UTF-8 names (the Korean
folder names), so the same PDFs always give the same fingerprint.
"""
import argparse, hashlib, pathlib, sys, zipfile

REPO = "netnovaops-training/garden-for-all"
ap = argparse.ArgumentParser()
ap.add_argument("out")
ap.add_argument("--english", required=True)
ap.add_argument("--korean", required=True)
ap.add_argument("--tag", default="pdf-guides-2026-10-14b")
ap.add_argument("--list-file", default="")
A = ap.parse_args()
OUT = pathlib.Path(A.out)
OUT.mkdir(parents=True, exist_ok=True)

rows = []
for code, folder, src, name in (("en", "English", A.english, "junsgarden-pdf-guides-english.zip"),
                                ("ko", "Korean", A.korean, "junsgarden-pdf-guides-korean.zip")):
    src = pathlib.Path(src)
    pdfs = sorted(p for p in src.rglob("*.pdf") if p.is_file())
    if not pdfs:
        sys.exit(f"no PDFs in {src}")
    dest = OUT / name
    with zipfile.ZipFile(dest, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for p in pdfs:
            info = zipfile.ZipInfo(f"{folder}/{p.relative_to(src).as_posix()}", date_time=(2026, 10, 14, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o644 << 16
            z.writestr(info, p.read_bytes())
    data = dest.read_bytes()
    sha = hashlib.sha256(data).hexdigest()
    rows.append((sha, name, len(data), code))
    print(f"{name}: {len(pdfs)} PDFs, {len(data):,} bytes ({len(data)/1048576:.1f} MB), sha256 {sha}")
    print(f"   at 25 Mbps about {len(data)*8/25e6:.0f} seconds")

if A.list_file:
    lines = ["# Fingerprints of the optional PDF guides (one zip per language), checked by tools/get-pdfs.py",
             "# and tools/get-pdfs.ps1 before anything is written. Format: sha256, file, bytes, language[, hosts].",
             "# hosts (optional): host[:id],host[:id] (default: github); their addresses are in DOWNLOAD-HOSTS.txt.",
             f"url=https://github.com/{REPO}/releases/download/{A.tag}"]
    lines += [f"{sha}  {name} {size} {code}" for sha, name, size, code in rows]
    pathlib.Path(A.list_file).write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    print("wrote", A.list_file)
