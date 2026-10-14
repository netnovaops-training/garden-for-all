#!/bin/bash
# shellcheck disable=SC2016,SC2012  # each check runs in its own bash -c on purpose; ls only shows a folder
# Final test: run every garden.sh option non-interactively on a throwaway
# machine and check the result. Test-only. Run as jun in ~/junsgarden-for-jun.
cd ~/junsgarden-for-jun || exit 1
P=0; F=0; RES=/tmp/test-results.txt; : > "$RES"
ok()  { echo "PASS  $*" | tee -a "$RES"; P=$((P + 1)); }
bad() { echo "FAIL  $*" | tee -a "$RES"; F=$((F + 1)); }
chk() { local name=$1; shift; if "$@"; then ok "$name"; else bad "$name"; fi; }
g()   { bash garden.sh "$@" 2>&1 | sed 's/\x1b\[[0-9;]*m//g'; }
env_get() { grep -m1 "^$1=" .env | cut -d= -f2-; }
H=garden.test; C="curl -sk --resolve $H:18543:127.0.0.1"
PW='GardenTest-2026!'

echo "== 0. Easy left the wiki off; switch it on (its pages come from the wiki download in Home)"
chk "wiki off after an Easy install" bash -c '! grep -q "^COMPOSE_PROFILES=.*wiki" .env'
printf '2
A
' | GARDEN_TEST_WIKI_PASSWORD="WikiAdmin-Test-2026" g parts > /tmp/t.log
chk "wiki on, pages from the add-on" grep -q "Its fingerprint matches" /tmp/t.log

echo "== 1. Website and data"
chk "website healthy with 485 plants" bash -c 'curl -fs http://127.0.0.1:8030/healthz | grep -q "\"plant_count\":485"'
chk "garden tools answer" curl -fs -o /dev/null http://127.0.0.1:8025/healthz
chk "wiki answers" curl -fs -o /dev/null http://127.0.0.1:3000/
chk "wiki has pages (small test copy: 40 pages)" bash -c '[ "$(sudo docker exec garden-db sh -c "MYSQL_PWD=\"\$MARIADB_ROOT_PASSWORD\" mariadb -uroot -N -e \"SELECT COUNT(*) FROM junsgarden_wiki.pages\"")" -ge 40 ]'
chk "photos came from the photos download" bash -c '[ "$(find local/images -type f | wc -l)" -ge 164 ]'
chk "a garden photo is served" bash -c 'f=$(ls local/images/Database | head -1); curl -fs -o /dev/null "http://127.0.0.1:8030/images/Database/$f"'

echo "== 2. Password and website login"
echo "$PW" | g password Jun > /tmp/t.log
chk "password set" grep -q "Website password set for Jun" /tmp/t.log
rm -f /tmp/jar; curl -s -c /tmp/jar -b /tmp/jar -o /dev/null --data-urlencode "admin_login=1" --data-urlencode "admin_user=Jun" --data-urlencode "admin_password=$PW" http://127.0.0.1:8030/login.php
chk "website login opens the plant admin" bash -c '[ "$(curl -s -b /tmp/jar -o /dev/null -w "%{http_code}" http://127.0.0.1:8030/streamlit/)" = 200 ]'
chk "wrong password does not" bash -c 'rm -f /tmp/j2; curl -s -c /tmp/j2 -o /dev/null --data "admin_login=1&admin_user=Jun&admin_password=wrong-wrong-1" http://127.0.0.1:8030/login.php; [ "$(curl -s -b /tmp/j2 -o /dev/null -w "%{http_code}" http://127.0.0.1:8030/streamlit/)" != 200 ]'

echo "== 3. Checklist and a second install (skip what works)"
g check > /tmp/t.log
chk "checklist has no FAIL" bash -c '! grep -q "^FAIL" /tmp/t.log'
grep "^FAIL" /tmp/t.log | tee -a "$RES"
GARDEN_MODE_SET=1 ASSUME_YES=1 g install > /tmp/t.log
chk "second install finishes" grep -q "ALL DONE" /tmp/t.log
chk "second install skips working steps (>=8)" bash -c '[ "$(grep -c "Already installed and working: skipped" /tmp/t.log)" -ge 8 ]'

echo "== 4. Backup, verify, restore"
g backup > /tmp/t.log
chk "backup made" grep -q "Saved: junsgarden-backup_" /tmp/t.log
g verify latest > /tmp/t.log
chk "backup verifies" grep -q "every piece of" /tmp/t.log
chk "backup holds the wiki" grep -q "wiki.sql.gz" /tmp/t.log
printf '1\ny\n' | g restore > /tmp/t.log
chk "restore finishes" grep -q '"plant_count":485' /tmp/t.log
chk "wiki still there after restore" bash -c 'sleep 20; curl -fs -o /dev/null http://127.0.0.1:3000/'

echo "== 5. API keys"
printf '1\n1\nFAKEKEY-not-real-123\n0\n0\n' | g keys > /tmp/t.log
chk "key saved" bash -c '[ "$(grep -m1 "^GEMINI_API_KEY=" .env | cut -d= -f2)" = FAKEKEY-not-real-123 ]'
chk "fake key reported as wrong" grep -q "WRONG" /tmp/t.log
chk "website got the key" sudo docker exec garden-site sh -c 'test -n "$JUNSGARDEN_GEMINI_API_KEY"'
printf '1\n4\ny\n0\n0\n' | g keys > /tmp/t.log
chk "key removed" bash -c '[ -z "$(grep -m1 "^GEMINI_API_KEY=" .env | cut -d= -f2)" ]'

echo "== 6. Settings: move the photos folder and back"
printf '8\n2\n./my-photos\ny\n0\n' | g settings > /tmp/t.log
chk "IMAGES_DIR moved" bash -c '[ "$(grep -m1 "^IMAGES_DIR=" .env | cut -d= -f2)" = ./my-photos ] && [ -d my-photos/Database ]'
chk "photos still served after move" bash -c 'f=$(ls my-photos/Database | head -1); curl -fs -o /dev/null "http://127.0.0.1:8030/images/Database/$f"'
printf '8\n2\n./local/images\ny\n0\n' | g settings > /tmp/t.log
chk "IMAGES_DIR moved back" bash -c '[ "$(grep -m1 "^IMAGES_DIR=" .env | cut -d= -f2)" = ./local/images ]'

echo "== 7. Parts: n8n on and off"
printf '3\nA\n' | g parts > /tmp/t.log
chk "n8n starts" bash -c 'for i in $(seq 1 30); do curl -fs -o /dev/null http://127.0.0.1:5678/ && exit 0; sleep 3; done; exit 1'
printf '3\nA\n' | g parts > /tmp/t.log
chk "n8n stops" bash -c '! sudo docker ps --format "{{.Names}}" | grep -qx n8n'

echo "== 8. LiteLLM on and off"
printf 'y\n5\n' | g llm > /tmp/t.log
chk "LiteLLM starts with its models" bash -c 'sleep 25; K=$(grep ^LITELLM_MASTER_KEY= .env | cut -d= -f2); sudo docker exec -e K="$K" garden-llm python3 -c "import os,urllib.request,json;r=urllib.request.Request(\"http://127.0.0.1:4000/v1/models\",headers={\"Authorization\":\"Bearer \"+os.environ[\"K\"]});print([m[\"id\"] for m in json.load(urllib.request.urlopen(r))[\"data\"]])" | grep -q gemini-flash'
printf '7\ny\n' | g llm > /tmp/t.log
chk "LiteLLM stops" bash -c '! sudo docker ps --format "{{.Names}}" | grep -qx garden-llm'

echo "== 9. Front door: not on the internet yet (spare port 18480: Docker Desktop holds 80 on this PC)"
sed -i 's/^CADDY_HTTP_PORT=.*/CADDY_HTTP_PORT=18480/; s/^CADDY_HTTPS_PORT=.*/CADDY_HTTPS_PORT=18543/' .env
printf '1
' | g public > /tmp/t.log
chk "home Wi-Fi only: shows the garden" bash -c 'curl -fs http://localhost:18480/healthz | grep -q plant_count'
chk "home Wi-Fi only: garden tools hidden (403)" bash -c '[ "$(curl -s -o /dev/null -w "%{http_code}" http://localhost:18480/beds/plan)" = 403 ]'
chk "home Wi-Fi only: names the next step (No-IP)" grep -q "Sign up at https://www.noip.com" /tmp/t.log

echo "== 10. Front door: practice name with the sign-in page (spare ports)"
echo "$PW" | PUBLIC_NAME_IN=$H WIKI_NAME_IN='' ASSUME_YES=1 g public > /tmp/t.log
chk "practice front door set up" grep -q "Sign-in page: jun is in the group 'gardeners'" /tmp/t.log
chk "website through Caddy" bash -c "$C https://$H:18543/healthz | grep -q plant_count"
chk "garden tools need signing in" bash -c '[ "$('"$C"' -o /dev/null -w "%{http_code}" https://'"$H"':18543/beds/plan)" = 302 ]'
chk "fake identity header refused" bash -c '[ "$('"$C"' -o /dev/null -w "%{http_code}" -H "X-Auth-Email: you@example.com" https://'"$H"':18543/beds/plan)" = 302 ]'
rm -f /tmp/aj; $C -c /tmp/aj -b /tmp/aj -o /tmp/login.json -H 'Content-Type: application/json' -X POST "https://$H:18543/auth/api/firstfactor" --data "{\"username\":\"jun\",\"password\":\"$PW\"}"
chk "sign-in page accepts Jun" grep -q '"status":"OK"' /tmp/login.json
chk "garden tools open after signing in" bash -c '[ "$('"$C"' -b /tmp/aj -o /dev/null -w "%{http_code}" https://'"$H"':18543/beds/plan)" = 200 ]'
chk "short link redirects" bash -c '[ "$('"$C"' -o /dev/null -w "%{http_code}" https://'"$H"':18543/checkup)" = 308 ]'

echo "== 11. Users: guest, editor, remove"
printf '3\nGuesty\nguest@example.com\nVisitor-Guest-99\n0\n' | g users > /tmp/t.log
rm -f /tmp/gj; $C -c /tmp/gj -b /tmp/gj -o /tmp/g.json -H 'Content-Type: application/json' -X POST "https://$H:18543/auth/api/firstfactor" --data '{"username":"guesty","password":"Visitor-Guest-99"}'
chk "guest can sign in" grep -q '"status":"OK"' /tmp/g.json
chk "guest kept out of garden tools" bash -c '[ "$('"$C"' -b /tmp/gj -o /dev/null -w "%{http_code}" https://'"$H"':18543/beds/plan)" = 403 ]'
printf '2\nBob\nbob@example.com\nBobPassword-2026\n0\n' | g users > /tmp/t.log
chk "editor Bob added" bash -c 'grep -q "Bob=bob@example.com" .env'
rm -f /tmp/bj; curl -s -c /tmp/bj -o /dev/null --data "admin_login=1&admin_user=Bob&admin_password=BobPassword-2026" http://127.0.0.1:8030/login.php
chk "Bob can log in to the website" bash -c '[ "$(curl -s -b /tmp/bj -o /dev/null -w "%{http_code}" http://127.0.0.1:8030/streamlit/)" = 200 ]'
printf '4\nBob\ny\n0\n' | g users > /tmp/t.log
printf '4\nGuesty\ny\n0\n' | g users > /tmp/t.log
chk "Bob removed" bash -c '! grep -q "Bob=" .env'
chk "Bob's session ended" bash -c '[ "$(curl -s -b /tmp/bj -o /dev/null -w "%{http_code}" http://127.0.0.1:8030/streamlit/)" != 200 ]'

echo "== 12. Subdomains (fake DuckDNS name; no real DNS needed with spare ports)"
sed -i 's/^DUCKDNS_DOMAIN=.*/DUCKDNS_DOMAIN=testgarden/' .env
printf '2\nn8n\n5678\ny\n0\n' | g subdomains > /tmp/t.log
chk "subdomain settings valid and applied" grep -q "Done. Each new address" /tmp/t.log
chk "subdomain sends strangers to the sign-in page" bash -c '[ "$(curl -sk --resolve n8n.testgarden.duckdns.org:18543:127.0.0.1 -o /dev/null -w "%{http_code}" https://n8n.testgarden.duckdns.org:18543/)" = 302 ]'
printf '3\nn8n\n0\n' | g subdomains > /tmp/t.log
chk "subdomain removed" bash -c '! grep -q "n8n.testgarden" /etc/caddy/Caddyfile'
g addresses > /tmp/t.log
chk "My addresses lists the web name" grep -q "https://$H/beds/plan" /tmp/t.log

echo "== 13. NAS (a Samba share stands in for Jun's NAS)"
sudo docker rm -f testnas >/dev/null 2>&1; sudo mkdir -p /srv/nas1 /srv/nas2; sudo chown 1000:1000 /srv/nas1 /srv/nas2
sudo docker run -d --name testnas -p 445:445 -v /srv/nas1:/shares/backups -v /srv/nas2:/shares/newnas \
  -e ACCOUNT_nasuser=NasPass-2026 -e UID_nasuser=1000 \
  -e "SAMBA_VOLUME_CONFIG_backups=[backups]; path=/shares/backups; writable = yes; read only = no; force user = nasuser; valid users = nasuser" \
  -e "SAMBA_VOLUME_CONFIG_newnas=[newnas]; path=/shares/newnas; writable = yes; read only = no; force user = nasuser; valid users = nasuser" \
  ghcr.io/servercontainers/samba:latest >/dev/null; sleep 10
printf 'n\n127.0.0.1\nnasuser\nNasPass-2026\nbackups\njunsgarden\ny\ny\n' | g nas > /tmp/t.log
chk "NAS connected" bash -c 'mount | grep -q " /mnt/garden-nas "'
chk "backup copied to NAS" bash -c 'ls /srv/nas1/junsgarden/junsgarden-backup_*.tar >/dev/null 2>&1'
chk "whole garden folder copied to NAS" bash -c 'ls /srv/nas1/junsgarden/whole-garden-folder/junsgarden-whole-folder_*.tar >/dev/null 2>&1'
printf '9\ny\nn\n127.0.0.1\nnasuser\nNasPass-2026\nnewnas\njunsgarden\ny\nn\ny\n0\n' | g backups > /tmp/t.log
chk "changed to the new NAS" bash -c 'grep -q "//127.0.0.1/newnas" /etc/fstab'
chk "backups copied across to the new NAS" bash -c '[ "$(ls /srv/nas2/junsgarden/junsgarden-backup_*.tar 2>/dev/null | wc -l)" -ge 1 ]'

echo "== 14. Help file, file check, language, front door off"
g helpme > /tmp/t.log
chk "help file made" bash -c 'ls help/junsgarden-help-*.txt >/dev/null'
chk "help file has no database password" bash -c '! grep -qF "$(grep ^DB_PASSWORD= .env | cut -d= -f2)" help/junsgarden-help-*.txt'
printf 'y\nn\n' | g files > /tmp/t.log
chk "file fingerprint check runs" grep -qE "exactly as it was|different from the zip" /tmp/t.log
echo ko > .garden-lang; printf '0\n' | g > /tmp/t.log; echo en > .garden-lang
chk "Korean menu shows" grep -q "설치" /tmp/t.log
printf 'y\n' | g offline > /tmp/t.log
chk "front door off" bash -c '! systemctl is-active --quiet caddy && [ -z "$(grep -m1 "^PUBLIC_NAME=" .env | cut -d= -f2)" ]'

echo "== 15. Uninstall level 1 (programs only)"
printf '1\ny\n' | g uninstall > /tmp/t.log
chk "garden containers removed" bash -c '! sudo docker ps -a --format "{{.Names}}" | grep -qE "^garden-(site|db)$"'
chk "garden data kept" bash -c '[ -d local/images/Database ] && [ -f .env ] && ls backups/*.tar >/dev/null'

echo "== 16. Reinstall, then uninstall level 2 (remove everything after a final backup)"
GARDEN_MODE_SET=1 ASSUME_YES=1 g install < /dev/null > /tmp/t.log
chk "reinstall after level 1 finishes" grep -q "ALL DONE" /tmp/t.log
mkdir -p ~/final-keep
printf '2
%s
DELETE
' "$HOME/final-keep" | g uninstall > /tmp/t.log
chk "final backup kept outside the folder" bash -c 'ls ~/final-keep/junsgarden-backup_*final.tar >/dev/null 2>&1'
chk "garden data removed" bash -c '[ ! -d local ] && [ ! -f .env ]'
chk "no garden containers left" bash -c '! sudo docker ps -a --format "{{.Names}}" | grep -qE "^(garden-|junsgarden-)"'
chk "Caddy removed" bash -c '! command -v caddy >/dev/null'
chk "nightly backup line removed" bash -c '! crontab -l 2>/dev/null | grep -q garden.sh'
echo
echo "RESULT: $P passed, $F failed" | tee -a "$RES"
