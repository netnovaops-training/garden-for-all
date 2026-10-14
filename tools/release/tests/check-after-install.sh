#!/bin/bash
# shellcheck disable=SC2016,SC2012  # each check runs in its own bash -c on purpose; ls only shows a folder
# Test-only: what the install left behind, then switch the wiki on like Jun would.
cd ~/junsgarden-for-jun || { echo "NO GARDEN FOLDER"; exit 1; }
P=0; F=0
chk() { local n=$1; shift; if "$@"; then echo "PASS  $n"; P=$((P+1)); else echo "FAIL  $n"; F=$((F+1)); fi; }
chk "all 14 install steps done" grep -q "DONE    step 14/14" logs/install-progress.txt
chk "website healthy with 485 plants" bash -c 'curl -fs http://127.0.0.1:8030/healthz | grep -q "\"plant_count\":485"'
chk "photos loaded from the photos download (164 test photos)" bash -c '[ "$(find local/images -type f | wc -l)" -ge 164 ]'
chk "a photo is served" bash -c 'f=$(ls local/images/Database | head -1); curl -fs -o /dev/null "http://127.0.0.1:8030/images/Database/$f"'
chk "garden tools on (Easy)" bash -c 'grep -q "^COMPOSE_PROFILES=.*beds" .env'
chk "wiki OFF by default (Easy)" bash -c '! grep -q "^COMPOSE_PROFILES=.*wiki" .env && ! sudo docker ps --format "{{.Names}}" | grep -qx garden-wiki'
chk "web name swapped during setup" bash -c 'grep -q "https://testgarden.zapto.org/apps/junsgarden.apk" site/apps/version.json'
printf '2\nA\n' | GARDEN_TEST_WIKI_PASSWORD="WikiAdmin-Test-2026" bash garden.sh parts > /tmp/parts.log 2>&1
sed 's/\x1b\[[0-9;]*m//g' /tmp/parts.log | grep -E "wiki add-on|fingerprint|Adding pages|Done. Your wiki|EMPTY" | head -5
chk "wiki switched on and answers" bash -c 'for i in $(seq 1 40); do curl -fs -o /dev/null http://127.0.0.1:3000/ && exit 0; sleep 3; done; exit 1'
chk "wiki got the 40 pages from the wiki download" bash -c '[ "$(sudo docker exec garden-db sh -c "MYSQL_PWD=\"\$MARIADB_ROOT_PASSWORD\" mariadb -uroot -N -e \"SELECT COUNT(*) FROM junsgarden_wiki.pages\"")" -ge 40 ]'
echo "RESULT: $P passed, $F failed"
