#!/bin/sh
# The installer test: Jun's install from scratch on a throwaway Ubuntu 24.04, then the garden
# checked from the inside. Run ON THE LAB MACHINE from ~/garden-rust-pages (after sync.sh):
#   sh wizard_test.sh en|ko|both [folder]     e.g.  sh wizard_test.sh both /garden
# It installs exactly as the wizard does (garden.sh install, no questions; the wizard's secrets
# file carries a random password), in the chosen language, from this branch's files. Prints one
# PASS/FAIL line per check; the full install log is wiz-<lang>.log. The machine is deleted after.
set -u
cd "$(dirname "$0")" || exit 1
lang=${1:-en}
folder=${2:-}
name="gp-wiz-$lang"
img=gardenlab/wizard:24.04
results=0; failed=0
ok()  { results=$((results+1)); echo "PASS $1"; }
bad() { results=$((results+1)); failed=$((failed+1)); echo "FAIL $1"; }
inside() { docker exec "$name" sh -c "$1"; }
code() { inside "curl -s -o /dev/null -w '%{http_code}' ${2:-} 'http://localhost:8030$1'"; }

docker build -q -t "$img" src-cand/docs/rust-pages/wizard >/dev/null
docker rm -f "$name" >/dev/null 2>&1
docker volume rm "$name-docker" "$name-containerd" >/dev/null 2>&1
docker run -d --name "$name" --hostname jun-laptop --privileged --cgroupns=host --cpus=6 --memory=8g \
    --tmpfs /run --tmpfs /run/lock -v "$name-docker:/var/lib/docker" -v "$name-containerd:/var/lib/containerd" \
    "$img" >/dev/null
for _ in $(seq 30); do inside "systemctl is-system-running" 2>/dev/null | grep -qE 'running|degraded' && break; sleep 1; done

# The garden folder, as the download unpacks it; a random password in the wizard's secrets file.
tar -C src-cand -cf - . | docker exec -i "$name" sh -c 'mkdir -p /home/jun/junsgarden && tar -xf - -C /home/jun/junsgarden && chown -R jun:jun /home/jun/junsgarden'
pw="$(head -c 12 /dev/urandom | od -An -tx1 | tr -d ' \n')-garden"
printf 'GARDEN_PASSWORD=%s\n' "$pw" | docker exec -i -u jun "$name" sh -c 'umask 077; cat > /home/jun/junsgarden/.garden-wizard-secrets'

start=$(date +%s)
# The same settings Garden-Menu.ps1 sends for a wizard install (custom, no extra parts), and
# nothing typed: every answer must come from the wizard. (No ASSUME_YES: the menu never sets it.)
docker exec -u jun -w /home/jun/junsgarden -e GARDEN_REMOTE_INSTALL=1 -e GARDEN_WIZARD=1 -e GARDEN_MODE_IN=custom \
    -e GARDEN_PARTS= -e GARDEN_NAME=Jun -e GARDEN_EMAIL=jun@example.test -e GARDEN_LANG_IN="$lang" -e GARDEN_FOLDER="$folder" \
    -e GARDEN_SECRETS_FILE=.garden-wizard-secrets "$name" bash garden.sh install < /dev/null > "wiz-$lang.log" 2>&1
rc=$?
echo "install took $(( ($(date +%s) - start) / 60 )) min (exit $rc)"
[ $rc = 0 ] && ok "install finished" || bad "install finished (exit $rc, see wiz-$lang.log)"

# The language: Korean lines in ko/both, none in en.
ko=$(grep -cP '[\x{AC00}-\x{D7A3}]' "wiz-$lang.log" 2>/dev/null); ko=${ko:-0}   # grep -c prints 0 (and exits 1) when none
case "$lang" in
    en) [ "$ko" = 0 ] && ok "install spoke English only" || bad "install spoke English only ($ko Korean lines)" ;;
    *)  [ "$ko" -gt 20 ] && ok "install spoke Korean ($ko lines)" || bad "install spoke Korean ($ko lines)" ;;
esac
inside "su jun -c 'crontab -l' | grep -q 'garden.sh backup auto'" && ok "hourly backups scheduled" || bad "hourly backups scheduled"
inside "test ! -e /home/jun/junsgarden/.garden-wizard-secrets" && ok "the wizard's secrets file is gone" || bad "the wizard's secrets file is gone"

# The garden answers, page by page, as a visitor.
for p in /healthz / /index.php /catalog.php /gallery.php /blog.php /data.php /login.php /plant_listing.php \
         "/catalog.php?ajax_load=1&page=1" "/app_api.php?action=list" "/app_api.php?action=bed_counts" \
         /app/setup /app/config.json /qr/app-setup.png /qr/app-apk.png /q/1 /favicon.svg; do
    c=$(code "$p")
    case "$c" in 200|302|303) ok "visitor $p ($c)" ;; *) bad "visitor $p ($c)" ;; esac
done
inside "curl -s http://localhost:8030/app/config.json" | grep -q '"sig":"[0-9a-f]\{128\}"' && ok "app config is signed" || bad "app config is signed"
[ "$(code /help)" = 303 ] && ok "help needs a login" || bad "help needs a login"

# Jun logs in with the password the wizard set: website and app.
inside "curl -s -c /tmp/j -o /dev/null -X POST --data-urlencode admin_user=Jun --data-urlencode 'admin_password=$pw' -d admin_login= -d back=/ http://localhost:8030/login.php"
for p in /help /labels "/help?guide=00-READ-ME-FIRST" "/streamlit/?tab=1"; do
    c=$(inside "curl -s -b /tmp/j -o /dev/null -w '%{http_code}' 'http://localhost:8030$p'")
    [ "$c" = 200 ] && ok "Jun sees $p" || bad "Jun sees $p ($c)"
done
tok=$(inside "curl -s -X POST --data-urlencode username=Jun --data-urlencode 'password=$pw' 'http://localhost:8030/app_api.php?action=login'" | sed -n 's/.*"token":"\([0-9a-f]*\)".*/\1/p')
[ ${#tok} = 64 ] && ok "the app logs in" || bad "the app logs in"
inside "curl -s -H 'Authorization: Bearer $tok' 'http://localhost:8030/app_api.php?action=whoami'" | grep -q '"username":"Jun"' \
    && ok "the app is Jun" || bad "the app is Jun"

# The new bash jobs run (no browser here, so they only print).
for job in labels app; do
    docker exec -u jun -w /home/jun/junsgarden -e ASSUME_YES=1 "$name" bash garden.sh "$job" >/dev/null 2>&1 \
        && ok "garden.sh $job" || bad "garden.sh $job"
done
if [ -n "$folder" ]; then
    inside "grep -q '^PUBLIC_PATH=$folder\$' /home/jun/junsgarden/.env" && ok "folder $folder saved" || bad "folder $folder saved"
fi

echo "$((results-failed))/$results passed"
docker rm -f "$name" >/dev/null 2>&1; docker volume rm "$name-docker" "$name-containerd" >/dev/null 2>&1
[ $failed = 0 ]
