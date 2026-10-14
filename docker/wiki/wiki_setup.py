#!/usr/bin/env python3
# =============================================================================
#  wiki_setup.py: set up YOUR wiki the first time, and fill it with the
#  garden pages.
#
#  You start it with:
#      docker compose run --rm wiki-setup
#  It asks for ONE thing: the password you want for your wiki admin login.
#  Nothing shows while you type (that is normal). The password is never
#  saved in a file; only the wiki keeps a scrambled "hash" of it.
#
#  What it does, step by step:
#    1. Waits until the wiki (container garden-wiki) answers.
#    2. If the wiki is already fully set up, it says so and stops.
#       Running it again is always safe.
#    3. Finishes the Wiki.js setup wizard for you (the page you would
#       otherwise fill in by hand): admin email (WIKI_ADMIN_EMAIL in .env),
#       your password, the web address (WIKI_SITE_URL in .env).
#    4. Signs in as you, through the wiki's GraphQL API (/graphql).
#    5. Lets visitors who are not signed in ("Guests") read the pages.
#       (No garden pages yet? They are the separate wiki add-on download. Then
#       it stops after step 4: your wiki is set up, empty, and running this
#       again once the add-on is there adds the pages.)
#    6. Adds every page from seed/wiki-pages.json.gz with its title,
#       description, tags, text and original address. Pages that are
#       already there are skipped, so a stopped run can just be started again.
#    7. Draws every page once more, so links between pages work (a link
#       to a page that was not added yet showed as "missing" before), then
#       puts back each page's original "created" and "last edited" dates.
#    8. Puts back the left-hand menu and names the site "Jun's Garden Wiki".
#    9. Writes a small "done" note into the wiki's settings, so step 2
#       knows next time.
#
#  For automatic tests the password can come in on standard input:
#      echo 'a-test-password' | docker compose run --rm -T wiki-setup
#
#  Help for the API: https://docs.requarks.io/dev/api
# =============================================================================
import getpass
import gzip
import json
import os
import subprocess
import sys
import time
import urllib.error
import urllib.request

# ---- Settings (they come from docker-compose.yml and .env) -----------------
WIKI_URL = os.environ.get("WIKI_URL", "http://wiki:3000").rstrip("/")
SITE_URL = os.environ.get("WIKI_SITE_URL", "http://localhost:3000").rstrip("/")
ADMIN_EMAIL = os.environ.get("WIKI_ADMIN_EMAIL", "").strip().lower()
SEED_FILE = os.environ.get("WIKI_SEED", "/seed/wiki-pages.json.gz")
SITE_TITLE = "Jun's Garden Wiki"
DB_HOST = os.environ.get("DB_HOST", "db")
DB_NAME = os.environ.get("WIKI_DB_NAME", "junsgarden_wiki")
DB_USER = os.environ.get("WIKI_DB_USER", "wikijs")
DB_PASSWORD = os.environ.get("WIKI_DB_PASSWORD", "")
DONE_KEY = "junsgardenSeed"      # the name of the "done" note in the settings table


def say(text):
    print(f"== {text}", flush=True)


def die(text):
    print(f"ERROR: {text}", file=sys.stderr, flush=True)
    sys.exit(1)


# ---- Talking to the database (only the wiki's own database) ----------------
def sql(query):
    """Run SQL in the wiki database and return the answer as text.

    The password goes in through MYSQL_PWD, never on the command line.
    """
    env = dict(os.environ, MYSQL_PWD=DB_PASSWORD)
    done = subprocess.run(
        ["mariadb", "-h", DB_HOST, "-u", DB_USER, "--default-character-set=utf8mb4",
         "-N", "-B", DB_NAME],
        input=query.encode("utf-8"), env=env, capture_output=True, check=False)
    if done.returncode != 0:
        die("the wiki database said: " + done.stderr.decode("utf-8", "replace").strip()[:300])
    return done.stdout.decode("utf-8").strip()


def sql_text(value):
    """Make a value safe to put between quotes in SQL."""
    return "'" + str(value).replace("\\", "\\\\").replace("'", "''") + "'"


# ---- Talking to the wiki over HTTP ------------------------------------------
class Wiki:
    """A tiny helper for the wiki's web API."""

    def __init__(self):
        self.token = None        # the sign-in "ticket" (JWT) after login

    def post(self, path, body, timeout=120):
        """Send JSON, get JSON back. Returns (http status, answer)."""
        data = json.dumps(body).encode("utf-8")
        req = urllib.request.Request(WIKI_URL + path, data=data, method="POST",
                                     headers={"Content-Type": "application/json"})
        if self.token:
            req.add_header("Authorization", "Bearer " + self.token)
        try:
            with urllib.request.urlopen(req, timeout=timeout) as resp:
                # The wiki hands out a fresh ticket when the old one gets
                # old (after 30 minutes). Keep the newest one.
                fresh = resp.headers.get("new-jwt")
                if fresh:
                    self.token = fresh
                text = resp.read().decode("utf-8")
                return resp.status, (json.loads(text) if text.startswith(("{", "[")) else text)
        except urllib.error.HTTPError as err:
            return err.code, err.read().decode("utf-8", "replace")

    def graphql(self, query, variables=None):
        """Ask the GraphQL API. Stops with a clear message on an error."""
        for attempt in range(1, 4):
            try:
                status, answer = self.post("/graphql", {"query": query, "variables": variables or {}})
            except (urllib.error.URLError, OSError) as err:
                status, answer = 0, str(err)
            if status == 200 and isinstance(answer, dict) and not answer.get("errors"):
                return answer["data"]
            if attempt < 3:
                time.sleep(3 * attempt)      # a short pause, then try again
        die(f"the wiki API said (HTTP {status}): {str(answer)[:400]}")

    def mode(self):
        """'setup' if the setup wizard is waiting, 'ready' if the wiki runs,
        None if it does not answer (yet)."""
        try:
            status, _ = self.post("/graphql", {"query": "{ __typename }"}, timeout=10)
        except (urllib.error.URLError, OSError):
            return None
        if status == 200:
            return "ready"
        if status == 404:              # the wizard has no /graphql yet
            return "setup"
        return None

    def wait(self, want=None, seconds=300):
        """Wait until the wiki answers (and is in the wanted mode)."""
        end = time.time() + seconds
        while time.time() < end:
            now = self.mode()
            if now and (want is None or now == want):
                return now
            time.sleep(3)
        die(f"the wiki did not answer at {WIKI_URL} for {seconds // 60} minutes. "
            "Is garden-wiki running? Try: docker compose logs wiki")


def check(result, what):
    """Every wiki answer has a responseResult. Stop if it did not work."""
    rr = (result or {}).get("responseResult") or {}
    if not rr.get("succeeded"):
        die(f"{what} did not work: {rr.get('message') or rr}")


# ---- The steps --------------------------------------------------------------
def ask_password(first_time):
    """Read the admin password: hidden prompt for a person, stdin for tests."""
    if sys.stdin.isatty():
        if first_time:
            print("Choose a password for your wiki admin login "
                  f"({ADMIN_EMAIL}). 10 characters or more.")
        pw = getpass.getpass("Wiki admin password (nothing shows as you type): ")
        if first_time and getpass.getpass("Type it again: ") != pw:
            die("the two passwords did not match. Run it again.")
    else:
        pw = sys.stdin.readline().rstrip("\r\n")
    if len(pw) < 10:
        die("too short: use at least 10 characters")
    return pw


def finish_wizard(wiki, password):
    say("Finishing the Wiki.js setup wizard")
    status, answer = wiki.post("/finalize", {
        "adminEmail": ADMIN_EMAIL,
        "adminPassword": password,
        "adminPasswordConfirm": password,
        "siteUrl": SITE_URL,
        "telemetry": False,           # do not send anything to Wiki.js's makers
    })
    if status != 200 or not isinstance(answer, dict) or not answer.get("ok"):
        die(f"the setup wizard said: {answer if isinstance(answer, str) else answer.get('error')}")
    say("Wizard done. Waiting for the wiki to restart")
    time.sleep(5)
    wiki.wait("ready")


def sign_in(wiki, password, first_time):
    say(f"Signing in as {ADMIN_EMAIL}")
    data = wiki.graphql("""
      mutation ($u: String!, $p: String!) {
        authentication { login(username: $u, password: $p, strategy: "local") {
          responseResult { succeeded message } jwt } } }""",
                        {"u": ADMIN_EMAIL, "p": password})
    result = data["authentication"]["login"]
    if not (result.get("responseResult") or {}).get("succeeded") or not result.get("jwt"):
        hint = "" if first_time else (
            " The wiki was set up before: use the password you chose then, and check "
            "that WIKI_ADMIN_EMAIL in .env is the email you used then.")
        die("signing in did not work: "
            f"{(result.get('responseResult') or {}).get('message')}.{hint}")
    wiki.token = result["jwt"]


def restore_guest_access(wiki, guest):
    say("Letting visitors read the pages without signing in (Guests group)")
    groups = wiki.graphql("{ groups { list { id name isSystem } } }")["groups"]["list"]
    gid = next((g["id"] for g in groups if g["name"] == "Guests" and g["isSystem"]), None)
    if gid is None:
        die("the wiki has no Guests group")
    data = wiki.graphql("""
      mutation ($id: Int!, $perm: [String]!, $rules: [PageRuleInput]!) {
        groups { update(id: $id, name: "Guests", redirectOnLogin: "/",
                        permissions: $perm, pageRules: $rules) {
          responseResult { succeeded message } } } }""",
                        {"id": gid, "perm": guest["permissions"],
                         "rules": [{k: r[k] for k in ("id", "deny", "match", "roles", "path", "locales")}
                                   for r in guest["pageRules"]]})
    check(data["groups"]["update"], "Guest access")


def existing_pages(wiki):
    pages = wiki.graphql("{ pages { list { path locale } } }")["pages"]["list"]
    return {(p["locale"], p["path"]) for p in pages}


def import_pages(wiki, pages):
    have = existing_pages(wiki)
    todo = [p for p in pages if (p["locale"], p["path"]) not in have]
    say(f"Adding pages: {len(todo)} to add, {len(pages) - len(todo)} already there")
    started = time.time()
    for n, page in enumerate(todo, 1):
        data = wiki.graphql("""
          mutation ($content: String!, $description: String!, $editor: String!,
                    $isPublished: Boolean!, $isPrivate: Boolean!, $locale: String!,
                    $path: String!, $tags: [String]!, $title: String!,
                    $publishStartDate: Date, $publishEndDate: Date,
                    $scriptCss: String, $scriptJs: String) {
            pages { create(content: $content, description: $description, editor: $editor,
                           isPublished: $isPublished, isPrivate: $isPrivate, locale: $locale,
                           path: $path, tags: $tags, title: $title,
                           publishStartDate: $publishStartDate, publishEndDate: $publishEndDate,
                           scriptCss: $scriptCss, scriptJs: $scriptJs) {
              responseResult { succeeded message } } } }""", {
            "content": page["content"],
            "description": page["description"],
            "editor": page["editor"] or "markdown",
            "isPublished": page["isPublished"],
            "isPrivate": page["isPrivate"],
            "locale": page["locale"],
            "path": page["path"],
            "tags": page["tags"],
            "title": page["title"],
            "publishStartDate": page["publishStartDate"] or None,
            "publishEndDate": page["publishEndDate"] or None,
            "scriptCss": page["scriptCss"],
            "scriptJs": page["scriptJs"],
        })
        check(data["pages"]["create"], f"Adding page /{page['locale']}/{page['path']}")
        if n % 25 == 0 or n == len(todo):
            mins = (time.time() - started) / 60
            print(f"   {n} of {len(todo)} pages added ({mins:.1f} minutes so far)", flush=True)


def rerender_all(wiki):
    """Turn every page into a web page again, now that ALL pages exist.

    Why: while the pages were being added, a link to a page that was not
    added yet was marked as "missing" (a red link). Wiki.js 2 does not fix
    links written like /plants by itself afterwards, so we ask it to draw
    every page once more. This is the same as the admin button
    "Rerender all pages". 4 pages at a time, to be quicker.
    """
    from concurrent.futures import ThreadPoolExecutor
    ids = [p["id"] for p in wiki.graphql("{ pages { list { id } } }")["pages"]["list"]]
    say(f"Fixing the links between pages: drawing all {len(ids)} pages again")
    query = "mutation ($id: Int!) { pages { render(id: $id) { responseResult { succeeded message } } } }"
    started, done = time.time(), 0
    with ThreadPoolExecutor(max_workers=4) as pool:
        for data in pool.map(lambda pid: wiki.graphql(query, {"id": pid}), ids):
            check(data["pages"]["render"], "Drawing a page")
            done += 1
            if done % 100 == 0 or done == len(ids):
                print(f"   {done} of {len(ids)} pages drawn "
                      f"({(time.time() - started) / 60:.1f} minutes so far)", flush=True)


def restore_dates(pages):
    say("Putting back each page's original dates")
    lines = [f"UPDATE pages SET createdAt={sql_text(p['createdAt'])}, updatedAt={sql_text(p['updatedAt'])} "
             f"WHERE localeCode={sql_text(p['locale'])} AND path={sql_text(p['path'])};"
             for p in pages if p.get("createdAt") and p.get("updatedAt")]
    sql("\n".join(lines))


def restore_menu_and_title(wiki, seed):
    say("Putting back the menu")
    nav = seed["navigation"]
    tree = [{"locale": t["locale"],
             "items": [{k: i.get(k) for k in ("id", "kind", "label", "icon", "targetType",
                                              "target", "visibilityMode", "visibilityGroups")}
                       for i in t["items"]]}
            for t in nav["tree"]]
    if tree:
        data = wiki.graphql("""mutation ($tree: [NavigationTreeInput]!) {
            navigation { updateTree(tree: $tree) { responseResult { succeeded message } } } }""",
                            {"tree": tree})
        check(data["navigation"]["updateTree"], "The menu")
    data = wiki.graphql("""mutation ($mode: NavigationMode!) {
        navigation { updateConfig(mode: $mode) { responseResult { succeeded message } } } }""",
                        {"mode": nav.get("mode") or "MIXED"})
    check(data["navigation"]["updateConfig"], "The menu style")

    say(f'Naming the site "{SITE_TITLE}"')
    data = wiki.graphql("""mutation ($title: String!, $description: String, $company: String) {
        site { updateConfig(title: $title, description: $description, company: $company) {
          responseResult { succeeded message } } } }""",
                        {"title": SITE_TITLE, "description": seed["site"].get("description", ""),
                         "company": seed["site"].get("company", "")})
    check(data["site"]["updateConfig"], "The site name")


def main():
    if not DB_PASSWORD:
        die("WIKI_DB_PASSWORD is not set (is the .env file there?)")
    if not ADMIN_EMAIL or "@" not in ADMIN_EMAIL:
        die("WIKI_ADMIN_EMAIL in .env must be your email address (it is your wiki login)")

    wiki = Wiki()
    say(f"Waiting for the wiki at {WIKI_URL}")
    mode = wiki.wait()

    if mode == "ready" and sql(f"SELECT COUNT(*) FROM settings WHERE `key`='{DONE_KEY}'") == "1":
        say("Your wiki is already set up. Nothing to do.")
        print(f"   Open it at {SITE_URL}")
        return

    # The garden's pages come in a separate download, the wiki add-on. Without
    # it the wiki is still set up (with you as its admin), just empty.
    have_pages = os.path.isfile(SEED_FILE)
    if not have_pages and mode == "ready":
        say("Your wiki is set up. The garden pages are not here yet.")
        print("   To add them: put junsgarden-wiki-addon.zip in the garden folder (or Downloads),")
        print("   then run:  bash garden.sh wiki")
        return

    first_time = mode == "setup"
    password = ask_password(first_time)
    if first_time:
        finish_wizard(wiki, password)
    sign_in(wiki, password, first_time)
    del password                      # forget it as soon as it is not needed

    if not have_pages:
        say("Your wiki is set up, with you as its admin. It is empty for now.")
        print("   The garden pages are in a separate download: junsgarden-wiki-addon.zip.")
        print("   Put it in the garden folder (or Downloads), then run:  bash garden.sh wiki")
        print(f"   Open the wiki at {SITE_URL}  and sign in with {ADMIN_EMAIL}")
        return

    with gzip.open(SEED_FILE, "rt", encoding="utf-8") as fh:
        seed = json.load(fh)
    pages = seed["pages"]

    restore_guest_access(wiki, seed["guestGroup"])
    import_pages(wiki, pages)

    have = existing_pages(wiki)
    missing = [p["path"] for p in pages if (p["locale"], p["path"]) not in have]
    if missing:
        die(f"{len(missing)} pages are still missing, for example {missing[:3]}. Run it again.")

    rerender_all(wiki)
    restore_dates(pages)
    restore_menu_and_title(wiki, seed)

    note = json.dumps({"v": f"{len(pages)} pages from {seed.get('exportedAt', '?')}"})
    sql(f"INSERT INTO settings (`key`, value, updatedAt) VALUES ('{DONE_KEY}', {sql_text(note)}, "
        f"{sql_text(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()))}) "
        "ON DUPLICATE KEY UPDATE value=VALUES(value), updatedAt=VALUES(updatedAt);")

    say(f"Done. Your wiki has {len(have)} pages.")
    print(f"   Open it at {SITE_URL}  and sign in with {ADMIN_EMAIL}")


if __name__ == "__main__":
    main()
