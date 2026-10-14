#!/bin/bash
# =============================================================================
#  02-install-docker.sh: install Docker the way Docker's own Ubuntu guide says.
#  (https://docs.docker.com/engine/install/ubuntu/)
#
#  Run:   bash lab/02-install-docker.sh
#  Safe to run again.
#
#  What it does:
#    1. removes old, conflicting Docker packages (if any)
#    2. adds Docker's official software source
#    3. installs Docker Engine and Docker Compose
#    4. lets your user run Docker without typing sudo
#    5. tests it with the "hello-world" container
#
#  After it finishes you must LOG OUT AND LOG BACK IN once (step 4 needs it).
# =============================================================================
set -eu

say() { printf '\n== %s\n' "$1"; }

if [ "$(id -u)" -eq 0 ]; then
    echo "Do not run this as root. Run it as your normal user; it uses sudo."
    exit 1
fi

say "1. Removing old Docker packages (none installed is fine)"
for pkg in docker.io docker-compose docker-compose-v2 docker-doc podman-docker containerd runc; do
    sudo apt-get remove -y "$pkg" >/dev/null 2>&1 || true
done

say "2. Adding Docker's software source"
sudo apt-get update
sudo apt-get install -y ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc
# shellcheck disable=SC1091
CODENAME=$(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}")
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $CODENAME stable" |
    sudo tee /etc/apt/sources.list.d/docker.list >/dev/null
sudo apt-get update

say "3. Installing Docker and Compose"
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
sudo systemctl enable --now docker

say "4. Letting $USER use Docker without sudo"
sudo usermod -aG docker "$USER"

say "5. Test (with sudo, because the group change needs a new login)"
sudo docker run --rm hello-world | head -5

say "DONE"
echo "Now LOG OUT and LOG BACK IN (or restart). Then check with:  docker run --rm hello-world"
echo "Next: bash lab/03-start-garden.sh"
