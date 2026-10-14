"""Turn a piece of real terminal output (with its colours) into a picture of a
terminal window, for the guides.

    python tools/release/term_shot.py OUTPUT.png "Window title" < captured-output.txt

The text is exactly what the program printed; only its colours are drawn.
Needs Google Chrome or Microsoft Edge (to draw the picture).

Marks for the guides: text between ⟦ and ⟧ gets an orange ring around it,
and ⟦text⟧{a note} also writes "← a note" next to it, so a reader sees at once
where to look or what to type.
"""
import html, os, re, subprocess, sys, tempfile, pathlib

COLORS = {30: "#3b4252", 31: "#e06c75", 32: "#8fd19b", 33: "#e5c07b", 34: "#7aa2f7", 35: "#c49ce4",
          36: "#7dcfff", 37: "#e6e6e6", 90: "#7f8794", 91: "#ff7b85", 92: "#a6e3a1", 93: "#f2d58b",
          94: "#89b4fa", 95: "#d7aefb", 96: "#94e2d5", 97: "#ffffff"}
BROWSERS = (r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe", "/usr/bin/google-chrome", "/usr/bin/chromium")


def ansi_to_html(text):
    out, style = [], {"fg": None, "bold": False, "dim": False}
    for part in re.split(r"(\x1b\[[0-9;]*m)", text):
        m = re.fullmatch(r"\x1b\[([0-9;]*)m", part)
        if m:
            for c in [int(x) if x else 0 for x in m.group(1).split(";")]:
                if c == 0:
                    style = {"fg": None, "bold": False, "dim": False}
                elif c == 1:
                    style["bold"] = True
                elif c == 2:
                    style["dim"] = True
                elif c in COLORS:
                    style["fg"] = COLORS[c]
                elif c == 39:
                    style["fg"] = None
            continue
        if not part:
            continue
        css = []
        if style["fg"]:
            css.append(f"color:{style['fg']}")
        if style["bold"]:
            css.append("font-weight:700")
        if style["dim"]:
            css.append("opacity:.7")
        esc = html.escape(part)
        out.append(f'<span style="{";".join(css)}">{esc}</span>' if css else esc)
    return "".join(out)


def marks(h):
    """⟦text⟧ -> an orange ring; ⟦text⟧{note} -> the ring and "← note" beside it."""
    return re.sub(r"⟦(.*?)⟧(?:\{([^}]*)\})?",
                  lambda m: f'<span class="mark">{m.group(1)}</span>' + (f'<span class="note">← {m.group(2)}</span>' if m.group(2) else ""), h)


def main():
    out_png, title = sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "jun@garden-lab: ~/junsgarden-for-jun"
    text = sys.stdin.buffer.read().decode("utf-8", "replace").replace("\r", "")
    text = re.sub(r"\x1b\[[0-9;]*[A-HJKSTfsu]", "", text)        # cursor moves: not drawn
    lines = text.rstrip("\n").split("\n")
    width = 980
    height = 70 + 21 * len(lines) + 24
    page = f"""<!doctype html><meta charset="utf-8"><style>
body {{ margin:0; background:#e9ede6; padding:14px; }}
.win {{ background:#1e2420; border-radius:10px; box-shadow:0 6px 18px rgba(0,0,0,.25); overflow:hidden; }}
.bar {{ background:#2c332e; color:#c8d0c9; font:13px "Segoe UI", sans-serif; padding:8px 12px; display:flex; gap:7px; align-items:center; }}
.dot {{ width:11px; height:11px; border-radius:50%; background:#5c655e; }}
.bar span {{ margin-left:10px; }}
pre {{ margin:0; padding:12px 16px 16px; color:#e6e6e6; font:14px/21px "Cascadia Mono", Consolas, "Malgun Gothic", monospace; white-space:pre-wrap; }}
.mark {{ outline:3px solid #ff9a3c; outline-offset:3px; border-radius:8px; }}
.note {{ color:#ffb46b; font:700 14px "Segoe UI", "Malgun Gothic", sans-serif; margin-left:14px; }}
</style><div class="win"><div class="bar"><i class="dot"></i><i class="dot"></i><i class="dot"></i><span>{html.escape(title)}</span></div>
<pre>{marks(ansi_to_html(chr(10).join(lines)))}</pre></div>"""
    tmp = pathlib.Path(tempfile.mkdtemp()) / "term.html"
    tmp.write_text(page, encoding="utf-8")
    browser = next((b for b in BROWSERS if os.path.exists(b)), None) or sys.exit("needs Chrome or Edge")
    subprocess.run([browser, "--headless=new", "--disable-gpu", "--hide-scrollbars", f"--window-size={width + 28},{height + 28}",
                    f"--screenshot={os.path.abspath(out_png)}", tmp.as_uri()], check=True,
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    print("made", out_png, f"({len(lines)} lines)")


if __name__ == "__main__":
    main()
