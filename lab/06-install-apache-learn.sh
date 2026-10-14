#!/bin/bash
# =============================================================================
#  06-install-apache-learn.sh: install Apache as a PRIVATE learning web server
#  (it listens on 127.0.0.1:8081 only, so only this laptop can reach it).
#
#  Run:   bash lab/06-install-apache-learn.sh
#
#  What it does:
#    1. installs Apache
#    2. moves Apache off port 80 (Caddy uses 80 and 443)
#    3. makes a hello page in /var/www/learn
#    4. checks the settings and starts Apache
#  Then it tests the page with curl.
#
#  Undo:  sudo systemctl disable --now apache2 && sudo apt remove -y apache2
#  Full steps and explanations: docker/extras/apache-learn.conf.example
# =============================================================================
set -eu
cd "$(dirname "$0")/.."

say() { printf '\n== %s\n' "$1"; }

say "1. Installing Apache"
sudo apt-get install -y apache2
sudo systemctl disable --now apache2 || true

say "2. Moving Apache to 127.0.0.1:8081"
sudo cp /etc/apache2/ports.conf /etc/apache2/ports.conf.bak
sudo sed -i 's/^Listen 80$/Listen 127.0.0.1:8081/' /etc/apache2/ports.conf
grep -n '^Listen' /etc/apache2/ports.conf

say "3. The learning site"
sudo mkdir -p /var/www/learn
echo '<h1>Hello from Apache</h1><p>This is my own web server.</p>' |
    sudo tee /var/www/learn/index.html >/dev/null
sudo cp docker/extras/apache-learn.conf.example /etc/apache2/sites-available/learn.conf
sudo a2dissite 000-default >/dev/null 2>&1 || true
sudo a2ensite learn

say "4. Checking and starting"
sudo apache2ctl configtest
sudo systemctl enable --now apache2
sleep 2
curl -fsS http://127.0.0.1:8081/ && echo
if ss -ltn | grep -q '0.0.0.0:8081\|\*:8081'; then
    echo "WARNING: Apache is listening on every address. It should only be 127.0.0.1."
fi

say "DONE"
echo "Apache is at http://127.0.0.1:8081/ (this laptop only)."
echo "To also reach it as https://YOUR-NAME/learn/ use:"
echo "  bash lab/04-install-caddy.sh YOUR-NAME.ddns.net --with-apache"
