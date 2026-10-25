#!/usr/bin/env python3
"""Made-up sample garden for the side-by-side lab and the installer test (no real data).

    python3 make_sample.py OUT_DIR

Writes OUT_DIR/sample-data.sql (rows for the repo's own db/schema.sql), OUT_DIR/images/
(hero, plant, gallery and post pictures as tiny generated PNG files, plus the gallery's
tags_cache.csv) and OUT_DIR/data/ (facts, guestbook, comment files). Same output every run.
"""
import json
import os
import random
import re
import struct
import sys
import zlib

R = random.Random(7)
OUT = sys.argv[1]


def save(path, data):
    """Write text or bytes to a file (closed at once)."""
    with open(path, "wb" if isinstance(data, bytes) else "w", **({} if isinstance(data, bytes) else {"encoding": "utf-8"})) as f:
        f.write(data)


def save_json(path, value):
    save(path, json.dumps(value, indent=4))

BEDS = ["Front Border", "Korea Bed", "Shade Garden", "Vegetable Garden Bed", "Herb Spiral",
        "Rose Corner", "Pond Edge", "Dead or Removed"]
FAMILIES = ["Asparagaceae", "Rosaceae", "Lamiaceae", "Asteraceae", "Ranunculaceae", "Solanaceae", "Hydrangeaceae"]
COMPANIES = ["Sample Nursery", "Green Test Co", "Proven Example", "Seed Lab", ""]
SUN = ["Full Sun", "Part Shade", "Full Shade", "Full Sun, Part Shade"]
WATER = ["Low", "Average", "High"]
DEER = ["Resistant", "Not Resistant", "Somewhat"]
ZONES = ["3-8", "4-9", "5-9", "6-10"]
TOX = ["Non-toxic", "Toxic to Dogs/Cats", "Toxic to Humans", "Toxic"]
EDIBLE = ["Edible", "Inedible", "Wildlife Only", "Toxic"]
SEASON = ["Spring", "Summer", "Fall", "Spring, Summer", ""]
COLORS = ["White", "Pink", "Purple", "Yellow", "Red", "Blue", "Not ornamental"]
TYPES = ["Perennial", "Shrub", "Annual", "Herb", "Vegetable", "Tree"]
FERT = ["Bio-tone", "Berry-tone", "Compost", "Fish emulsion, Compost", ""]
NAMES = ["Hosta", "Rose", "Lavender", "Coneflower", "Peony", "Tomato", "Hydrangea", "Sage", "Basil",
         "Daylily", "Astilbe", "Clematis", "Mint", "Pepper", "Iris", "Phlox", "Salvia", "Yarrow",
         "Fern", "Thyme"]
ADJ = ["Blue", "Lil' Sample", "Test", "Dwarf", "Giant", "Early", "Late", "Golden", "Velvet", "Moon"]


def png(rgb, w=24, h=16):
    """A tiny one-colour PNG, without any image library."""
    raw = b"".join(b"\x00" + bytes(rgb) * w for _ in range(h))

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def q(v):
    if v is None:
        return "NULL"
    if isinstance(v, (int, float)):
        return str(v)
    if isinstance(v, bytes):
        return "0x" + v.hex()
    return "'" + str(v).replace("\\", "\\\\").replace("'", "''") + "'"


def name(n):
    """A table or column name, checked: only letters, digits and _ (so it can never carry SQL)."""
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", n):
        raise ValueError(f"not a plain SQL name: {n!r}")
    return n


# The two statement builders put SQL together from parts: fixed keywords, names checked by
# name(), values written by q() and ids made ints. Nothing else ever reaches the SQL text.
def insert(table, rows):
    cols = list(rows[0].keys())
    vals = ",\n".join("(" + ",".join(q(r[c]) for c in cols) + ")" for r in rows)
    col_list = ",".join("`" + name(c) + "`" for c in cols)
    return "".join(["INSERT INTO `", name(table), "` (", col_list, ") VALUES\n", vals, ";\n"])


def update(table, values, ids):
    """UPDATE table SET col = value, ... WHERE id IN (ids)."""
    sets = ", ".join(name(c) + " = " + q(v) for c, v in values.items())
    id_list = ", ".join(str(int(i)) for i in ids)
    return "".join(["UPDATE ", name(table), " SET ", sets, " WHERE id IN (", id_list, ");\n"])


def plant(i, future=False):
    name = f"{R.choice(ADJ)} {NAMES[i % len(NAMES)]}"
    bed = "Want" if future else BEDS[i % len(BEDS)]
    died = 1 if bed == "Dead or Removed" else 0
    row = {
        "id": i, "plant_name": name, "botanical_name": f"Exemplum {NAMES[i % len(NAMES)].lower()}",
        "family": R.choice(FAMILIES), "company": R.choice(COMPANIES), "flower_bed": bed, "garden_beds": bed,
        "qty": R.choice([1, 2, 3, 5, 0]) if not died else 0, "died": died,
        "year_planted": str(R.choice([2021, 2022, 2023, 2024, 2025])), "sun_exposure": R.choice(SUN),
        "water_needs": R.choice(WATER), "deer_resistance": R.choice(DEER), "zones": R.choice(ZONES),
        "toxicity": R.choice(TOX), "fruit_edibility": R.choice(EDIBLE), "season": R.choice(SEASON),
        "bloom_color": R.choice(COLORS), "plant_type": R.choice(TYPES), "fertilizer": R.choice(FERT),
        "attracts": R.choice(["Bees", "Bees, Butterflies", "Hummingbirds", ""]),
        "care_notes": "Sample care note.", "local_store": R.choice(["Store A", "Store B", ""]),
        "picture": f"plant-{i % 6}.png" if i % 4 else None,
        "additional_pictures": f"plant-{(i + 1) % 6}.png" if i % 5 == 0 else None,
    }
    if not future:
        row.update({"image_approved": i % 2, "photo_position": "50% 50%", "died_year": 2025 if died else None,
                    "barcode": f"0000000{i:05d}" if i % 7 == 0 else None})
    return row


def main():
    os.makedirs(f"{OUT}/data", exist_ok=True)
    plants = [plant(i) for i in range(1, 61)]
    future = [plant(i, future=True) for i in range(1, 11)]
    sql = ["SET FOREIGN_KEY_CHECKS=0;\n", insert("data_source", plants), insert("future_plants", future),
           insert("garden_blog", [{"id": n, "author": a, "title": t, "content": c, "picture": p,
                                    "created_at": d} for n, a, t, c, p, d in [
               (1, "Jun", "First sprouts", "The sample seeds are up.\nTwo lines.", "post-1.png", "2026-04-02 10:00:00"),
               (2, None, "Rain day", "Watered nothing &amp; enjoyed it.", None, "2026-05-10 09:30:00"),
               (3, "Joshua", "Tomatoes", "Staked the test tomatoes.", "https://images.unsplash.com/photo-1", "2026-06-01 18:00:00"),
               (4, "Jun", "Fall cleanup", "Cut back the sample perennials.", None, "2026-10-01 12:00:00")]]),
           insert("user_favorites", [{"username": u, "plant_id": p} for u, p in
                                     [("Joshua", 2), ("Joshua", 5), ("Mina", 3), ("Kona", 4), ("Guest_abc", 6), ("Someone", 2)]]),
           insert("user_likes", [{"username": u, "plant_id": p} for u, p in [("Joshua", 1), ("Guest_x", 1), ("Kona", 9)]]),
           insert("plant_quantity_log", [{"plant_id": 1, "event_type": "purchased", "quantity": 2, "event_date": "2026-04-06",
                                          "event_week": 15, "event_month": 4, "event_year": 2026, "notes": None, "recorded_by": "Jun"}]),
           insert("plant_barcodes", [{"plant_id": 7, "source_table": "data_source", "size_label": "1 gal",
                                      "barcode": "000000000007", "barcode_image": png((0, 0, 0))}]),
           insert("flower_bed_qr", [{"bed_name": "Korea Bed", "qr_code": png((20, 20, 20))}]),
           update("data_source", {"qr_code": png((10, 10, 10))}, [1, 2])]
    save(f"{OUT}/sample-data.sql", "".join(sql))

    pics = {"HeroBanner": [f"hero-{n}.png" for n in range(5)], "Database": [f"plant-{n}.png" for n in range(6)],
            "Gallery": [f"garden-{n}.png" for n in range(6)], "post": ["post-1.png"]}
    for folder, names in pics.items():
        os.makedirs(f"{OUT}/images/{folder}", exist_ok=True)
        for n in names:
            save(f"{OUT}/images/{folder}/{n}", png((R.randrange(256), R.randrange(256), R.randrange(256))))
    with open(f"{OUT}/images/Gallery/tags_cache.csv", "w", encoding="utf-8") as f:
        f.write("file,name,thumb,thumb_tab,thumb_mob,download,tags,width,height\n")
        for n, name in enumerate(pics["Gallery"]):
            f.write(f'{name},"Sample garden {n}",,,,{name},{"Bees|Flowers" if n % 2 else "Vegetables"},24,16\n')

    d = f"{OUT}/data"
    save(f"{d}/facts.txt", "Sample fact one.\nSample fact two.\nSample fact three.\n")
    save(f"{d}/comments.txt",
         "<span class='c-date'>Oct 1</span> <strong>Kona:</strong> Lovely sample garden\n"
         "<span class='c-date'>Sep 3</span> <strong>Guest:</strong> Hello\n")
    save_json(f"{d}/plant_comments.json", {"2": [{"name": "Kona", "message": "Nice", "date": "Oct 1, 2026", "ts": 1790000000}]})
    save_json(f"{d}/gallery_comments.json",
              {"garden-1.png": [{"name": "Guest", "message": "Pretty", "date": "Oct 2, 2026", "ts": 1790000100}]})
    save(f"{d}/comment_notify.txt", "0")
    print(f"sample written: {len(plants)} plants, {len(future)} wish-list, 4 posts")


if __name__ == "__main__":
    main()
