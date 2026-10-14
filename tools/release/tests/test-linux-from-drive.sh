#!/bin/bash
# shellcheck disable=SC2016,SC2012  # each check runs in its own bash -c on purpose; ls only shows a folder
# Test-only: the Ubuntu path exactly as a new owner does it, starting from the
# Google Drive links: download the three zips into Home, check each fingerprint,
# unzip the main one, let its start file get the program from GitHub (every file
# checked against FILES-FROM-AUTHOR.txt), install (Easy). The add-ons stay zipped
# in Home, where the menu looks for them.
#
#   bash test-linux-from-drive.sh LINKS-FILE
#
# LINKS-FILE is the test-links.txt that drive-cycle.ps1 writes: one line per
# download, "name drive-file-id sha256". Run it on a throwaway Ubuntu (it installs).
set -u
LINKS=${1:?give the test-links.txt that drive-cycle.ps1 wrote}
EMAIL=${GARDEN_EMAIL:-you@example.com}
cd ~ || exit 1
sudo apt-get update -qq && sudo apt-get install -y -qq unzip curl >/dev/null
while read -r f id want; do
    real=${f/-TEST-small/}                       # the names a real owner's files have
    curl -sSL "https://drive.usercontent.google.com/download?id=$id&export=download&confirm=t" -o "$real"
    if [ "$(sha256sum "$real" | cut -c1-64)" = "$want" ]; then echo "DOWNLOAD OK  $real"; else echo "DOWNLOAD DIFFERENT  $real"; exit 1; fi
done < <(tr -d '\r' < "$LINKS")
unzip -q junsgarden-for-jun.zip -d ~ && cd ~/junsgarden-for-jun || exit 1
ls -l Start-Here-Ubuntu.sh | awk '{print $1, $NF}'   # must be runnable for "Run as a Program"
echo "files in the zip: $(find . -type f | wc -l); garden.sh there before the first start: $([ -f garden.sh ] && echo yes || echo no)"
# The first start, as Start-Here-Ubuntu.sh does it: the program from GitHub, every file checked.
python3 tools/get-code.py || { echo "GET-CODE FAILED"; exit 1; }
ls -l garden.sh Start-Here-Ubuntu.sh | awk '{print $1, $NF}'
grep -v '  ./DOWNLOADS-SHA256.txt$' FILES-FROM-AUTHOR.txt | sha256sum -c --quiet && echo "EVERY FILE MATCHES FILES-FROM-AUTHOR.txt"
GARDEN_IN_TERMINAL=1 GARDEN_MODE_SET=1 ASSUME_YES=1 GARDEN_NAME=Jun GARDEN_EMAIL="$EMAIL" bash Start-Here-Ubuntu.sh install > /tmp/linux-install.log 2>&1
echo "install exit=$?"
sed 's/\x1b\[[0-9;]*m//g' /tmp/linux-install.log | grep -E "STOPPED|stopped at|ALL DONE|WARNING|Photos ready|No photos|EASY mode chose" | tail -12
curl -fs http://127.0.0.1:8030/healthz; echo
grep ^COMPOSE_PROFILES= .env
