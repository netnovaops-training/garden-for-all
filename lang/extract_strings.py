#!/usr/bin/env python3
"""Find every sentence in garden.sh that goes through the language layer.

garden.sh shows text in English, Korean or both (see LANG_MODE in garden.sh).
The Korean words live in lang/ko.tsv as "English<TAB>Korean", looked up by
EXACT match. This script lists the English side: every text passed to
say / note / warn / fail / ask / confirm / offer_email / Lt / L / explain, and every line of
an "Lblock <<EOF" block, the way it looks when garden.sh runs.

  python lang/extract_strings.py            # static strings, one per line
  python lang/extract_strings.py --dynamic  # strings with $variables (cannot match)

Static  = no "$" in the source text, so the runtime text is known exactly.
Dynamic = contains "$..." (a variable or a command), so it changes at runtime
          and cannot be matched exactly; it stays in English.
"""
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
GARDEN = HERE.parent / "garden.sh"

TARGETS = {"say", "note", "warn", "fail", "ask", "confirm", "Lt", "L", "explain", "ask_secret", "offer_email"}
KEYWORDS = {"then", "else", "elif", "do", "if", "while", "until", "!", "time"}
HEREDOC_RE = re.compile(r"(?<!<)<<-?\s*(['\"]?)([A-Za-z_]+)\1")


def strip_heredocs(lines):
    """Blank out here-document bodies; return (lines, Lblock bodies)."""
    out, blocks, i = [], [], 0
    while i < len(lines):
        line = lines[i]
        out.append(line)
        m = HEREDOC_RE.search(line)
        if m and not line.lstrip().startswith("#"):
            quoted, word = bool(m.group(1)), m.group(2)
            body = []
            i += 1
            while i < len(lines) and lines[i] != word:
                body.append(lines[i])
                out.append("")
                i += 1
            if i < len(lines):
                out.append("")  # the closing EOF line
            if re.search(r"\bLblock\s*<<", line):
                blocks.append((quoted, body))
        i += 1
    return out, blocks


class Scanner:
    """A small shell reader: finds calls of TARGETS and reads their arguments."""

    def __init__(self, text):
        self.t = text
        self.calls = []  # (name, [(value, dynamic)])

    # --- skipping helpers; each returns the index just after the construct
    def skip_single(self, i):          # at the opening '
        j = self.t.index("'", i + 1)
        return j + 1

    def skip_ansi(self, i):            # at the $ of $'...'
        j = i + 2
        while self.t[j] != "'":
            j += 2 if self.t[j] == "\\" else 1
        return j + 1

    def skip_double(self, i, scan=True):   # at the opening "
        j = i + 1
        while self.t[j] != '"':
            c = self.t[j]
            if c == "\\":
                j += 2
            elif c == "$" and self.t[j + 1] == "(":
                j = self.skip_subst(j, scan)
            elif c == "$" and self.t[j + 1] == "{":
                j = self.skip_brace(j, scan)
            elif c == "`":
                j = self.t.index("`", j + 1) + 1
            else:
                j += 1
        return j + 1

    def skip_subst(self, i, scan=True):    # at the $ of $(...)
        start = i + 2
        end = self.match_close(start, "(", ")")
        if scan:
            self.scan(start, end)
        return end + 1

    def skip_brace(self, i, scan=True):    # at the $ of ${...}
        start = i + 2
        end = self.match_close(start, "{", "}")
        if scan:
            self.scan(start, end)
        return end + 1

    def match_close(self, j, op, cl):
        depth = 1
        while True:
            c = self.t[j]
            if c == "\\":
                j += 2
                continue
            if c == "'" and op == "(":
                j = self.skip_single(j)
                continue
            if c == '"':
                j = self.skip_double(j, scan=False)
                continue
            if c == "$" and self.t[j + 1] == "'":
                j = self.skip_ansi(j)
                continue
            if c == "$" and self.t[j + 1] in "({":
                j = self.match_close(j + 2, self.t[j + 1], ")" if self.t[j + 1] == "(" else "}") + 1
                continue
            if c == "#" and op == "(" and (j == 0 or self.t[j - 1] in " \t\n"):
                j = self.t.index("\n", j)
                continue
            if c == op:
                depth += 1
            elif c == cl:
                depth -= 1
                if depth == 0:
                    return j
            j += 1

    # --- reading the arguments of one call
    def read_args(self, i, end):
        args = []
        while i < end:
            c = self.t[i]
            if c in " \t":
                i += 1
                continue
            if c == "\\" and self.t[i + 1] == "\n":
                i += 2
                continue
            if c in "\n;&|)}<>" or c == "#":
                break
            # one word
            val, dyn = [], False
            while i < end and self.t[i] not in " \t\n;&|)}<>":
                c = self.t[i]
                if c == "'":
                    j = self.skip_single(i)
                    val.append(self.t[i + 1:j - 1])
                    i = j
                elif c == '"':
                    j = self.skip_double(i)
                    raw = self.t[i + 1:j - 1]
                    if "$" in raw or "`" in raw:
                        dyn = True
                    val.append(re.sub(r'\\([$`"\\\n])', r"\1", raw))
                    i = j
                elif c == "$":
                    dyn = True
                    if self.t[i + 1] == "(":
                        j = self.skip_subst(i)
                    elif self.t[i + 1] == "{":
                        j = self.skip_brace(i)
                    elif self.t[i + 1] == "'":
                        j = self.skip_ansi(i)
                    else:
                        j = i + 1
                    val.append(self.t[i:j])
                    i = j
                elif c == "\\":
                    val.append(self.t[i + 1])
                    i += 2
                else:
                    val.append(c)
                    i += 1
            word = "".join(val)
            if re.fullmatch(r"\d+", word) and i < end and self.t[i] in "<>":
                break  # a redirect like 2>/dev/null
            args.append((word, dyn))
        return args, i

    # --- the main walk
    def scan(self, i, end):
        at_cmd = True
        while i < end:
            c = self.t[i]
            if c in " \t":
                i += 1
            elif c == "\\" and i + 1 < end and self.t[i + 1] == "\n":
                i += 2
            elif c == "#" and (i == 0 or self.t[i - 1] in " \t\n;"):
                nl = self.t.find("\n", i)
                i = end if nl < 0 or nl > end else nl
            elif c == "'":
                i = self.skip_single(i); at_cmd = False
            elif c == '"':
                i = self.skip_double(i); at_cmd = False
            elif c == "$" and i + 1 < end and self.t[i + 1] == "'":
                i = self.skip_ansi(i); at_cmd = False
            elif c == "$" and i + 1 < end and self.t[i + 1] == "(":
                i = self.skip_subst(i); at_cmd = False
            elif c == "$" and i + 1 < end and self.t[i + 1] == "{":
                i = self.skip_brace(i); at_cmd = False
            elif c in "\n;&|({)`":
                i += 1; at_cmd = True
            elif c in "}<>":
                i += 1; at_cmd = c == "}"
            else:
                m = re.compile(r"[^\s;&|(){}<>'\"$`]+").match(self.t, i)
                if not m:          # a lone $var or a backslash escape
                    i += 2 if c == "\\" else 1
                    at_cmd = False
                    continue
                word = m.group(0)
                j = m.end()
                if at_cmd and word in TARGETS and j < end and self.t[j] in " \t":
                    args, i = self.read_args(j, end)
                    self.calls.append((word, args))
                    at_cmd = False
                    continue
                at_cmd = word in KEYWORDS
                i = j
        return i


def collect():
    lines = GARDEN.read_text(encoding="utf-8").split("\n")
    body_lines, blocks = strip_heredocs(lines)
    sc = Scanner("\n".join(body_lines) + "\n")
    sc.scan(0, len(sc.t))

    static, dynamic = [], []

    def add(text, dyn, source):
        if not text.strip():
            return
        (dynamic if dyn else static).append((text, source))

    for name, args in sc.calls:
        if not args:
            continue
        if name in ("say", "warn", "fail"):
            add(" ".join(a for a, _ in args), any(d for _, d in args), name)
        elif name == "note":
            add(" ".join(a for a, _ in args).lstrip(" "), any(d for _, d in args), name)
        elif name == "L":
            add(args[0][0].lstrip(" "), args[0][1], name)
        elif name in ("Lt", "ask", "confirm", "ask_secret", "offer_email"):
            add(args[0][0], args[0][1], name)
        elif name == "explain":
            for a, d in args[:3]:
                add(a, d, name)

    for quoted, body in blocks:
        for line in body:
            core = line.lstrip(" ")
            add(core, (not quoted) and ("$" in line or "`" in line), "Lblock")

    # Indirect: the API key labels go through  Lt "$label"  and  say "$label",
    # and the key state through  Lt "$k"  (k="not set").
    src = GARDEN.read_text(encoding="utf-8")
    m = re.search(r"^KEYS_LIST=\((.*?)^\)", src, re.S | re.M)
    if m:
        for entry in re.findall(r'"([^"]*)"', m.group(1)):
            add(entry.split("|")[1], False, "KEYS_LIST")
    # The install step names go through  Lt "$what"  (what = the part after |).
    m = re.search(r"^INSTALL_STEPS=\((.*?)^\)", src, re.S | re.M)
    if m:
        for entry in re.findall(r'"([^"]*)"', m.group(1)):
            add(entry.split("|", 1)[1], False, "INSTALL_STEPS")
    if 'k="not set"' in src:
        add("not set", False, "do_keys")

    # Pure plumbing like  Lt "$1"  or  note "$d"  is not a sentence: drop it.
    dynamic = [(t, s) for t, s in dynamic
               if re.sub(r"\$(\{[^}]*\}|\([^)]*\)|[\w*@#?!]+)", "", t).strip(" :")]

    def dedup(items):
        seen, out = set(), []
        for t, s in items:
            if t not in seen:
                seen.add(t)
                out.append((t, s))
        return out

    return dedup(static), dedup(dynamic)


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8", newline="\n")
    static, dynamic = collect()
    for t, s in (dynamic if "--dynamic" in sys.argv else static):
        print(f"{s}\t{t}" if "--source" in sys.argv else t)
