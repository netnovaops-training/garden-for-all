#!/bin/bash
# shellcheck disable=SC2016,SC2012  # each check runs in its own bash -c on purpose; ls only shows a folder
# Test-only: run "garden.sh webname" on a throwaway copy (no .env = before install).
set -u
SRC=${GARDEN_SRC:?set GARDEN_SRC to the package folder, for example ~/junsgarden-for-jun}
T=$(mktemp -d); cp -r "$SRC/." "$T/"; rm -rf "$T/backups" "$T/PDF" "$T/.env"
cd "$T" && git init -q -b main . && git add -A && git -c user.name=t -c user.email=t@t commit -qm start
P=0; F=0
chk() { local n=$1; shift; if "$@"; then echo "PASS  $n"; P=$((P+1)); else echo "FAIL  $n"; F=$((F+1)); fi; }
printf 'HTTPS://WWW.MomsGarden.zapto.org/beds\ny\n' | bash garden.sh webname > /tmp/wn.log 2>&1
sed 's/\x1b\[[0-9;]*m//g' /tmp/wn.log | grep -E "Using:|changed|Saved|kept" | head -5
chk "messy input cleaned to momsgarden.zapto.org" grep -q "Using: momsgarden.zapto.org" /tmp/wn.log
chk "no file still says junsgarden.zapto.org" bash -c '! grep -rlF junsgarden.zapto.org site/privacy.html site/apps/version.json rust/junsgarden/src rust/junsgarden/templates beds/freeai.py android-app/app/build.gradle.kts android-app/app/src/main/kotlin/org/jasonagibson/junsgarden/app/Updater.kt'
chk "app update file points at the new name" grep -q '"https://momsgarden.zapto.org/apps/junsgarden.apk"' site/apps/version.json
chk "Rust test string changed consistently" grep -q 'urlencode("https://momsgarden.zapto.org/"), "https%3A%2F%2Fmomsgarden.zapto.org%2F"' rust/junsgarden/src/php.rs
chk "old copies kept" bash -c 'ls logs/web-name-change_*/site/apps/version.json >/dev/null'
chk "history notes left alone" grep -q junsgarden.zapto.org CHANGES.md
chk "saved in git with a message" bash -c 'git log -1 --format=%s | grep -q "from junsgarden.zapto.org to momsgarden.zapto.org"'
chk "only the listed files changed" bash -c '[ "$(git show --stat --format= HEAD | grep -c "|")" -le 12 ]'
printf 'not a name!\n' | bash garden.sh webname > /tmp/wn2.log 2>&1
chk "a bad name is refused" grep -q "does not look like a web name" /tmp/wn2.log
printf '\n' | bash garden.sh webname > /tmp/wn3.log 2>&1
chk "empty answer changes nothing" grep -q "Nothing changed" /tmp/wn3.log
echo "RESULT: $P passed, $F failed"; rm -rf "$T"
