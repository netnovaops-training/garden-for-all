#!/bin/bash
# =============================================================================
#  05-install-noip-duc.sh: install the No-IP update client (DUC) so your web
#  name keeps pointing at your home internet address.
#
#  Run:   bash lab/05-install-noip-duc.sh
#
#  You need the DDNS Key USERNAME and PASSWORD from your own No-IP account. They are
#  typed in here with the screen hidden. They are saved only in
#  /etc/default/noip-duc, which only the administrator can read.
#  (This is the DDNS Key, NOT your No-IP account password. Guide 04, Part A.)
#
#  What it does:
#    1. downloads the latest No-IP DUC for Linux from noip.com
#    2. installs the .deb that matches your laptop
#    3. saves your key in /etc/default/noip-duc (mode 600)
#    4. starts the service and shows its status
#
#  Official steps: https://www.noip.com/support/knowledgebase/install-linux-3-x-dynamic-update-client-duc
#  If a step fails, those pages are the ones to trust.
# =============================================================================
set -eu

say() { printf '\n== %s\n' "$1"; }

if [ "$(id -u)" -eq 0 ]; then
    echo "Do not run this as root. Run it as your normal user; it uses sudo."
    exit 1
fi

say "1. Downloading the No-IP client"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"
wget --content-disposition -q https://www.noip.com/download/linux/latest
tar xf noip-duc_*.tar.gz

say "2. Installing"
ARCH=$(dpkg --print-architecture)
DEB=$(find . -name "noip-duc_*_${ARCH}.deb" | head -1)
if [ -z "$DEB" ]; then
    echo "No package for '$ARCH' was found in the download."
    echo "Ask Gemini (prompt 3) and show it:  ls -R $WORK"
    exit 1
fi
sudo apt-get install -y "$DEB"

say "3. Your DDNS Key"
printf 'DDNS Key username: '
read -r KEYUSER
printf 'DDNS Key password (hidden): '
stty -echo
read -r KEYPASS
stty echo
printf '\n'
if [ -z "$KEYUSER" ] || [ -z "$KEYPASS" ]; then
    echo "Both are needed."
    exit 1
fi
# umask makes the file private from the very first byte written.
( umask 077
  printf 'NOIP_USERNAME=%s\nNOIP_PASSWORD=%s\nNOIP_HOSTNAMES=all.ddnskey.com\n' "$KEYUSER" "$KEYPASS" |
      sudo tee /etc/default/noip-duc >/dev/null )
sudo chmod 600 /etc/default/noip-duc
unset KEYUSER KEYPASS

say "4. Starting the service"
if systemctl list-unit-files | grep -q '^noip-duc.service'; then
    sudo systemctl enable --now noip-duc
    sudo systemctl status noip-duc --no-pager | head -8
else
    echo "The package did not add a noip-duc service."
    echo "Follow the 'Running the Linux DUC v3.0 at startup' page on noip.com,"
    echo "or ask Gemini (prompt 3). Your key is already saved in /etc/default/noip-duc."
    exit 1
fi

say "DONE"
echo "Check that your name points at your home address (use YOUR name):"
echo "  getent hosts YOUR-NAME.ddns.net ; curl -s https://api.ipify.org ; echo"
