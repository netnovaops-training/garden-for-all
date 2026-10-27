"""python -m image_index build | search | status | eval | serve"""
import argparse
import json
import os
import sys


def main(argv=None):
    ap = argparse.ArgumentParser(prog="image_index", description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    out_default = os.environ.get("IMAGE_INDEX_DIR", "/index")

    b = sub.add_parser("build", help="add new or changed photos to the index (safe to re-run)")
    b.add_argument("--photos", action="append", required=True, metavar="DIR",
                   help="photo folder; repeat. Optional prefix current= or future= forces the set")
    b.add_argument("--db", required=True, help="folder with current.json and future.json")
    b.add_argument("--out", default=out_default, help="index folder")
    b.add_argument("--workers", type=int, default=None, help="photo decode processes")
    b.add_argument("--model", default=None,
                   help="dinov2-small (default) | plantclef-b14 | bioclip-2 | bioclip | ...; "
                        "changing it rebuilds the index")
    b.add_argument("--limit", type=int, default=None, help="stop after N new photos (testing)")

    s = sub.add_parser("search", help="best-matching plants for one photo")
    s.add_argument("image")
    s.add_argument("-k", type=int, default=5)
    s.add_argument("--out", default=out_default)
    s.add_argument("--hint", action="append", help="text to re-rank with (tag text, a name)")

    st = sub.add_parser("status", help="what is in the index")
    st.add_argument("--out", default=out_default)

    e = sub.add_parser("eval", help="leave-one-photo-out accuracy on the current plants")
    e.add_argument("--out", default=out_default)
    e.add_argument("--write-temperature", action="store_true",
                   help="store the fitted confidence temperature in the index")

    v = sub.add_parser("verify-name", help="does this photo match this botanical name?")
    v.add_argument("image")
    v.add_argument("name", help="botanical name as in the database, e.g. \"Salvia nemorosa\"")
    v.add_argument("--out", default=out_default, help="index folder (its plants give the names)")
    v.add_argument("--model", default=None, help="bioclip-2 (default) | bioclip | plantclef-b14")

    en = sub.add_parser("eval-names", help="how well a model checks names (zero-shot)")
    en.add_argument("--out", default=out_default)
    en.add_argument("--model", required=True)
    en.add_argument("--limit", type=int, default=800, help="photos in the random sample")

    sv = sub.add_parser("serve", help="small HTTP service for this computer only")
    sv.add_argument("--port", type=int, default=8099)
    sv.add_argument("--bind", default="127.0.0.1")
    sv.add_argument("--out", default=out_default)

    a = ap.parse_args(argv)
    if a.cmd == "build":
        from .indexer import build
        build(a.photos, a.db, a.out, a.workers, model=a.model, limit=a.limit)
    elif a.cmd == "search":
        from .finder import search
        print(json.dumps(search(a.image, a.k, out_dir=a.out, hints=a.hint), indent=2))
    elif a.cmd == "status":
        from .finder import status
        print(json.dumps(status(a.out), indent=2))
    elif a.cmd == "eval":
        from .evaluate import run
        print(json.dumps(run(a.out, write_temperature=a.write_temperature), indent=2))
    elif a.cmd == "verify-name":
        from .verify import verify_name
        print(json.dumps(verify_name(a.image, a.name, out_dir=a.out, model=a.model), indent=2))
    elif a.cmd == "eval-names":
        from .names_eval import run
        print(json.dumps(run(a.out, a.model, a.limit), indent=2))
    elif a.cmd == "serve":
        from .server import serve
        serve(a.bind, a.port, a.out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
