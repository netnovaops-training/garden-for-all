#!/bin/bash
# Test-only: tools/get-pdfs.py (and tools/get-pdfs.ps1, when PowerShell is there).
# Uses made-up PDFs and file:// downloads in a throwaway folder; nothing real is touched.
#   bash tools/release/tests/test-get-pdfs.sh
# Checks: a good download is unpacked and the status notes are right; a wrong fingerprint,
# a wrong name inside the zip and a missing download each write NOTHING (and leave the
# status as "not downloaded"); "none" writes only the status; a second run refreshes.
set -u
HERE=$(cd "$(dirname "$0")/../../.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
export PYTHONIOENCODING=utf-8
PY=python3; python3 -c "" >/dev/null 2>&1 || PY=python   # on Windows, python3 can be a store shortcut
pass=0; fail=0
ok() { if eval "$2"; then pass=$((pass + 1)); echo "PASS  $1"; else fail=$((fail + 1)); echo "FAIL  $1"; fi; }

# Made-up zips, the list that names them, and a copy of the scripts in a fake garden folder.
mkdir -p "$T/serve" "$T/garden/tools"
cp "$HERE/tools/get-pdfs.py" "$HERE/tools/get-pdfs.ps1" "$HERE/tools/get-parts.py" "$T/garden/tools/"
$PY - "$T" <<'PY'
import hashlib, sys, zipfile
t = sys.argv[1]
def mk(name, folder, members):
    with zipfile.ZipFile(f"{t}/serve/{name}", "w") as z:
        for m in members:
            z.writestr(f"{folder}/{m}", b"%PDF-1.4 test " + m.encode("utf-8"))
mk("junsgarden-pdf-guides-english.zip", "English", ["0 - Start here/FULL - 00 a.pdf", "1 - Two/QUICK - 01 b.pdf"])
mk("junsgarden-pdf-guides-korean.zip", "Korean", ["0 - 여기서 시작/FULL - 00 a.pdf"])
lines = ["url=file:///x"]
for code, name in (("en", "junsgarden-pdf-guides-english.zip"), ("ko", "junsgarden-pdf-guides-korean.zip")):
    b = open(f"{t}/serve/{name}", "rb").read()
    lines.append(f"{hashlib.sha256(b).hexdigest()}  {name} {len(b)} {code}")
open(f"{t}/garden/PDF-GUIDES-SHA256.txt", "w").write("\n".join(lines) + "\n")
# A zip with a name that tries to leave the folder, listed with its REAL fingerprint:
with zipfile.ZipFile(f"{t}/serve/evil-english.zip", "w") as z:
    z.writestr("English/../../escape.pdf", b"x")
    z.writestr("English/0 - a/ok.pdf", b"x")
b = open(f"{t}/serve/evil-english.zip", "rb").read()
open(f"{t}/garden/PDF-GUIDES-EVIL.txt", "w").write(
    "url=file:///x\n" + f"{hashlib.sha256(b).hexdigest()}  evil-english.zip {len(b)} en\n" + lines[2] + "\n")
PY
# A file:// address for a folder, in a form Windows programs understand too.
furl() { local d; d=$(cygpath -m "$1" 2>/dev/null || echo "$1"); case "$d" in /*) echo "file://$d" ;; *) echo "file:///$d" ;; esac; }
G="$T/garden"; URL=$(furl "$T/serve")
run() { $PY "$G/tools/get-pdfs.py" "$@" > "$T/out.txt" 2>&1; }

D1="$T/d1"; mkdir -p "$D1"
run en --dest "$D1" --url-base "$URL"
ok "a good English download exits 0" "[ $? -eq 0 ]"
ok "its PDFs are in PDF guides/English (2 files)" "[ \$(find \"$D1/PDF guides/English\" -name '*.pdf' | wc -l) -eq 2 ]"
ok "STATUS.txt: English ready, Korean not downloaded" "grep -q '^English: ready (2 PDFs)' \"$D1/PDF guides/STATUS.txt\" && grep -q '^Korean: not downloaded - use menu item P' \"$D1/PDF guides/STATUS.txt\""
ok "the _arriving.txt note is gone once the set is in" "[ -z \"\$(find \"$D1\" -name '_arriving.txt')\" ]"
run ko --dest "$D1" --url-base "$URL"
ok "Korean (Korean folder names) is unpacked too, English stays ready" "[ \$(find \"$D1/PDF guides/Korean\" -name '*.pdf' | wc -l) -eq 1 ] && grep -q '^English: ready' \"$D1/PDF guides/STATUS.txt\" && grep -q '^Korean: ready (1 PDFs)' \"$D1/PDF guides/STATUS.txt\""
run both --dest "$D1" --url-base "$URL"
ok "a second run (refresh) works" "[ $? -eq 0 ]"

# A wrong fingerprint: tamper with a zip AFTER the list was written.
D2="$T/d2"; mkdir -p "$D2"; cp -R "$T/serve" "$T/serve-bad"; printf 'x' >> "$T/serve-bad/junsgarden-pdf-guides-english.zip"
run en --dest "$D2" --url-base "$(furl "$T/serve-bad")"
rc=$?
ok "a wrong fingerprint exits 1" "[ $rc -ne 0 ]"
ok "a wrong fingerprint writes NO pdf and says so in plain words" "[ -z \"\$(find \"$D2\" -name '*.pdf')\" ] && grep -q 'did not pass the check' \"$T/out.txt\" && grep -q 'nothing was written' \"$T/out.txt\""
ok "the status says not downloaded and a note points to menu P" "grep -q '^English: not downloaded - use menu item P' \"$D2/PDF guides/STATUS.txt\" && [ -f \"$D2/PDF guides/English/_not-downloaded.txt\" ] && [ ! -f \"$D2/PDF guides/English/_arriving.txt\" ]"

# A name that tries to leave the folder.
D3="$T/d3"; mkdir -p "$D3"
$PY "$G/tools/get-pdfs.py" en --dest "$D3" --url-base "$URL" --list "$G/PDF-GUIDES-EVIL.txt" > "$T/out.txt" 2>&1
ok "a zip with a name that leaves the folder is refused, nothing written, nothing escaped" "[ -z \"\$(find \"$D3\" -name '*.pdf')\" ] && [ ! -e \"$T/escape.pdf\" ] && [ ! -e \"$D3/escape.pdf\" ] && grep -q 'unexpected file name' \"$T/out.txt\""

# No connection (nothing there to download).
D4="$T/d4"; mkdir -p "$D4"
run en --dest "$D4" --url-base "$(furl "$T/nothing-here")"
rc=$?
ok "no download: exit 1, plain message, no pdf" "[ $rc -ne 0 ] && grep -q 'Could not download' \"$T/out.txt\" && [ -z \"\$(find \"$D4\" -name '*.pdf')\" ]"

# "none".
D5="$T/d5"; mkdir -p "$D5"
run none --dest "$D5" --url-base "$URL"
rc=$?
ok "'none' writes only the status (both sets: not downloaded)" "[ $rc -eq 0 ] && [ -z \"\$(find \"$D5\" -name '*.pdf')\" ] && [ \$(grep -c 'not downloaded' \"$D5/PDF guides/STATUS.txt\") -eq 2 ]"
run none --dest "$D1" --url-base "$URL"
ok "'none' leaves a set that is already in as ready" "grep -q '^English: ready' \"$D1/PDF guides/STATUS.txt\""

# --check (used by the release build).
run --check "$T/serve/junsgarden-pdf-guides-english.zip" en
ok "--check says ALL MATCH for the real zip" "grep -q 'ALL MATCH' \"$T/out.txt\""
run --check "$T/serve-bad/junsgarden-pdf-guides-english.zip" en
rc=$?
ok "--check refuses a changed zip" "[ $rc -ne 0 ] && ! grep -q 'ALL MATCH' \"$T/out.txt\""

# The same for PowerShell, when it is there (Windows).
if command -v powershell.exe >/dev/null 2>&1; then
    W() { cygpath -w "$1" 2>/dev/null || echo "$1"; }
    U() { echo "file:///$(W "$1" | tr '\\' '/')"; }
    D6="$T/d6"; mkdir -p "$D6"
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(W "$G/tools/get-pdfs.ps1")" -Lang ko -Dest "$(W "$D6")" -UrlBase "$(U "$T/serve")" > "$T/out.txt" 2>&1
    ok "PowerShell: a good Korean download is unpacked (Korean folder names)" "[ \$(find \"$D6/PDF guides/Korean\" -name '*.pdf' | wc -l) -eq 1 ] && grep -q '^Korean: ready' \"$D6/PDF guides/STATUS.txt\""
    D7="$T/d7"; mkdir -p "$D7"
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(W "$G/tools/get-pdfs.ps1")" -Lang en -Dest "$(W "$D7")" -UrlBase "$(U "$T/serve-bad")" > "$T/out.txt" 2>&1
    ok "PowerShell: a wrong fingerprint writes no pdf and says so" "[ -z \"\$(find \"$D7\" -name '*.pdf')\" ] && grep -q 'did not pass the check' \"$T/out.txt\" && grep -q '^English: not downloaded' \"$D7/PDF guides/STATUS.txt\""
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(W "$G/tools/get-pdfs.ps1")" -Lang ko -Dest "$(W "$D6")" -UrlBase "$(U "$T/serve-bad")" > "$T/out.txt" 2>&1
    ok "PowerShell: a second run is 'already up to date' and fetches nothing (even from a bad address)" "grep -q 'already up to date' \"$T/out.txt\""
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(W "$G/tools/get-pdfs.ps1")" -Lang ko -Dest "$(W "$D6")" -IsCurrent > "$T/out.txt" 2>&1
    ok "PowerShell: -IsCurrent exits 0 when current" "[ $? -eq 0 ]"
    # A second host: the first address (GitHub's) is dead, the list names "author" for the file.
    mkdir -p "$T/garden2/tools"; cp "$G/tools/get-pdfs.ps1" "$T/garden2/tools/"
    sed 's#^\(.* ko\)$#\1 github,author#' "$G/PDF-GUIDES-SHA256.txt" > "$T/garden2/PDF-GUIDES-SHA256.txt"
    printf 'author  %s/{name}\n' "$(U "$T/serve")" > "$T/garden2/DOWNLOAD-HOSTS.txt"
    D8="$T/d8"; mkdir -p "$D8"
    powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(W "$T/garden2/tools/get-pdfs.ps1")" -Lang ko -Dest "$(W "$D8")" -UrlBase "$(U "$T/nothing-here")" > "$T/out.txt" 2>&1
    ok "PowerShell: the first host is dead, the second host gives the file" "[ \$(find \"$D8/PDF guides/Korean\" -name '*.pdf' | wc -l) -eq 1 ] && grep -q '^Korean: ready' \"$D8/PDF guides/STATUS.txt\""
fi
echo "RESULT: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
