"""Draw the pictures of the newest menus for the guides, from the real output
that capture-menus.sh saved (with sample names already in).

    python tools/release/make_menu_pictures.py CAPTURE-FOLDER

When a script types the answers, the program's questions do not show (bash
shows a question only to a person at a keyboard), so each question is put back
with its answer after it, exactly as the screen shows it when a person types.
Marks ⟦..⟧{note} draw the orange rings (term_shot.py). Any email address in the
output is replaced by the sample you@example.com.
"""
import io, re, subprocess, sys, pathlib

if len(sys.argv) < 2:
    sys.exit("give the folder capture-menus.sh wrote")
S = pathlib.Path(sys.argv[1])
PKG = pathlib.Path(__file__).resolve().parents[2]
IMG = PKG / "START-HERE" / "images"
SHOT = PKG / "tools" / "release" / "term_shot.py"
ESC = re.compile(r"\x1b\[[0-9;]*[HJ]")          # screen-clear codes (colours stay)
EMAIL = re.compile(r"[\w.+-]+@[\w-]+\.[\w.]+")


def text(name):
    t = io.open(S / name, encoding="utf-8", errors="replace").read().replace("\r", "")
    return EMAIL.sub("you@example.com", ESC.sub("", t))


def plain(line):
    return re.sub(r"\x1b\[[0-9;]*m", "", line)


def mark(t, target, note=None):
    """Put a ring around the first line part that reads `target`."""
    lines = t.split("\n")
    for i, l in enumerate(lines):
        p = plain(l)
        if target in p:
            lead, rest = p[: p.index(target)], p[p.index(target) + len(target):]
            lines[i] = lead + "⟦" + target + "⟧" + ("{" + note + "}" if note else "") + rest
            return "\n".join(lines)
    sys.exit(f"not found: {target}")


def after(t, target, new_lines):
    """Add lines right after the first line that reads `target`."""
    lines = t.split("\n")
    for i, l in enumerate(lines):
        if target in plain(l):
            lines[i + 1:i + 1] = new_lines
            return "\n".join(lines)
    sys.exit(f"not found: {target}")


def shoot(png, t):
    t = re.sub(r"\n{3,}", "\n\n", t).strip("\n") + "\n"
    if subprocess.run([sys.executable, str(SHOT), str(IMG / png)], input=t.encode("utf-8")).returncode:
        sys.exit(f"term_shot failed for {png}")


# More menu: only the More screen, the two new items ringed.
t = text("more.txt")
t = mark(t, "14) On / off: the whole garden, or only off the internet; check-up, clean-up", "new")
t = mark(t, "18) Rename or move a file or folder (every place that uses it follows)", "new")
shoot("screen-02-more-menu.png", t + "\n  Type a number, then press Enter: 18")

# Settings, item 8 ringed.
t = mark(text("settings.txt"), "8) Rename or move a file or folder")
shoot("screen-10-settings.png", t)

# The rename tool's first screen.
t = mark(text("rename-menu.txt"), "1) A file or folder of the website", "type 1")
shoot("screen-25-rename-menu.png", t + "\nNumber [0]: 1")

# The preview, with the two answers typed.
t = text("rename-preview.txt")
t = after(t, "site/privacy.html           a page", ["File or folder (empty = back): ⟦local/images/Gallery⟧{Tab helps}"])
t = after(t, "Letters, numbers, - _ and . only", ["New name (empty = back): ⟦Photos⟧"])
t = t.split("   Nothing changed.")[0]
t = mark(t, "Code and pages: 8 place(s) in 4 file(s):", "read the list")
shoot("screen-26-rename-preview.png", t.rstrip("\n") + "\n⟦Change all of this now? (Y/n): y⟧{nothing changed until here}")

# The end of a real rename (the website build's own lines left out).
full = text("rename-full.txt")
keep = [l for l in full.split("\n") if not re.match(r"^(#\d|\s?Container |\s?Image |\s*$|\{)", plain(l))]
t = "\n".join(keep)
if "   Rename: local/images/Gallery" in t:
    t = t[t.index("   Rename: local/images/Gallery"):]
t = after(t, "logs/renames.tsv will list this change", ["Change all of this now? (Y/n): y",
          "Make a full backup first (recommended, a few minutes)? (Y/n): n   (skipped only for this picture)"])
checked = next(plain(l).strip() for l in t.split("\n") if "Checked: " in plain(l))
shoot("screen-27-rename-done.png", mark(t, checked, "it works"))

# The switch.
t = text("switch.txt")
t = mark(t, "1) Everything OFF: stops all the garden's programs and the front door.", "off, nothing deleted")
t = mark(t, "2) Everything ON again: starts it all and checks the website", "on again")
shoot("screen-28-switch.png", t + "\nNumber [0]: ")

# The clean-up, asking first.
t = mark(text("cleanup.txt"), "junsgarden/rs:old-version", "an old version")
shoot("screen-29-cleanup.png", after(t, "junsgarden/rs:old-version", ["⟦Remove these? (y/N): ⟧{it asks first; Enter = no}"]))
