#!/bin/bash
# =============================================================================
#  01-prepare-ubuntu.sh: get a fresh Ubuntu laptop ready to be a home server.
#
#  Run:   bash lab/01-prepare-ubuntu.sh
#  Safe to run again: it skips anything that is already done.
#
#  What it does, in order:
#    1. updates Ubuntu and installs a few basic tools
#    2. turns on the firewall, allowing only SSH (remote login) for now
#    3. adds 4 GB of swap (extra memory on the disk) if there is none, so the
#       website's first build does not run out of memory
#    4. keeps the laptop awake with the lid closed
#    5. turns on automatic security updates
#
#  It asks for your password (sudo). It does not open any internet ports.
#  To undo the lid and sleep settings:
#      sudo rm /etc/systemd/logind.conf.d/lab.conf
#      sudo systemctl unmask sleep.target suspend.target hibernate.target hybrid-sleep.target
# =============================================================================
set -eu

say() { printf '\n== %s\n' "$1"; }

if [ "$(id -u)" -eq 0 ]; then
    echo "Do not run this as root. Run it as your normal user; it uses sudo."
    exit 1
fi

say "1. Updating Ubuntu (this can take a while the first time)"
sudo apt-get update
sudo DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    git curl wget unzip htop openssh-server ufw argon2 openssl ca-certificates

say "2. Firewall: allow SSH only"
sudo ufw allow OpenSSH
# "--force" skips the yes/no question. SSH is already allowed above, so this
# cannot lock you out of a remote login.
sudo ufw --force enable
sudo ufw status

say "3. Swap (extra memory on disk)"
if swapon --show | grep -q .; then
    echo "Swap already exists, leaving it:"
    swapon --show
else
    sudo fallocate -l 4G /swapfile2
    sudo chmod 600 /swapfile2
    sudo mkswap /swapfile2
    sudo swapon /swapfile2
    echo '/swapfile2 none swap sw 0 0' | sudo tee -a /etc/fstab >/dev/null
    echo "Added 4 GB of swap."
fi

say "4. Keep the laptop awake with the lid closed"
sudo mkdir -p /etc/systemd/logind.conf.d
printf '[Login]\nHandleLidSwitch=ignore\nHandleLidSwitchExternalPower=ignore\nHandleLidSwitchDocked=ignore\n' |
    sudo tee /etc/systemd/logind.conf.d/lab.conf >/dev/null
sudo systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target
# Changes to logind take effect after a reboot. Restarting the service now can
# end your desktop session, so this script does not do that.

say "5. Automatic security updates"
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y unattended-upgrades
printf 'APT::Periodic::Update-Package-Lists "1";\nAPT::Periodic::Unattended-Upgrade "1";\n' |
    sudo tee /etc/apt/apt.conf.d/20auto-upgrades >/dev/null

say "DONE"
echo "Your laptop's address on the home network:"
hostname -I
echo
echo "Next: restart the laptop (sudo reboot), then run  bash lab/02-install-docker.sh"
