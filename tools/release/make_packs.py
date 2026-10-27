#!/usr/bin/env python3
"""Make (or check) the feature packs' index, the list the page builder checks every download against.

    python tools/release/make_packs.py                  write rust/junsgarden/packs/index.json
    python tools/release/make_packs.py --check          only check that index.json is exactly right (exit 1 if not)
    python tools/release/make_packs.py --bundle OUT     also write ONE file with the index, its signature and
                                                         every pack file (for Google Drive / the wizard)

Each pack lives in rust/junsgarden/packs/<id>/pack.json (names, version, what it provides) with its files in
<id>/<version>/. This script lists every file the pack uses with its size and sha256 and puts the whole pack
entry into index.json. Then the AUTHOR signs index.json (docs/page-builder/sign-packs.sh), which writes
index.json.sig. Changing any pack file or pack.json makes index.json (and so the signature) out of date:
run this script again, bump `serial` (it does that for you), and sign again.

A pack's files are plain text only (.svg, .css, .json); no program code. See docs/page-builder/PACKS.md.
"""
import argparse, hashlib, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
PACKS = os.path.join(HERE, "..", "..", "rust", "junsgarden", "packs")
CHANNEL = "garden-for-all"
FILE_KEYS = ("top", "bottom", "hero", "css", "template")


def used_files(provide):
    return [provide[k] for k in FILE_KEYS if provide.get(k)]


def build_pack(pack_dir):
    with open(os.path.join(pack_dir, "pack.json"), encoding="utf-8") as f:
        pack = json.load(f)
    names = []
    for p in pack["provides"]:
        for n in used_files(p):
            if n not in names:
                names.append(n)
    files = []
    for n in names:
        path = os.path.join(pack_dir, pack["version"], n)
        with open(path, "rb") as f:
            data = f.read()
        files.append({"path": n, "sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)})
    entry = dict(pack)
    entry["files"] = files
    return entry


def make_index(serial, released):
    packs = []
    for name in sorted(os.listdir(PACKS)):
        d = os.path.join(PACKS, name)
        if os.path.isfile(os.path.join(d, "pack.json")):
            packs.append(build_pack(d))
    return {"schema": 1, "channel": CHANNEL, "serial": serial, "released": released, "packs": packs}


def text_of(index):
    return json.dumps(index, indent=2, ensure_ascii=False) + "\n"


def read_current():
    path = os.path.join(PACKS, "index.json")
    if not os.path.exists(path):
        return None, ""
    with open(path, encoding="utf-8", newline="") as f:
        text = f.read()
    return json.loads(text), text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--serial", type=int, default=0, help="the list number (default: one more than now, only when something changed)")
    ap.add_argument("--released", default="", help="a label such as 2026.12 (default: the current one)")
    ap.add_argument("--bundle", default="", help="write one file with the index, its signature and all pack files")
    A = ap.parse_args()
    cur, cur_text = read_current()
    released = A.released or (cur or {}).get("released", "2026.12")
    serial = A.serial or (cur or {}).get("serial", 1)
    index = make_index(serial, released)
    if cur is not None:
        same = dict(cur, serial=0, released="") == dict(index, serial=0, released="")
        if not same and not A.serial:
            index["serial"] = serial + 1
    want = text_of(index)
    if A.check:
        if want == cur_text:
            print("index.json: up to date (%d packs, serial %d)" % (len(index["packs"]), index["serial"]))
            return
        # A different serial alone is not "stale"; compare everything else.
        if cur is not None and dict(cur, serial=0) == dict(index, serial=0):
            print("index.json: up to date (%d packs, serial %d)" % (len(cur["packs"]), cur["serial"]))
            return
        print("index.json is stale. Run: python tools/release/make_packs.py")
        sys.exit(1)
    out = os.path.join(PACKS, "index.json")
    if want != cur_text:
        with open(out, "w", encoding="utf-8", newline="") as f:
            f.write(want)
        sig = out + ".sig"
        if os.path.exists(sig):
            os.remove(sig)
            print("the old index.json.sig no longer matches: sign again (docs/page-builder/sign-packs.sh)")
        print("wrote index.json: %d packs, serial %d" % (len(index["packs"]), index["serial"]))
    else:
        print("index.json is already right (serial %d)" % index["serial"])
    if A.bundle:
        sigp = out + ".sig"
        sig = open(sigp, encoding="ascii").read().strip() if os.path.exists(sigp) else ""
        files = {}
        for p in index["packs"]:
            for fe in p["files"]:
                with open(os.path.join(PACKS, p["id"], p["version"], fe["path"]), encoding="utf-8", newline="") as f:
                    files["%s/%s/%s" % (p["id"], p["version"], fe["path"])] = f.read()
        with open(A.bundle, "w", encoding="utf-8", newline="") as f:
            json.dump({"index": want, "sig": sig, "files": files}, f, ensure_ascii=False)
        print("wrote %s (%s)" % (A.bundle, "signed" if sig else "NOT signed: sign index.json first"))


if __name__ == "__main__":
    main()
