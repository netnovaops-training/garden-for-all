#!/usr/bin/env python3
# =============================================================================
#  export_wiki.py: copy the garden wiki's PAGES out of the author's server.
#
#  Jun, you do NOT need to run this. The author ran it once on his server (the
#  NUC) to make the file seed/wiki-pages.json.gz that your own wiki is
#  filled from. It is kept here so you can see exactly what was copied.
#
#  What it copies (content only):
#    - every page: path, language, title, description, the page text
#      (Markdown), editor, published/private, tags, created/updated dates
#    - the tag list
#    - the left-hand menu (navigation) of the wiki
#    - what the "Guests" group may read, so public pages stay public
#    - the wiki's short description and company name
#
#  What it does NOT copy, on purpose:
#    - users and their password hashes (the author's logins)
#    - API keys, sign-in settings, the session secret, certificates
#    - page history (old versions), comments, analytics
#
#  It only READS. It never changes the wiki or its database.
#
#  How the author ran it (on the NUC):
#      python3 export_wiki.py /tmp/wiki-pages.json.gz
# =============================================================================
import datetime
import gzip
import hashlib
import json
import subprocess
import sys

# The database container on the NUC and the wiki's database inside it.
DB_CONTAINER = "nuc-homelab-junsgarden-db-1"
DB_NAME = "junsgarden_wiki"

# One SELECT per kind of thing. Each row comes back as ONE line of JSON
# (MariaDB's JSON_OBJECT turns new lines inside the text into \n), so the
# output is easy and safe to read line by line.
SQL = r"""
SELECT JSON_OBJECT(
  'kind','page', 'id', id, 'path', path, 'locale', localeCode,
  'title', title, 'description', description, 'content', content,
  'editor', editorKey, 'contentType', contentType,
  'isPublished', isPublished, 'isPrivate', isPrivate,
  'publishStartDate', publishStartDate, 'publishEndDate', publishEndDate,
  'extra', extra, 'createdAt', createdAt, 'updatedAt', updatedAt,
  'sha256', SHA2(content, 256))
FROM pages ORDER BY id;
SELECT JSON_OBJECT('kind','pagetag', 'pageId', pt.pageId, 'tag', t.tag)
FROM pageTags pt JOIN tags t ON t.id = pt.tagId ORDER BY pt.pageId, t.tag;
SELECT JSON_OBJECT('kind','tag', 'tag', tag, 'title', title) FROM tags ORDER BY tag;
SELECT JSON_OBJECT('kind','navigation', 'key', `key`, 'config', config) FROM navigation;
SELECT JSON_OBJECT('kind','group', 'name', name, 'permissions', permissions,
                   'pageRules', pageRules)
FROM `groups` WHERE id = 2;
SELECT JSON_OBJECT('kind','setting', 'key', `key`, 'value', value)
FROM settings WHERE `key` IN ('seo', 'company', 'nav');
"""


def run_sql():
    """Run the SELECTs inside the database container.

    The root password is read INSIDE the container from its own settings
    ($MARIADB_ROOT_PASSWORD), so it is never typed, shown or saved here.
    """
    inner = ('export MYSQL_PWD="$MARIADB_ROOT_PASSWORD"; '
             f'exec mariadb -uroot --default-character-set=utf8mb4 -N -B --raw {DB_NAME}')
    done = subprocess.run(["docker", "exec", "-i", DB_CONTAINER, "sh", "-c", inner],
                          input=SQL.encode("utf-8"), capture_output=True, check=False)
    if done.returncode != 0:
        sys.exit("ERROR: the database did not answer: "
                 + done.stderr.decode("utf-8", "replace")[:300])
    return [json.loads(line) for line in done.stdout.decode("utf-8").splitlines() if line.strip()]


def as_json(value, fallback):
    """Some columns hold JSON text. Turn it into real JSON (lists, dicts)."""
    if isinstance(value, (dict, list)):
        return value
    try:
        return json.loads(value) if value else fallback
    except ValueError:
        return fallback


def main():
    out_file = sys.argv[1] if len(sys.argv) > 1 else "wiki-pages.json.gz"
    rows = run_sql()

    pages, tags_by_page, tags, navigation, guests, settings = [], {}, [], None, None, {}
    for row in rows:
        kind = row.pop("kind")
        if kind == "page":
            pages.append(row)
        elif kind == "pagetag":
            tags_by_page.setdefault(row["pageId"], []).append(row["tag"])
        elif kind == "tag":
            tags.append(row)
        elif kind == "navigation" and row["key"] == "site":
            navigation = as_json(row["config"], [])
        elif kind == "group":
            guests = {"permissions": as_json(row["permissions"], []),
                      "pageRules": as_json(row["pageRules"], [])}
        elif kind == "setting":
            settings[row["key"]] = as_json(row["value"], {})

    # Check every page text arrived exactly as it is in the database:
    # the database's own fingerprint (SHA-256) must match ours.
    bad = [p["path"] for p in pages
           if hashlib.sha256((p["content"] or "").encode("utf-8")).hexdigest() != p["sha256"]]
    if bad:
        sys.exit(f"ERROR: {len(bad)} page texts did not copy cleanly, for example {bad[:3]}")

    seed_pages = []
    for p in pages:
        extra = as_json(p["extra"], {})
        seed_pages.append({
            "path": p["path"],
            "locale": p["locale"],
            "title": p["title"],
            "description": p["description"] or "",
            "content": p["content"],
            "editor": p["editor"],
            "contentType": p["contentType"],
            "isPublished": bool(p["isPublished"]),
            "isPrivate": bool(p["isPrivate"]),
            "publishStartDate": p["publishStartDate"] or "",
            "publishEndDate": p["publishEndDate"] or "",
            "scriptCss": extra.get("css", ""),
            "scriptJs": extra.get("js", ""),
            "tags": sorted(tags_by_page.get(p["id"], [])),
            "createdAt": p["createdAt"],
            "updatedAt": p["updatedAt"],
            "sha256": p["sha256"],
        })

    seed = {
        "format": "junsgarden-wiki-seed/1",
        "about": "Pages of the Jun's Garden wiki. Content only: no users, "
                 "passwords, keys, sign-in settings or history.",
        "exportedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "pageCount": len(seed_pages),
        "pages": seed_pages,
        "tags": tags,
        "navigation": {
            # No "nav" setting saved means Wiki.js uses its default, MIXED.
            "mode": settings.get("nav", {}).get("mode", "MIXED"),
            "tree": navigation or [],
        },
        "guestGroup": guests,
        "site": {
            "description": settings.get("seo", {}).get("description", ""),
            "company": settings.get("company", {}).get("v", ""),
        },
    }
    with gzip.open(out_file, "wt", encoding="utf-8") as fh:
        json.dump(seed, fh, ensure_ascii=False, indent=1)

    tag_links = sum(len(p["tags"]) for p in seed_pages)
    print(f"pages={len(seed_pages)} tags={len(tags)} page-tag-links={tag_links} "
          f"nav-items={sum(len(t.get('items', [])) for t in seed['navigation']['tree'])} "
          f"guest-rules={len((guests or {}).get('pageRules', []))} -> {out_file}")
    print("All page texts match the database fingerprints (SHA-256).")


if __name__ == "__main__":
    main()
