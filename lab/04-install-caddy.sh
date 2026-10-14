#!/bin/bash
# =============================================================================
#  04-install-caddy.sh: install Caddy, the front door that gives your site HTTPS.
#
#  Run:   bash lab/04-install-caddy.sh YOUR-NAME.ddns.net
#         bash lab/04-install-caddy.sh YOUR-NAME.ddns.net --with-apache
#
#  YOUR-NAME is the web name you will use. Until guide 04b says so, use your
#  TEST name, not junsgarden.zapto.org.
#
#  What it does:
#    1. installs Caddy from its official Ubuntu software source
#    2. writes /etc/caddy/Caddyfile for your name (a copy of the old one is
#       kept as /etc/caddy/Caddyfile.bak)
#    3. checks the file, then reloads Caddy
#    4. opens ports 80 and 443 in the laptop's firewall (ufw)
#
#  It does NOT touch your router. You still must forward ports 80 and 443 to
#  this laptop (guide 04, Part D). Do that last.
#  --with-apache uses the Caddyfile that also sends /learn/ to Apache.
#  Caddy docs: https://caddyserver.com/docs/install#debian-ubuntu-raspbian
# =============================================================================
set -eu
cd "$(dirname "$0")/.."

say() { printf '\n== %s\n' "$1"; }

NAME="${1:-}"
if [ -z "$NAME" ]; then
    echo "Usage: bash lab/04-install-caddy.sh YOUR-NAME.ddns.net [--with-apache]"
    exit 1
fi
case "$NAME" in
    *[!A-Za-z0-9.-]*|.*|*..*) echo "That does not look like a web name: $NAME"; exit 1;;
esac
SRC=docker/extras/Caddyfile.example
[ "${2:-}" = "--with-apache" ] && SRC=docker/extras/Caddyfile.with-apache.example

if [ "$NAME" = "junsgarden.zapto.org" ]; then
    echo "WARNING: this is the real site's name. Only continue if you have done"
    echo "guide 04b and the author has told you the switch time is now."
    printf 'Type YES to continue: '
    read -r ANSWER
    [ "$ANSWER" = "YES" ] || { echo "Stopped."; exit 1; }
fi

say "1. Installing Caddy"
if command -v caddy >/dev/null 2>&1; then
    echo "Caddy is already installed."
else
    sudo apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' |
        sudo gpg --dearmor --yes -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' |
        sudo tee /etc/apt/sources.list.d/caddy-stable.list >/dev/null
    sudo apt-get update
    sudo apt-get install -y caddy
fi

say "2. Writing the Caddyfile for $NAME"
[ -f /etc/caddy/Caddyfile ] && sudo cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak
sed "s/YOUR-NAME\.ddns\.net/$NAME/g" "$SRC" | sudo tee /etc/caddy/Caddyfile >/dev/null
sudo mkdir -p /var/log/caddy
sudo chown caddy:caddy /var/log/caddy

say "3. Checking and reloading"
sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl enable caddy
sudo systemctl reload caddy || sudo systemctl restart caddy

say "4. Firewall: ports 80 and 443"
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw status

say "DONE"
echo "Caddy now answers for https://$NAME once:"
echo "  - the name points at your home address (lab/05-install-noip-duc.sh), and"
echo "  - your router forwards ports 80 and 443 to this laptop."
