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

# A throwaway "Jun's laptop": Ubuntu 24.04 with systemd as the first process (so services start
# like on a real machine; systemd must run as root, which is why this is not a Dockerfile in the
# repo for the scanners to flag), user jun with sudo. Nothing is published on the network.
docker build -q -t "$img" - >/dev/null <<'EOF'
FROM ubuntu:24.04
ENV container=docker DEBIAN_FRONTEND=noninteractive
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends systemd systemd-sysv sudo dbus curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && (userdel -r ubuntu 2>/dev/null || true) \
    && useradd -m -s /bin/bash -c Jun jun && echo 'jun ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/jun \
    && systemctl mask systemd-logind getty.target console-getty.service
STOPSIGNAL SIGRTMIN+3
CMD ["/sbin/init"]
EOF
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

# The web address changes in ONE step, the way Jun would do it on the website: the labels page's
# form files the request, the address watcher (systemd, set up by the install) carries it out.
newname=garden-new.example.test
inside "systemctl is-enabled junsgarden-address.path" 2>/dev/null | grep -q enabled \
    && ok "address watcher installed" || bad "address watcher installed"
c=$(inside "curl -s -b /tmp/j -o /dev/null -w '%{http_code}' -X POST --data-urlencode name=$newname --data-urlencode folder=$folder http://localhost:8030/labels/address")
[ "$c" = 303 ] && ok "labels page accepts the new address ($c)" || bad "labels page accepts the new address ($c)"
st=/home/jun/junsgarden/local/data/address-status.json
for _ in $(seq 120); do inside "grep -q '\"new\":\"$newname' $st" 2>/dev/null && break; sleep 5; done
inside "grep -q '\"state\":\"done\"' $st" 2>/dev/null && ok "the garden computer made the change" || bad "the garden computer made the change"
inside "curl -s http://localhost:8030/app/config.json" | grep -q "\"base\":\"https://$newname$folder\"" \
    && ok "app config (signed) names the new address" || bad "app config names the new address"
lp=$(inside "curl -s -b /tmp/j http://localhost:8030/labels")
printf '%s' "$lp" | grep -q "<b>https://$newname$folder</b>" && ok "labels page shows the new address" || bad "labels page shows the new address"
printf '%s' "$lp" | grep -q "Changed on" && ok "labels page says the codes use it" || bad "labels page says the codes use it"
inside "grep -q $newname /home/jun/junsgarden/site/privacy.html /home/jun/junsgarden/site/apps/version.json" \
    && ok "site files say the new name" || bad "site files say the new name"
[ "$(code /q/1)" = 302 ] || [ "$(code /q/1)" = 303 ] && ok "an old printed code (/q/1) still lands" || bad "an old printed code (/q/1) still lands"
inside "grep -q '^OLD_NAMES=.*junsgarden.zapto.org' /home/jun/junsgarden/.env" \
    && ok "the old name is kept forwarding" || bad "the old name is kept forwarding"

echo "$((results-failed))/$results passed"
# KEEP=1 keeps the machine to look inside (docker exec -it gp-wiz-<lang> bash); remove it after.
[ "${KEEP:-0}" = 1 ] || { docker rm -f "$name" >/dev/null 2>&1; docker volume rm "$name-docker" "$name-containerd" >/dev/null 2>&1; }
[ $failed = 0 ]
