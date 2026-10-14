#!/bin/bash
# Test-only: the health check's "Backups" section before an install (no .env), and the
# address the install hands to the browser for the overview (spaces, brackets, & ' and Korean).
# Uses a copy of garden.sh in a throwaway folder and a fake xdg-open; nothing real is touched.
#   bash tools/release/tests/test-final-fixes.sh
set -u
HERE=$(cd "$(dirname "$0")/../../.." && pwd)
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
pass=0; fail=0
ok() { if eval "$2"; then pass=$((pass + 1)); echo "PASS  $1"; else fail=$((fail + 1)); echo "FAIL  $1"; fi; }

# 1. check before the install: no .env, but an old backup file lying in ./backups
mkdir -p "$T/g/backups"; cp "$HERE/garden.sh" "$T/g/"; cp -r "$HERE/lang" "$T/g/" 2>/dev/null
touch "$T/g/backups/junsgarden-backup_2026-01-01_0000.tar"
( cd "$T/g" && bash garden.sh check >"$T/out1.txt" 2>&1 )
ok "check without .env prints no stat error"       "! grep -qi 'cannot stat' '$T/out1.txt'"
ok "check without .env prints no syntax error"     "! grep -qi 'syntax error' '$T/out1.txt'"
ok "the backup section still answers in words"     "grep -q '== 5. Backups' '$T/out1.txt' && sed -n '/== 5. Backups/,/== 6./p' '$T/out1.txt' | grep -qiE 'Newest backup|No backups yet'"
rm -rf "$T/g/backups"
( cd "$T/g" && bash garden.sh check >"$T/out2.txt" 2>&1 )
ok "no backup folder at all: says No backups yet"  "sed -n '/== 5. Backups/,/== 6./p' '$T/out2.txt' | grep -q 'No backups yet'"
ok "no backup folder at all: no stat/syntax error" "! grep -qiE 'cannot stat|syntax error' '$T/out2.txt'"

# 2. the overview address: take file_url out of garden.sh and try awkward paths
sed -n '/^file_url() {/,/^}/p' "$HERE/garden.sh" > "$T/fn.sh"
ok "file_url was found in garden.sh" "[ -s '$T/fn.sh' ]"
. "$T/fn.sh"
for p in "/home/jun/Downloads/junsgarden-for-jun (1)/handbook/00-OVERVIEW.html" \
         "/home/jun/a&b/it's/x.html" "/home/jun/정원 폴더/한국어.html" "/home/jun/plain/00-OVERVIEW.html"; do
    u=$(file_url "$p")
    back=$(python3 - "$u" <<'PY'
import sys, urllib.parse
u = sys.argv[1]
assert u.startswith("file://")
print(urllib.parse.unquote(u[7:], errors="strict"))
PY
)
    same=0; [ "$back" = "$p" ] && same=1
    ok "round trip: $p" "[ $same = 1 ]"
    left=0; printf '%s' "${u#file://}" | grep -q "[][ ()&'?#]" && left=1
    ok "no space, bracket, quote or & left in the address" "[ $left = 0 ]"
done
ok "the plain path stays readable" "[ \"\$(file_url /home/jun/plain/00-OVERVIEW.html)\" = 'file:///home/jun/plain/00-OVERVIEW.html' ]"

# 3. open_overview hands that address to xdg-open
mkdir -p "$T/bin" "$T/o/ünï (1)/handbook"; touch "$T/o/ünï (1)/handbook/00-OVERVIEW.html"
printf '#!/bin/sh\nprintf "%%s\n" "$1" > "%s/arg.txt"\n' "$T" > "$T/bin/xdg-open"; chmod +x "$T/bin/xdg-open"
sed -n '/^open_overview() {/,/^}/p' "$HERE/garden.sh" > "$T/fn2.sh"
note() { :; }
( . "$T/fn2.sh"; . "$T/fn.sh"; cd "$T/o/ünï (1)" && HERE=$PWD DISPLAY=:0 ASSUME_YES=0 LANG_MODE=en PATH="$T/bin:$PATH" open_overview; sleep 1 )
ok "xdg-open got an encoded file:// address" "grep -q '^file:///.*%C3%BC.*%20%281%29/handbook/00-OVERVIEW.html\$' '$T/arg.txt'"

echo; echo "$pass passed, $fail failed"; [ "$fail" = 0 ]
