#!/usr/bin/env python3
"""Release side: encrypt the photos add-on and cut it into small parts, with a list.

    python tools/release/make_photo_parts.py ZIP OUT_FOLDER --key-dir FOLDER
        [--part-mb 25] [--tag photos-2026-10-14] [--author-every 10]
        [--hosts github,drive,author] [--prefer 1=drive] [--pin N=HOST] [--manifest PHOTO-PARTS.json]

  1. Makes the key ONCE: a random 256-bit key written to FOLDER/photos.key (line 1 is the key,
     line 2 is "id=<8 hex>"). FOLDER must be OUTSIDE every git repository (it refuses
     otherwise). The key is never printed and never written anywhere else.
  2. Encrypts the zip as ONE stream (openssl enc -aes-256-ctr -pbkdf2, the same openssl that
     is on a stock Ubuntu) and cuts the result into parts of --part-mb, named
     <zip name>.enc.001, .002, ... in OUT_FOLDER, each with its own SHA-256.
  3. Writes the list (PHOTO-PARTS.json): every part's name, size, SHA-256 and the hosts that
     hold it (all parts on the first two hosts; every --author-every'th part also on "author";
     --prefer N=HOST makes part N fetched first from that host, the others only if it does not answer;
     --pin N=HOST makes part N held ONLY by that host), plus the plaintext
     zip's size and SHA-256 and the key id. Copy it into the program (git).
  4. Prints how to put the key into the main data zip:
       GARDEN_PHOTOS_KEY=FOLDER/photos.key  python tools/release/make_zip.py OUT/junsgarden-for-jun.zip
     (or build_release.py ... --photos-key FOLDER/photos.key). The key goes ONLY into that zip.

    python tools/release/make_photo_parts.py --set-ids PHOTO-PARTS.json IDS.txt [--host drive]
  rewrites the list so each part names its file id on that host (IDS.txt: "<part name> <id>" per
  line). Run it once the parts are uploaded.
"""
import argparse, hashlib, json, os, secrets, subprocess, sys, threading

ITER = 10000


def inside_a_repo(path):
    p = os.path.abspath(path)
    while True:
        if os.path.exists(os.path.join(p, ".git")):
            return True
        q = os.path.dirname(p)
        if q == p:
            return False
        p = q


def set_ids(manifest, idsfile, host):
    m = json.load(open(manifest, encoding="utf-8"))
    ids = dict(l.split(None, 1) for l in open(idsfile, encoding="utf-8").read().splitlines() if l.strip())
    for p in m["parts"]:
        if p["name"] in ids:
            keep = [h for h in p["hosts"] if h.split(":")[0] != host]
            p["hosts"] = keep + [f"{host}:{ids[p['name']].strip()}"]
    json.dump(m, open(manifest, "w", encoding="utf-8", newline="\n"), indent=1)
    print(f"ids set on {sum(1 for p in m['parts'] if any(h.startswith(host + ':') for h in p['hosts']))} parts")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("zip", nargs="?")
    ap.add_argument("out", nargs="?")
    ap.add_argument("--key-dir")
    ap.add_argument("--part-mb", type=float, default=25)
    ap.add_argument("--tag", default="photos-2026-10-14")
    ap.add_argument("--author-every", type=int, default=10)
    ap.add_argument("--hosts", default="github,drive,author")
    ap.add_argument("--pin", action="append", default=[])
    ap.add_argument("--prefer", action="append", default=[])
    ap.add_argument("--private-part", action="append", type=int, default=[],
                    help="N: part N is NOT public: it is held by drive (preferred) and the private repository only")
    ap.add_argument("--private-repo", default="", help="OWNER/REPO@REF of the private repository (with --private-part)")
    ap.add_argument("--set-private-repo", nargs=2, metavar=("LIST", "OWNER/REPO@REF"))
    ap.add_argument("--manifest", default="PHOTO-PARTS.json")
    ap.add_argument("--set-ids", nargs=2)
    ap.add_argument("--host", default="drive")
    a = ap.parse_args()
    if a.set_private_repo:
        m = json.load(open(a.set_private_repo[0], encoding="utf-8"))
        n = 0
        for p in m["parts"]:
            if any(h.startswith("ghprivate") for h in p["hosts"]):
                p["hosts"] = [h for h in p["hosts"] if not h.startswith("ghprivate")] + ["ghprivate:" + a.set_private_repo[1]]
                n += 1
        json.dump(m, open(a.set_private_repo[0], "w", encoding="utf-8", newline=chr(10)), indent=1)
        print(f"private repository set on {n} part(s)")
        return 0
    if a.set_ids:
        set_ids(a.set_ids[0], a.set_ids[1], a.host)
        return 0
    if a.private_part and not a.private_repo:
        ap.error("--private-part needs --private-repo OWNER/REPO@REF")
    if not (a.zip and a.out and a.key_dir):
        ap.error("ZIP, OUT_FOLDER and --key-dir are needed")
    if inside_a_repo(a.key_dir):
        sys.exit("STOP: --key-dir is inside a git repository. Give a folder outside every repo.")
    os.makedirs(a.key_dir, exist_ok=True)
    os.makedirs(a.out, exist_ok=True)
    keyfile = os.path.join(a.key_dir, "photos.key")
    if not os.path.exists(keyfile):
        key = secrets.token_hex(32)
        kid = hashlib.sha256(b"garden-photos-key-id:" + key.encode()).hexdigest()[:8]
        fd = os.open(keyfile, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, "w", newline="\n") as f:
            f.write(f"{key}\nid={kid}\n")
        print("made a new key (not shown); id", kid)
    lines = open(keyfile, encoding="utf-8").read().splitlines()
    kid = lines[1][3:]

    name = os.path.basename(a.zip)
    part_size = int(a.part_mb * (1 << 20))
    cmd = ["openssl", "enc", "-aes-256-ctr", "-pbkdf2", "-md", "sha256", "-iter", str(ITER), "-pass", "file:" + keyfile]
    proc = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    plain = hashlib.sha256()
    plain_size = [0]

    def feed():
        with open(a.zip, "rb") as f:
            for block in iter(lambda: f.read(1 << 20), b""):
                plain.update(block)
                plain_size[0] += len(block)
                proc.stdin.write(block)
        proc.stdin.close()

    t = threading.Thread(target=feed)
    t.start()
    hosts = a.hosts.split(",")
    pins = {int(x.split("=")[0]): x.split("=")[1] for x in a.pin}
    prefers = {int(x.split("=")[0]): x.split("=")[1] for x in a.prefer}
    parts, n = [], 0
    while True:
        data = proc.stdout.read(part_size)
        if not data:
            break
        n += 1
        pname = f"{name}.enc.{n:03d}"
        if n in a.private_part:        # never in the public folder: it goes to private-repo/parts/ next to it
            pdir = os.path.join(os.path.dirname(os.path.abspath(a.out)), "private-repo", "parts")
            os.makedirs(pdir, exist_ok=True)
            open(os.path.join(pdir, pname), "wb").write(data)
        else:
            open(os.path.join(a.out, pname), "wb").write(data)
        h = list(hosts[:2]) + ([hosts[2]] if len(hosts) > 2 and n % a.author_every == 0 else [])
        entry = {"name": pname, "size": len(data), "sha256": hashlib.sha256(data).hexdigest(), "hosts": h}
        if n in prefers:             # fetched first from this host; the others only if it does not answer
            entry["hosts"] = [prefers[n]] + [x for x in entry["hosts"] if x != prefers[n]]
            entry["prefer"] = prefers[n]
            entry["priority"] = 1
        if n in a.private_part:        # drive first; the private repository only as the backup; not on public GitHub
            drive = next((h for h in hosts if h != "github"), "drive")
            entry["hosts"] = [drive, "ghprivate:" + a.private_repo]
            entry["prefer"] = drive
            entry["priority"] = 1
        if n in pins:
            entry["hosts"] = [pins[n]]
            entry["pinned"] = pins[n]
            entry["priority"] = 1
        parts.append(entry)
    t.join()
    if proc.wait() != 0 or not parts:
        sys.exit("STOP: openssl failed")
    manifest = {"format": 1, "set": "photos", "tag": a.tag, "file": name, "size": plain_size[0], "sha256": plain.hexdigest(),
                "encryption": {"cipher": "aes-256-ctr", "kdf": "pbkdf2", "iter": ITER, "key_id": kid, "key_file": "photos.key"},
                "parts": parts}
    with open(os.path.join(a.out, a.manifest), "w", encoding="utf-8", newline="\n") as f:
        json.dump(manifest, f, indent=1)
        f.write("\n")
    print(f"{n} parts of about {a.part_mb:g} MB in {a.out}; list: {os.path.join(a.out, a.manifest)}")
    print("Next: copy the list into the program as PHOTO-PARTS.json; upload the parts to each host;")
    print("put the key into the data zip (never anywhere else):")
    print(f"  GARDEN_PHOTOS_KEY=<key folder>/photos.key python tools/release/make_zip.py OUT/junsgarden-for-jun.zip")
    return 0


if __name__ == "__main__":
    sys.exit(main())
