#!/usr/bin/env bash
# The front door's settings for a moved garden: garden.sh's own write_caddyfile, with a folder
# (PUBLIC_PATH) and an old web name (OLD_NAMES), checked by Caddy itself and then run:
#   - the folder serves the website, the top of the web name still does too;
#   - an old name forwards to the new address, path and all, with or without the folder.
# Needs Docker (caddy image). From the garden folder:  bash tools/release/tests/test-caddy-address.sh
set -uo pipefail
cd "$(dirname "$0")/../../.." || exit 1
pass=0; fail=0
chk() { if eval "$2" >/dev/null 2>&1; then pass=$((pass+1)); echo "PASS $1"; else fail=$((fail+1)); echo "FAIL $1"; fi; }
work=$(mktemp -d); trap 'docker rm -f caddy-addr-test >/dev/null 2>&1; rm -rf "$work"' EXIT

# Only the functions that make the file (the rest of garden.sh is not run).
# shellcheck disable=SC1090
source <(sed -n -e '/^env_get()/p' -e '/^write_caddyfile()/,/^}$/p' -e '/^old_name_blocks()/,/^}$/p' garden.sh)
subdomain_blocks() { :; }
cp docker/caddy/Caddyfile.template "$work/"
mkdir -p "$work/docker/caddy" && mv "$work/Caddyfile.template" "$work/docker/caddy/"
cd "$work" || exit 1
printf 'PUBLIC_NAME=new.test\nPUBLIC_PATH=/garden\nPUBLIC_URL=https://new.test/garden\nOLD_NAMES=old.test\nCADDY_HTTP_PORT=8080\n' > .env
# Plain http for the test: every site address gets http:// (no certificates needed).
write_caddyfile new.test jun@example.test "" 0 | sed -E 's#^(new\.test|old\.test) \{#http://\1 {#' > Caddyfile
chk "garden.sh wrote a Caddyfile with the folder block" "grep -q 'handle_path /garden/\*' Caddyfile"
chk "and a block for the old name" "grep -q '^http://old.test {' Caddyfile"

img=caddy:2.10-alpine
chk "Caddy says the file is valid" "docker run --rm -v '$work/Caddyfile:/etc/caddy/Caddyfile:ro' $img caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile"

# Run it, with a stand-in website on 8030 that answers with the path it was asked for.
# (Its own network: nothing is published, and the computer's own 8030/8080 are never touched.)
docker run -d --name caddy-addr-test -v "$work/Caddyfile:/etc/caddy/Caddyfile:ro" $img >/dev/null
docker exec -d caddy-addr-test sh -c 'mkdir -p /w && cd /w && while true; do printf "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok" | nc -l -p 8030 >/dev/null; done'
sleep 2
# curl in a throwaway container that shares the test Caddy's network; prints the status and Location.
get() { docker run --rm --network container:caddy-addr-test curlimages/curl:8.17.0 -s -o /dev/null \
    -w '%{http_code} %{redirect_url}\n' -H "Host: $1" "http://127.0.0.1:8080$2"; }
chk "the folder serves the website (/garden/catalog.php)" "get new.test /garden/catalog.php | grep -q '^200 '"
chk "/garden goes to /garden/" "get new.test /garden | grep -q '^308 .*/garden/\$'"
chk "old name, no folder: /q/12 -> new.test/garden/q/12" "get old.test /q/12 | grep -qx '308 https://new.test/garden/q/12'"
chk "old name, with folder: /garden/b/3 -> new.test/garden/b/3 (not /garden/garden)" "get old.test /garden/b/3 | grep -qx '308 https://new.test/garden/b/3'"
chk "old name keeps the query: /catalog.php?plant=7" "get old.test '/catalog.php?plant=7' | grep -qx '308 https://new.test/garden/catalog.php?plant=7'"
# Phones not yet updated keep using the old name: their addresses are answered, not forwarded
# (a forwarded login POST is not repeated by Android, and the login key is dropped).
post() { docker run --rm --network container:caddy-addr-test curlimages/curl:8.17.0 -s -o /dev/null \
    -w '%{http_code}\n' -H "Host: $1" -H 'Authorization: Bearer x' -d 'username=Jun' "http://127.0.0.1:8080$2"; }
chk "old app, old name: login POST /app_api.php answered (200)" "post old.test '/app_api.php?action=login' | grep -qx 200"
chk "old app, old name: /app/garden/tag answered (200)" "post old.test /app/garden/tag | grep -qx 200"
chk "old app, old name: /apps/version.json answered (200)" "get old.test /apps/version.json | grep -q '^200 '"
chk "new app set up with the folder, old name: /garden/app_api.php answered (200)" "post old.test '/garden/app_api.php?action=whoami' | grep -qx 200"
chk "new app, old name: /app/config.json answered (200, then it moves itself)" "get old.test /app/config.json | grep -q '^200 '"
chk "but a page at the old name still forwards: /garden/catalog.php" "get old.test /garden/catalog.php | grep -qx '308 https://new.test/garden/catalog.php'"

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]
