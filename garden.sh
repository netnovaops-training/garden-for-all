#!/bin/bash
# =============================================================================
#  garden.sh: the one program for Jun's Garden on Ubuntu.
#
#  Run it:   bash garden.sh            (shows a menu with numbers)
#  Or jump straight to a job:
#            bash garden.sh install    install everything on THIS computer
#            bash garden.sh public     the front door: HTTPS, sign-in page, web name
#            bash garden.sh remote     install on ANOTHER computer over SSH
#            bash garden.sh start | stop | open | logs | rebuild
#            bash garden.sh backup     save everything into backups/
#            bash garden.sh restore    put a backup back (it asks which)
#            bash garden.sh check      the checklist: programs, ports, services
#            bash garden.sh password   set your password (website + sign-in page)
#            bash garden.sh email      set up sending email (Gmail)
#            bash garden.sh network    this computer's address + router steps
#            bash garden.sh parts      turn optional parts on or off
#            bash garden.sh ai         connect a local AI computer (optional)
#            bash garden.sh dbweb      look inside the database in the browser
#            bash garden.sh db         a database command line (SQL)
#            bash garden.sh icon       put "Jun's Garden" on the desktop
#            bash garden.sh webname    change junsgarden.zapto.org to another name in the files
#            bash garden.sh photos     load the photos download (now, or again)
#            bash garden.sh rename     rename or move a file or folder (every place that uses it follows)
#            bash garden.sh off | on   switch everything off (nothing is deleted) or on again
#            bash garden.sh checkup    start again any program that is stuck
#            bash garden.sh cleanup    remove Docker's leftovers (never your files)
#            bash garden.sh test       run the Rust tests (slow the first time)
#            bash garden.sh uninstall  remove what this program installed
#
#  OPEN BOOK: before each step it says WHAT it installs, WHY, and where the
#  official instructions are, and it shows each command before running it.
#  The install and the front door also save everything they print in the logs/ folder.
#  Nothing here prints a password. Nothing here gives anyone else access.
#  The full list of what can be installed: WHAT-GETS-INSTALLED.md
# =============================================================================

# If someone typed "sh garden.sh", start again with bash (this file needs bash).
if [ -z "${BASH_VERSION:-}" ]; then exec bash "$0" "$@"; fi
set -uo pipefail

HERE=$(cd "$(dirname "$(readlink -f "$0")")" && pwd)
cd "$HERE" || exit 1

# Running inside Windows' built-in Ubuntu (WSL)? Then ignore Windows' own
# programs (like Docker Desktop's docker.exe), so the garden uses its own
# Docker inside Ubuntu, exactly like on a real Ubuntu computer.
if grep -qi microsoft /proc/version 2>/dev/null; then
    PATH=$(printf '%s' "$PATH" | tr ':' '\n' | grep -v '^/mnt/' | paste -sd: -)
    export PATH
fi

# Started by a click (no terminal window)? Open a terminal window and run
# there, so you can see everything and answer the questions.
if [ ! -t 0 ] && [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] && [ -z "${GARDEN_IN_TERMINAL:-}" ]; then
    export GARDEN_IN_TERMINAL=1
    for t in gnome-terminal x-terminal-emulator xfce4-terminal konsole xterm; do
        command -v "$t" >/dev/null 2>&1 || continue
        case "$t" in
            gnome-terminal) exec gnome-terminal --title="Jun's Garden" -- bash "$HERE/garden.sh" "$@" ;;
            *)              exec "$t" -e bash "$HERE/garden.sh" "$@" ;;
        esac
    done
fi

REMOTE_DIR=junsgarden-for-jun          # the folder name used on another computer
SETTINGS=.garden-remote                # remembers the last computer you set up

# ---------- language: English, Korean, or both -----------------------------------
# Only the WORDS change. What you type (numbers, letters, y/n) stays the same.
# The Korean words live in lang/ko.tsv: one line per sentence,
# "English<TAB>Korean". A sentence that is not in that file shows in English.
# Your choice is saved in .garden-lang (en, ko or both). Menu: L.
LANG_MODE=$(cat .garden-lang 2>/dev/null || echo en)
case "$LANG_MODE" in en|ko|both) ;; *) LANG_MODE=en ;; esac
declare -A KO=()
# Reads lang/ko.tsv into memory when Korean (or both) is chosen.
load_lang() {
    KO=()
    [ "$LANG_MODE" = en ] && return 0
    [ -f lang/ko.tsv ] || return 0
    local en ko
    while IFS=$'\t' read -r en ko; do
        [ -n "$en" ] && [ "${en:0:1}" != "#" ] && KO["$en"]=$ko
    done < lang/ko.tsv
}
load_lang

# Lt "text": the text in the chosen language, as one line (for questions).
Lt() {
    local k=; [ -n "$1" ] && k=${KO[$1]:-}
    case "$LANG_MODE" in
        ko)   printf '%s' "${k:-$1}" ;;
        both) if [ -n "$k" ]; then printf '%s / %s' "$1" "$k"; else printf '%s' "$1"; fi ;;
        *)    printf '%s' "$1" ;;
    esac
}
# L "text": print a line in the chosen language. In "both" mode the Korean
# line comes right under the English one, a little dimmer. Keeps indentation.
L() {
    local s=$1 lead core k
    lead=${s%%[! ]*}; core=${s#"$lead"}; k=; [ -n "$core" ] && k=${KO[$core]:-}
    case "$LANG_MODE" in
        ko)   printf '%s%s\n' "$lead" "${k:-$core}" ;;
        both) printf '%s\n' "$s"; [ -n "$k" ] && printf '%s\033[2m%s\033[0m\n' "$lead" "$k" ;;
        *)    printf '%s\n' "$s" ;;
    esac
}
# Lblock: print every line of a block of text (a "here document") with L.
Lblock() { local line; while IFS= read -r line; do L "$line"; done; }

# Menu L: pick English, Korean or both. Saved in .garden-lang.
choose_language() {
    printf '\n   1) English\n   2) 한국어 (Korean)\n   3) Both / 둘 다\n'
    local c; read -r -p "   1, 2 or 3: " c
    case "$c" in 1) LANG_MODE=en ;; 2) LANG_MODE=ko ;; 3) LANG_MODE=both ;; *) return 0 ;; esac
    echo "$LANG_MODE" > .garden-lang
    load_lang
}

# ---------- small helpers ------------------------------------------------------
# has PATTERN: like grep -q, but reads ALL its input first. With pipefail (set above),
# grep -q stops at the first match and the program feeding it can fail on the
# cut-off pipe, which turns a true "yes" into "no" when the output is long.
has() { grep "$@" >/dev/null; }
say()  { printf '\n\033[1;32m== %s\033[0m\n' "$(Lt "$*")"; }
note() { L "   $*"; }
# A yellow WARNING line: something to fix soon, not a stop.
warn() { printf '\033[1;33m%s:\033[0m %s\n' "$(Lt WARNING)" "$(Lt "$*")"; }
# A red STOPPED line, and 'return 1' so the step counts as failed.
fail() { printf '\033[1;31m%s:\033[0m %s\n' "$(Lt STOPPED)" "$(Lt "$*")"; return 1; }
# Ask a question and print the answer (Enter = the suggested answer in [brackets]).
ask()  { local q=$1 def=${2:-} a; read -r -p "$(Lt "$q")${def:+ [$def]}: " a; printf '%s' "${a:-$def}"; }
# Ask a yes/no question. Capital letter = what Enter means. Returns yes (0) or no (1).
confirm() {   # confirm "question" [default y|n]
    local def=${2:-y} a
    [ "${ASSUME_YES:-0}" = 1 ] && { [ "$def" = y ]; return; }
    read -r -p "$(Lt "$1") ($( [ "$def" = y ] && echo 'Y/n' || echo 'y/N' )): " a
    a=${a:-$def}; [ "${a,,}" = y ] || [ "${a,,}" = yes ]
}

# Show a command, then run it. You always see exactly what runs.
run() { printf '   \033[36m$ %s\033[0m\n' "$*"; "$@"; }

# The "open book" box shown before every install step.
explain() {   # explain "Title" "what it is" "why" "official page"
    printf '\n\033[1;34m+-- %s\033[0m\n' "$(Lt "$1")"
    printf '\033[1;34m|\033[0m  %s %s\n' "$(Lt What:)" "$(Lt "$2")"
    printf '\033[1;34m|\033[0m  %s %s\n' "$(Lt Why:)" "$(Lt "$3")"
    [ -n "${4:-}" ] && printf '\033[1;34m|\033[0m  %s %s\n' "$(Lt Read:)" "$4"
    printf '\033[1;34m+------\033[0m\n'
}

# Save everything this run prints into logs/, so you can read it later.
start_log() {
    mkdir -p logs
    LOG="logs/$1-$(date +%Y-%m-%d_%H%M).log"
    exec > >(tee -a "$LOG") 2>&1
    note "A copy of everything shown here is being saved to: $HERE/$LOG"
}

# Read / change one setting in .env without printing anything else.
env_get() { [ -f .env ] && grep -m1 "^$1=" .env | cut -d= -f2-; }
env_set() {
    local key=$1 val=$2 tmp
    tmp=$(mktemp)
    if grep -q "^$key=" .env; then
        awk -v k="$key" -v v="$val" 'BEGIN{FS=OFS="="} $1==k{print k "=" v; next} {print}' .env > "$tmp"
    else
        cat .env > "$tmp"; printf '%s=%s\n' "$key" "$val" >> "$tmp"
    fi
    cat "$tmp" > .env; rm -f "$tmp"; chmod 600 .env
}
# Is an optional part (beds, wiki, n8n, gate) switched on in COMPOSE_PROFILES?
has_part() { case ",$(env_get COMPOSE_PROFILES)," in *",$1,"*) return 0 ;; *) return 1 ;; esac; }
# Switch an optional part on (adds it to COMPOSE_PROFILES in .env).
part_on()  { has_part "$1" || env_set COMPOSE_PROFILES "$(env_get COMPOSE_PROFILES | sed 's/^,//;s/,$//')${1:+,$1}"; env_set COMPOSE_PROFILES "$(env_get COMPOSE_PROFILES | sed 's/^,//')"; }
# Switch an optional part off (removes it from COMPOSE_PROFILES in .env).
part_off() { env_set COMPOSE_PROFILES "$(env_get COMPOSE_PROFILES | tr ',' '\n' | grep -vx "$1" | paste -sd, -)"; }

# Docker without "sudo" works only after you log out and back in once.
# Until then, use sudo for Docker so the setup can carry on in one go.
dk() { if docker info >/dev/null 2>&1; then docker "$@"; else sudo docker "$@"; fi; }
dc() { dk compose "$@"; }
# "--profile tools --profile beds ..." for every part that is on, so a build
# or download covers exactly what will run.
profile_args() { local p; printf -- "--profile tools"; for p in $(env_get COMPOSE_PROFILES | tr "," " "); do printf -- " --profile %s" "$p"; done; }

# Run a job in the toolbox container (backup, restore, set-password).
tools() {
    dc run --rm -T -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" \
        -e FROM_COMPUTER="$(hostname)" "$@"
}
# Like tools(), but keeps the keyboard attached so a hidden password can be typed.
tools_tty() {   # the same, but with a keyboard attached (for typing a password)
    dc run --rm -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" "$@"
}

# The website's port on this computer (SITE_PORT in .env, normally 8030).
site_port() { local p; p=$(env_get SITE_PORT); printf '%s' "${p:-8030}"; }
# The website's address on this computer, e.g. http://localhost:8030
site_url()  { printf 'http://localhost:%s' "$(site_port)"; }

# Wait up to 5 minutes for the website's health page to answer.
wait_site() {
    local i
    for i in $(seq 1 60); do
        if curl -fsS --max-time 3 "$(site_url)/healthz" 2>/dev/null; then echo; return 0; fi
        [ "$i" = 1 ] && note "waiting for the website to start..."
        sleep 5
    done
    fail "the website did not answer in 5 minutes. Look at:  bash garden.sh logs"
}

# The file name of the newest backup in the backups folder (or nothing).
newest_backup() {
    local dir; dir=$(env_get BACKUP_DIR); dir=${dir:-./backups}
    find "$dir" -maxdepth 1 -name 'junsgarden-backup_*.tar' -printf '%T@ %f\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-
}

# Wait for Enter before the menu comes back, so you can read the screen.
pause() { [ "${ASSUME_YES:-0}" = 1 ] || { echo; read -r -p "$(Lt "Press Enter to go back to the menu...")" _; }; }

# ---------- what the install will do (shown first, before anything changes) ------
show_plan() {
    Lblock <<'EOF'

  WHAT THIS INSTALL DOES  (full details with links: WHAT-GETS-INSTALLED.md)
  Nothing is hidden. Each step explains itself and shows its commands first.

  NECESSARY (the website cannot run without these)
    - Ubuntu safety updates and a few standard tools (git, curl, openssl)
    - Firewall (ufw): blocks every incoming connection except SSH from home
    - SSH server: so YOU can use this laptop's terminal from another computer
    - Docker: runs each program in its own sealed box (a "container")
    - MariaDB database (in Docker): your plants, comments and logins
    - Your garden website (in Docker), built here from the Rust code in rust/
    - Your plants and photos, loaded from the backup in backups/
    - Toolbox + nightly backup at 2:30 am (kept 7 days)

  OPTIONAL (recommended; you choose yes or no next)
    - Garden tools (/beds): bed planner, checkup, diagnose, tag scan, spend
    - Wiki: your garden encyclopedia (Wiki.js) with all its pages
    - Email: lets the site send password links, with YOUR Gmail
    - A "Jun's Garden" icon on the desktop

  ADVANCED OPTIONAL (later from the menu, when you are ready)
    - The front door: Caddy (HTTPS), your sign-in page (Authelia), the
      No-IP updater, and the router: only ports 80 and 443 open
    - n8n email automations, Adminer database viewer, local AI on another PC

  NOT DONE, EVER: no accounts for anybody else, no remote access for anybody
  else, no paid services, nothing sent anywhere except the downloads from the
  official sources listed in WHAT-GETS-INSTALLED.md.

  WHAT COMES FROM THE AUTHOR: only the website and its code (site/, rust/, beds/),
  this setup program and the guides, and YOUR garden (plants, photos, wiki
  pages, in backups/). Every file is listed with its fingerprint in
  FILES-FROM-AUTHOR.txt. Menu M has a screen to look through them.
EOF
}

# ---------- install: every step from a fresh Ubuntu to a working site ------------
check_system() {
    say "Checking this computer"
    if [ "$(id -u)" = 0 ]; then fail "do not run this as root. Run it as your normal user; it asks for sudo when needed."; return 1; fi
    # shellcheck disable=SC1091
    . /etc/os-release
    case " ${ID:-} ${ID_LIKE:-} " in
        *" ubuntu "*|*" debian "*) note "System: ${PRETTY_NAME:-Linux}" ;;
        *) warn "This is ${PRETTY_NAME:-not Ubuntu}. The steps are written for Ubuntu 24.04." ;;
    esac
    [ "$(uname -m)" = x86_64 ] || [ "$(uname -m)" = aarch64 ] || { fail "this processor ($(uname -m)) cannot run Docker"; return 1; }
    note "Memory: $(free -h | awk '/^Mem:/{print $2}')   Free disk: $(df -h --output=avail "$HERE" | tail -1 | tr -d ' ')"
    note "'sudo' means 'do this as the administrator'. Ubuntu asks for YOUR password."
    sudo true || { fail "sudo did not accept the password"; return 1; }
}

# Install step 2: show the folder the garden lives in and ask if that is right.
confirm_location() {
    say "Where your garden lives"
    note "Folder: $HERE"
    note "Everything (settings, photos, database, backups) stays inside this folder."
    if ! confirm "Is this the right place for it?" y; then
        note "OK. Move the whole folder where you want it (for example your home folder),"
        note "open a terminal there and run:  bash garden.sh install"
        return 1
    fi
}

# Install step 3: Ubuntu updates and small standard tools, from Ubuntu's own store.
install_packages() {
    explain "Ubuntu updates and standard tools" \
        "apt is Ubuntu's own app store for programs. It installs: git (keeps a history of your changes), curl (fetches web pages), openssl (makes random passwords), rsync (copies folders), ca-certificates (trusted HTTPS list)." \
        "Safety updates first; the other tools are used by the steps below." \
        "https://ubuntu.com/server/docs/package-management"
    run sudo apt-get -o DPkg::Lock::Timeout=900 update -qq || warn "apt update had problems (check the internet connection)"
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq git curl openssl rsync unzip ca-certificates >/dev/null ||
        { fail "could not install the basic programs. Right after a fresh Ubuntu install, Ubuntu updates itself for 10-30 minutes: wait 15 minutes, then choose 1 again"; return 1; }
    note "done"
}

# Install step 4: the SSH server, so you can manage this computer from another one.
install_ssh() {
    explain "SSH server (OpenSSH)" \
        "Lets you open this laptop's terminal from another computer at home (ssh you@laptop). It needs YOUR password or a key you made." \
        "So the laptop can sit in a corner with the lid shut and you still manage it. The firewall step allows it only from your home network." \
        "https://ubuntu.com/server/docs/openssh-server"
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq openssh-server >/dev/null || { fail "SSH server did not install"; return 1; }
    run sudo systemctl enable --now ssh >/dev/null 2>&1 || true
}

# Docker Desktop (a Windows program) can share its own Docker with an Ubuntu in
# WSL ("WSL integration"): it then puts links to itself in /usr/bin. The garden
# uses its own Docker inside Ubuntu instead, so it works the same as on any Ubuntu
# computer and keeps running when Docker Desktop is closed.
docker_desktop_shared() {
    grep -qi microsoft /proc/version 2>/dev/null || return 1
    local f
    for f in /usr/bin/docker /usr/local/bin/docker; do
        [ -L "$f" ] && case "$(readlink "$f")" in *docker-desktop*) return 0 ;; esac
    done
    return 1
}
# Which Docker runs the garden. "desktop": Docker Desktop on Windows, the easy way
# with buttons (chosen in the Windows installer, saved in .garden-docker).
# Otherwise "engine": Docker Engine inside Ubuntu, the same on every computer.
docker_mode() { if [ "$(cat .garden-docker 2>/dev/null)" = desktop ]; then echo desktop; else echo engine; fi; }
# The Windows installer passes its choice as GARDEN_DOCKER=desktop or engine.
case "${GARDEN_DOCKER:-}" in desktop|engine) printf '%s\n' "$GARDEN_DOCKER" > .garden-docker ;; esac
# Docker Desktop's way: its Docker must reach this Ubuntu ("WSL integration" on).
docker_desktop_ready() {
    local os
    os=$(dk info --format '{{.OperatingSystem}}' 2>/dev/null)
    case "$os" in *"Docker Desktop"*) note "Docker Desktop is connected to this Ubuntu: $(dk --version)"; return 0 ;; esac
    warn "Docker Desktop is not connected to this Ubuntu yet."
    Lblock <<'EOF'
   1. Start Docker Desktop on Windows (its whale icon appears near the clock).
   2. Docker Desktop > Settings (the gear) > Resources > WSL integration:
      switch ON this Ubuntu (JunsGarden), then click Apply & restart.
   3. Run the install again: it carries on from here.
EOF
    [ -x /usr/bin/dockerd ] && note "This Ubuntu also has its own Docker Engine (from an earlier install). Docker Desktop's way needs it gone first: sudo apt remove docker-ce docker-ce-cli (guide 25)."
    return 1
}
docker_desktop_note() {
    warn "Docker Desktop (on Windows) is sharing its own Docker with this Ubuntu."
    Lblock <<'EOF'
   The garden uses its own Docker inside this Ubuntu, so it runs the same way for
   everyone and keeps running when Docker Desktop is closed. Docker Desktop stays
   as it is for your other projects. One switch keeps the two apart:
     Docker Desktop > Settings > Resources > WSL integration:
     switch OFF this Ubuntu (JunsGarden), then click Apply & restart.
   Then run the install again: it carries on from here. (Guide 25 explains more.)
EOF
}

# Install step 6: Docker Engine and Compose, from Docker's official Ubuntu source.
install_docker() {
    explain "Docker Engine and Docker Compose" \
        "Docker runs each program in its own sealed box (a container) with everything it needs. Compose starts all the boxes from one list: docker-compose.yml in this folder." \
        "Programs cannot mess up Ubuntu or each other, and the same setup works on any computer." \
        "https://docs.docker.com/engine/install/ubuntu/  (the official steps this follows)"
    # The easy way on Windows: Docker Desktop is the Docker; nothing to install here.
    if [ "$(docker_mode)" = desktop ]; then docker_desktop_ready; return; fi
    if docker_desktop_shared; then docker_desktop_note; return 1; fi
    if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
        note "Docker is already installed: $(docker --version)"
    else
        local pkg codename
        for pkg in docker.io docker-compose docker-compose-v2 podman-docker containerd runc; do
            sudo apt-get -o DPkg::Lock::Timeout=900 remove -y -qq "$pkg" >/dev/null 2>&1 || true
        done
        # Ubuntu uses Docker's "ubuntu" source; Raspberry Pi OS and Debian use "debian".
        local distro=ubuntu
        # shellcheck disable=SC1091
        [ "$(. /etc/os-release && echo "$ID")" = ubuntu ] || distro=debian
        run sudo install -m 0755 -d /etc/apt/keyrings
        note "Adding Docker's signing key, so Ubuntu can check the downloads really come from Docker:"
        run sudo curl -fsSL "https://download.docker.com/linux/$distro/gpg" -o /etc/apt/keyrings/docker.asc ||
            { fail "could not download Docker's key (internet?)"; return 1; }
        run sudo chmod a+r /etc/apt/keyrings/docker.asc
        # shellcheck disable=SC1091
        codename=$(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}")
        note "Adding Docker's official download source (download.docker.com, $distro):"
        echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/$distro $codename stable" |
            sudo tee /etc/apt/sources.list.d/docker.list
        run sudo apt-get -o DPkg::Lock::Timeout=900 update -qq
        run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq docker-ce docker-ce-cli containerd.io \
            docker-buildx-plugin docker-compose-plugin >/dev/null || { fail "Docker did not install"; return 1; }
    fi
    run sudo systemctl enable --now docker >/dev/null 2>&1 || true
    if ! id -nG "$USER" | has -w docker; then
        run sudo usermod -aG docker "$USER"
        note "You are now in the 'docker' group. After you next log out and in,"
        note "Docker works without sudo. Until then this program uses sudo for you."
    fi
    dk version --format '   Docker engine {{.Server.Version}} is running' || { fail "Docker is installed but not running:  sudo systemctl status docker"; return 1; }
    docker_dns_check
}

# Containers must be able to look up web names (to download while building).
# Some home networks and Windows test machines use a DNS address that
# containers cannot reach. If so, give Docker two well-known public DNS
# servers (Cloudflare 1.1.1.1 and Google 8.8.8.8) and say so.
docker_dns_check() {
    note "Checking that containers can reach the internet..."
    if dk run --rm alpine:3.20 nslookup download.docker.com >/dev/null 2>&1; then note "yes"; return 0; fi
    explain "Fixing Docker's DNS" \
        "DNS turns names like download.docker.com into addresses. Containers here could not reach this computer's DNS, so Docker is told to use 1.1.1.1 (Cloudflare) and 8.8.8.8 (Google)." \
        "Without it the build cannot download anything. The setting is in /etc/docker/daemon.json." \
        "https://docs.docker.com/reference/cli/dockerd/#daemon-configuration-file"
    if [ -f /etc/docker/daemon.json ] && [ -s /etc/docker/daemon.json ]; then
        warn "/etc/docker/daemon.json already exists; not changing it. Add  \"dns\": [\"1.1.1.1\", \"8.8.8.8\"]  to it."
        return 1
    fi
    echo '{ "dns": ["1.1.1.1", "8.8.8.8"] }' | sudo tee /etc/docker/daemon.json
    run sudo systemctl restart docker
    sleep 3
    if dk run --rm alpine:3.20 nslookup download.docker.com >/dev/null 2>&1; then note "Fixed."; return 0; fi
    fail "containers still cannot reach the internet. Check this computer's internet, then run the install again."
}

# Install step 7: add 4 GB of swap on small computers, so the build does not run out of memory.
make_swap() {
    local mem swp
    mem=$(free -m | awk '/^Mem:/{print $2}'); swp=$(free -m | awk '/^Swap:/{print $2}')
    if [ $((mem + swp)) -lt 6000 ] && [ ! -f /swapfile-garden ]; then
        explain "Swap file (4 GB)" \
            "Disk space Ubuntu may use as extra memory when memory is full." \
            "This computer has ${mem} MB of memory; building the website needs about 3 GB. Without swap the build can stop half way." \
            "https://manpages.ubuntu.com/manpages/noble/man8/swapon.8.html"
        { run sudo fallocate -l 4G /swapfile-garden && run sudo chmod 600 /swapfile-garden &&
          run sudo mkswap /swapfile-garden >/dev/null && run sudo swapon /swapfile-garden &&
          echo '/swapfile-garden none swap sw 0 0' | sudo tee -a /etc/fstab >/dev/null; } ||
            warn "could not add swap; the first build may be slow or fail on a small laptop"
    fi
}

# Your home network's range, e.g. 192.168.1.0/24 (used for the SSH firewall rule).
# Your home network: the one on the connection that leads to the router (the
# "default route"), never Docker's own inside network (172.17...), which also
# has an address once Docker is installed.
home_subnet() {
    local dev; dev=$(ip -4 route show default 2>/dev/null | awk '{print $5; exit}')
    [ -n "$dev" ] || return 0
    ip -4 route show dev "$dev" 2>/dev/null | awk '!/default/ && /src/ {print $1; exit}'
}

# Install step 5: ufw blocks incoming connections except SSH from your home network.
firewall() {
    local net; net=$(home_subnet)
    explain "Firewall (ufw)" \
        "ufw = Uncomplicated Firewall. It decides which connections may come INTO this laptop. Rule: block everything, except SSH from your home network (${net:-your home network})." \
        "The website, database and tools only listen on the laptop itself (127.0.0.1). Later the front door adds ports 80 and 443, the only ones the internet will ever reach." \
        "https://ubuntu.com/server/docs/how-to/security/firewalls/"
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq ufw >/dev/null
    run sudo ufw default deny incoming >/dev/null
    run sudo ufw default allow outgoing >/dev/null
    # Allow SSH on the port(s) it REALLY uses (22, or another one if it was
    # moved), so switching the firewall on never cuts off your own SSH.
    local p ports
    ports=$(sudo sshd -T 2>/dev/null | awk '$1 == "port" {print $2}' | sort -u)
    [ -n "$ports" ] || ports=22
    for p in $ports; do
        if [ -n "$net" ]; then run sudo ufw allow from "$net" to any port "$p" proto tcp comment 'SSH from home only' >/dev/null
        else run sudo ufw allow "$p/tcp" comment 'SSH' >/dev/null; fi
        note "SSH stays open on port $p ($([ -n "$net" ] && echo "from $net only" || echo "from anywhere"))"
    done
    # Already-open doors from before (other programs on this computer) are kept:
    # ufw only adds rules here, it removes none.
    if run sudo ufw --force enable >/dev/null 2>&1; then note "$(sudo ufw status | head -1)"
    else warn "could not turn on the firewall (normal inside WSL or a container)"; fi
}

# Install step 8: write .env (settings + random passwords), or add missing settings to it.
make_env() {
    explain "Your settings file (.env)" \
        "One private text file with every setting: your name and email, folder names, and long random passwords the programs use to talk to each other (you never type those)." \
        "Settings live in one place you can read (nano .env), never inside the code. Only you can open it (permission 600)." \
        "https://docs.docker.com/compose/how-tos/environment-variables/variable-interpolation/"
    if [ ! -f .env ]; then
        local name=${GARDEN_NAME:-} email=${GARDEN_EMAIL:-}
        [ -n "$name" ]  || name=$(ask "Your first name for the login (letters only)" "Jun")
        [ -n "$email" ] || email=$(ask "Your email address (password links go here)")
        case "$name" in *[!A-Za-z]*|"") fail "the name must be letters only"; return 1 ;; esac
        case "$email" in *@*.*) ;; *) fail "that does not look like an email address"; return 1 ;; esac
        # The database's password (the website and Adminer use it; you never
        # need it to log in to your garden). Enter = a strong one made for you.
        local dbpw=""
        if [ "${ASSUME_YES:-0}" != 1 ] && [ -t 0 ]; then
            note "Your database (MariaDB, the same language as MySQL) needs a password that the"
            note "website uses to talk to it. Press Enter and a strong one is made for you"
            note "(recommended: you never type it; it is kept in .env, menu 8 can renew it)."
            note "Or type your own: 16 or more letters and numbers (no spaces or symbols)."
            dbpw=$(ask_secret "Database password (Enter = make a strong one for me)")
            if [ -n "$dbpw" ] && ! [[ "$dbpw" =~ ^[A-Za-z0-9]{16,}$ ]]; then
                fail "16 or more letters and numbers only. Run the install again (menu 1)."; unset dbpw; return 1
            fi
        fi
        ( umask 077
          cat > .env <<EOF
# Your private settings. NEVER share this file or paste it into a chat.
TZ=$(tz=$(timedatectl show -p Timezone --value 2>/dev/null); [ -n "$tz" ] || tz=$(cat /etc/timezone 2>/dev/null); echo "${tz:-America/New_York}")

# Long random passwords, made by the setup. You never need to type them.
DB_ROOT_PASSWORD=$(openssl rand -hex 24)
DB_PASSWORD=${dbpw:-$(openssl rand -hex 24)}
WIKI_DB_PASSWORD=$(openssl rand -hex 24)
AUTHELIA_JWT_SECRET=$(openssl rand -hex 32)
AUTHELIA_SESSION_SECRET=$(openssl rand -hex 32)
AUTHELIA_STORAGE_KEY=$(openssl rand -hex 32)

# Who may log in and edit: Name=email  (more than one: Name=email,Name2=email2)
GARDEN_USERS=$name=$email
GARDEN_UPLOADER_NAMES=$name
# Who may use the garden tools (/beds) after signing in. Emails, comma between.
BEDS_ALLOWED_EMAILS=$email
# Your user number on this computer: the programs save files as you.
HOST_UID=$(id -u)
HOST_GID=$(id -g)
# Your own wiki, for the garden tools' wiki queue (filled in by the wiki setup).
WIKIJS_API_URL=
WIKIJS_API_KEY=
# Your wiki admin login (the wiki setup uses it) and its address.
WIKI_ADMIN_EMAIL=$email
WIKI_SITE_URL=http://localhost:3000

# Extra backup copies (menu: Backups > NAS / Google Drive). Empty = not used.
BACKUP_NAS_DIR=
BACKUP_CLOUD=
BACKUP_NAS_HOST=
NAS_FULL_COPY=
HOME_ONLY=

# Only if your internet company blocks ports 80/443 (or for testing): other ports for Caddy.
CADDY_HTTP_PORT=
CADDY_HTTPS_PORT=

# DuckDNS name and subdomains (menu M, Subdomains). SUBDOMAINS: sub:port:gate|open,...
DUCKDNS_DOMAIN=
SUBDOMAINS=

# LiteLLM AI gateway (menu M, 15): its own key for your projects, and a monthly limit.
LITELLM_MASTER_KEY=sk-$(openssl rand -hex 24)
LITELLM_BUDGET=
LITELLM_PORT=4000

# Which optional parts run (garden.sh parts changes this):
#   beds = garden tools, wiki = wiki, n8n = automations, gate = sign-in page
COMPOSE_PROFILES=

# Where visitors find the site (the front door step fills these in).
PUBLIC_URL=http://localhost:8030
PUBLIC_NAME=
WIKI_NAME=
SITE_PORT=8030
# This laptop's reserved address on your home network (garden.sh network).
LAPTOP_IP=

# FOLDERS. To rename one: stop the site, rename the folder, change the line
# here, start the site. Guide 13 shows how. Keep the ./ at the front.
SITE_DIR=./site
IMAGES_DIR=./local/images
DATA_DIR=./local/data
CACHE_DIR=./local/cashe
DB_DIR=./local/db
N8N_DIR=./local/n8n
AUTH_DIR=./local/authelia
BACKUP_DIR=./backups

# Email (garden.sh email fills these in). Gmail needs an APP password.
SMTP_HOST=
SMTP_PORT=587
SMTP_USER=
SMTP_PASSWORD=

# AI and plant-data keys. Menu 8 Settings, 4 (API keys) pastes, tests and replaces them.
GEMINI_API_KEY=
OPENROUTER_API_KEY=
PERENUAL_KEY=
# Optional local AI on another computer (garden.sh ai).
OLLAMA_URL=
OLLAMA_MODEL=
EOF
        )
        unset dbpw
        note "Wrote .env (only you can read it). The passwords in it were never shown."
    else
        note ".env already exists, keeping it. Adding any settings it is missing:"
        local pair key
        for pair in "SITE_PORT=8030" "SITE_DIR=./site" "IMAGES_DIR=./local/images" "DATA_DIR=./local/data" \
                    "CACHE_DIR=./local/cashe" "DB_DIR=./local/db" "N8N_DIR=./local/n8n" "AUTH_DIR=./local/authelia" \
                    "BACKUP_DIR=./backups" "COMPOSE_PROFILES=" "PUBLIC_NAME=" "WIKI_NAME=" "LAPTOP_IP=" \
                    "OPENROUTER_API_KEY=" "PERENUAL_KEY=" "OLLAMA_URL=" "OLLAMA_MODEL=" \
                    "BEDS_ALLOWED_EMAILS=$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)" \
                    "HOST_UID=$(id -u)" "HOST_GID=$(id -g)" "WIKIJS_API_URL=" "WIKIJS_API_KEY=" \
                    "WIKI_ADMIN_EMAIL=$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)" "WIKI_SITE_URL=http://localhost:3000" \
                    "BACKUP_NAS_DIR=" "BACKUP_CLOUD=" "DUCKDNS_DOMAIN=" "SUBDOMAINS=" \
                    "BACKUP_NAS_HOST=" "NAS_FULL_COPY=" "CADDY_HTTP_PORT=" "CADDY_HTTPS_PORT=" \
                    "LITELLM_MASTER_KEY=sk-$(openssl rand -hex 24)" "LITELLM_BUDGET=" "LITELLM_PORT=4000" \
                    "WIKI_DB_PASSWORD=$(openssl rand -hex 24)" "AUTHELIA_JWT_SECRET=$(openssl rand -hex 32)" \
                    "AUTHELIA_SESSION_SECRET=$(openssl rand -hex 32)" "AUTHELIA_STORAGE_KEY=$(openssl rand -hex 32)"; do
            key=${pair%%=*}
            grep -q "^$key=" .env || { echo "$pair" >> .env; note "added $key"; }
        done
    fi
    chmod 600 .env
}

# Install step 9: ask which optional parts to install (garden tools, wiki, n8n).
choose_parts() {
    say "Choose the OPTIONAL parts (you can change this later: menu 'Parts')"
    env_set PARTS_CHOSEN yes
    # EASY mode: the key parts of the old site, chosen for you.
    if [ "$GARDEN_MODE" = easy ] && [ -z "${GARDEN_PARTS+set}" ]; then
        part_on beds; part_off wiki; part_off n8n
        note "EASY mode chose: the garden tools (/beds). The wiki and n8n are left for you"
        note "to add if you want them (menu 9, Parts; the wiki's pages are a separate download)."
        note "To change this: menu 9 (Parts), or run the install again and choose CUSTOM."
        return 0
    fi
    # Already chosen in the Windows guided installer? Then use those choices.
    if [ -n "${GARDEN_PARTS+set}" ]; then
        local p
        for p in beds wiki n8n; do
            case ",$GARDEN_PARTS," in *",$p,"*) part_on "$p" ;; *) part_off "$p" ;; esac
        done
        note "Chosen in the Windows installer. Parts on: $(env_get COMPOSE_PROFILES)"
        return 0
    fi
    explain "Garden tools (/beds)" \
        "A small Python program: bed planner, plant checkup, diagnose, tag scan, spend list, wiki queue. It also reads tag photos for the phone app." \
        "They were part of the live garden site. They open only after signing in on your sign-in page (front door step)." \
        "beds/README.md"
    if confirm "Install the garden tools?" y; then part_on beds; else part_off beds; fi
    explain "Wiki (Wiki.js)" \
        "Your garden encyclopedia: a website of articles. You make your own admin account. The garden's pages from the old wiki are a separate download (junsgarden-wiki-addon.zip); without it the wiki starts empty." \
        "Only if you want it: it downloads one more program (Wiki.js), and adding the pages takes about 30 minutes. You can add it any time later: menu 9 (Parts)." \
        "https://docs.requarks.io  (Wiki.js documentation)"
    if confirm "Install the wiki?" n; then part_on wiki; else part_off wiki; fi
    explain "n8n automations (ADVANCED)" \
        "A drag-and-drop tool for 'when this happens, do that' (for example: soil is dry -> send me an email). Private to this laptop." \
        "Fun for later. Guides: START-HERE/n8n/. You can add it any time with: bash garden.sh parts" \
        "https://docs.n8n.io"
    if confirm "Install n8n now?" n; then part_on n8n; else part_off n8n; fi
    note "Parts on: $(env_get COMPOSE_PROFILES)"
}

# Install step 10: make the folders the programs use, with the right permissions.
make_folders() {
    say "Making the folders the programs use"
    local d
    for d in "$(env_get IMAGES_DIR)" "$(env_get DATA_DIR)" "$(env_get CACHE_DIR)" \
             "$(env_get DB_DIR)" "$(env_get BACKUP_DIR)" "$(env_get N8N_DIR)" "$(env_get AUTH_DIR)"; do
        mkdir -p "$d" && note "$d"
    done
    # The picture folders the website expects (with their new names, if any
    # were renamed with Settings 8: IMAGE_FOLDERS in .env).
    local folders; folders=$(env_get IMAGE_FOLDERS); folders=${folders:-$IMAGE_FOLDERS_FIRST}
    for d in $folders; do mkdir -p "$(env_get IMAGES_DIR)/$d"; done
    mkdir -p "$(env_get CACHE_DIR)/AI"
    chmod -R a+rwX "$(env_get IMAGES_DIR)" "$(env_get DATA_DIR)" "$(env_get CACHE_DIR)" "$(env_get N8N_DIR)" 2>/dev/null || true
    chmod 700 "$(env_get AUTH_DIR)" 2>/dev/null || true
}

# Install step 11: start a git history so every change can be compared and undone.
git_start() {
    command -v git >/dev/null 2>&1 || return 0
    [ -d .git ] && return 0
    explain "A git history of your changes" \
        "git remembers every saved version of the code and settings examples (not your photos, not .env)." \
        "Any change you make can be compared (git diff) and undone (git checkout -- FILE). Guide 11." \
        "https://git-scm.com/book/en/v2/Getting-Started-What-is-Git%3F"
    # Your LOCAL history skips the two big files that never change (the starter
    # backup, 800 MB, and the phone app, 45 MB): they are safe in the zip already,
    # and copying them into the history would use the disk twice.
    { run git init -q -b main . &&
        printf 'backups/*.tar\nsite/apps/*.apk\n' >> .git/info/exclude &&
        git config user.name "$(env_get GARDEN_USERS | cut -d= -f1)" &&
        git config user.email "$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)" &&
        git add -A && git commit -q -m "My garden as it came from the zip" && note "first save made"; } ||
        warn "git history was not started (the garden works without it)"
    return 0
}

# A help file for someone helping you: what happened, with NO passwords.
do_helpme() {
    local f d
    mkdir -p help
    f="help/junsgarden-help-$(date +%Y-%m-%d_%H%M).txt"
    say "Making a help file (passwords and keys are removed)"
    {
        echo "Jun's Garden help file, made $(date) on $(hostname)"
        echo; echo "== Install progress (where it got to)"; cat "$PROGRESS" 2>/dev/null || echo "(no install yet)"
        echo; echo "== System"; . /etc/os-release 2>/dev/null; echo "${PRETTY_NAME:-?}  $(uname -m)"; free -h | head -2; df -h "$HERE" | tail -1
        echo; echo "== Docker"; docker --version 2>&1; dk compose version 2>&1; dk compose ps -a 2>&1
        echo; echo "== Settings (.env, secret values hidden)"
        [ -f .env ] && sed -E 's/^([A-Z_]*(PASSWORD|SECRET|KEY|TOKEN)[A-Z_]*)=.+/\1=(hidden)/' .env | grep -v '^#' | grep -v '^$'
        echo; echo "== Checklist"; do_check 2>&1 | grep -E '^(PASS|WARN|FAIL|INFO|RESULT|        ->)'
        for d in $(find logs -name '*.log' -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -2 | cut -d' ' -f2); do
            echo; echo "== Last 150 lines of $d"
            tail -150 "$d" | sed -E 's/(password|secret|key|token)([^ ]*)[=:] *[^ ]+/\1\2=(hidden)/Ig'
        done
        echo; echo "== Last 60 lines of each garden program's log"
        for d in garden-site garden-db junsgarden-ingest garden-wiki garden-auth; do
            dk inspect "$d" >/dev/null 2>&1 || continue
            echo "-- $d"; dk logs --tail 60 "$d" 2>&1 | sed -E 's/(password|secret|key|token)([^ ]*)[=:] *[^ ]+/\1\2=(hidden)/Ig'
        done
    } > "$f" 2>&1
    note "Saved: $HERE/$f"
    note "Read it first (it is plain text), then send it to whoever is helping, or paste it into Gemini."
}

# Every container image this garden can download, where it comes from, and
# why. Shown BEFORE anything is downloaded. "Official Image" means Docker and
# the program's own makers maintain it.  part|image|what|why|page
IMAGES_LIST=(
    "core|mariadb:11|MariaDB database (Docker Official Image)|stores plants, logins, wiki pages|https://hub.docker.com/_/mariadb"
    "core|rust:1.95-slim-bookworm|Rust compiler (Docker Official Image), used only while building|turns the website's code in rust/ into a program|https://hub.docker.com/_/rust"
    "core|debian:bookworm-slim|Debian Linux, small (Docker Official Image)|the base the finished website runs on|https://hub.docker.com/_/debian"
    "core|alpine:3.20|Alpine Linux, tiny (Docker Official Image)|the base of the toolbox (backup, restore, passwords)|https://hub.docker.com/_/alpine"
    "beds|python:3.12-slim|Python (Docker Official Image)|the base of the garden tools in beds/|https://hub.docker.com/_/python"
    "wiki|ghcr.io/requarks/wiki:2|Wiki.js, from its makers on GitHub|your garden wiki|https://github.com/requarks/wiki"
    "gate|authelia/authelia:4.39|Authelia, from its makers|your sign-in page|https://hub.docker.com/r/authelia/authelia"
    "n8n|docker.n8n.io/n8nio/n8n|n8n, from its makers' own registry|automations|https://docs.n8n.io/hosting/installation/docker/"
)
# Show every container image that will be downloaded, and its official page.
explain_images() {
    say "What will be downloaded (container images), and from where"
    note "Docker downloads ready-made 'images' (a sealed copy of a program) from these"
    note "official places. Open any page to see who makes it. Nothing else is downloaded."
    local entry part img what why page
    for entry in "${IMAGES_LIST[@]}"; do
        IFS='|' read -r part img what why page <<<"$entry"
        [ "$part" = core ] || has_part "$part" || continue
        printf '\n   \033[1m%s\033[0m\n      %s\n      why: %s\n      page: %s\n' "$img" "$what" "$why" "$page"
    done
    note ""
    note "Built HERE from the code in this folder (you can read every line first):"
    note "   the website       rust/Dockerfile + rust/"
    note "   the toolbox       docker/tools/Dockerfile + garden-tools.sh"
    has_part beds && note "   the garden tools  beds/Dockerfile + beds/*.py"
    has_part wiki && note "   the wiki setup    docker/wiki/"
    note "The list of every file that came in the zip, with fingerprints: FILES-FROM-AUTHOR.txt"
}

# Install step 12: download, build, start the database, load your garden, start all.
build_and_start() {
    explain_images
    explain "Downloading fresh copies of those images" \
        "docker compose pull gets the newest copy of each image above, straight from its official place, and checks its fingerprint (digest)." \
        "Fresh copies have the latest safety fixes." \
        "https://docs.docker.com/reference/cli/docker/compose/pull/"
    # shellcheck disable=SC2046  # the profile list is meant to split into words
    run dc $(profile_args) pull --ignore-buildable || warn "some images did not download; the build below tries again"
    explain "Building your programs (Docker images)" \
        "Docker downloads the official base images (Rust, Debian, Alpine, MariaDB, Python, Wiki.js) and builds the website from the Rust code in rust/. Each recipe is a 'Dockerfile' you can read." \
        "Building here means the code you see is exactly the code that runs." \
        "https://docs.docker.com/get-started/docker-concepts/building-images/"
    note "First time: 10 to 60 minutes on an old laptop. Text scrolls by; that is normal."
    docker_dns_check || return 1
    # shellcheck disable=SC2046  # the profile list is meant to split into words
    run dc $(profile_args) build || { fail "the build failed. Look at the last red lines above, or ask Gemini with them."; return 1; }

    explain "The database (MariaDB)" \
        "MariaDB is a free database that speaks the same language as MySQL. It keeps plants, comments, logins and the wiki's pages." \
        "It only talks to the other containers, never to the network." \
        "https://mariadb.com/kb/en/documentation/"
    run dc up -d db || { fail "the database did not start:  bash garden.sh logs"; return 1; }
    tools tools wait-db || return 1

    local plants backup
    plants=$(tools tools plant-count 2>/dev/null | tail -1)
    backup=$(newest_backup)
    if [ "${plants:-0}" = 0 ] && [ -n "$backup" ]; then
        explain "Loading your plants" \
            "Unpacks $backup: the database, comments and the AI cache. Each piece is checked against its fingerprint (SHA256SUMS) first. The photos come from their own download, next." \
            "So your copy starts with the whole garden, exactly as it was." \
            "START-HERE/13-backups-and-folders.md"
        dc run --rm -T -e NO_SAFETY=1 -e HOST_UID="$(id -u)" -e HOST_GID="$(id -g)" tools restore "$backup" ||
            { fail "loading the backup failed"; return 1; }
    else
        note "The database already has ${plants:-0} plants. Not loading anything."
    fi
    # The photos are a separate download. Load them if they are here; if not,
    # the garden works without pictures until they are (menu 6, then 11).
    # EASY loads them now. CUSTOM may leave them for later (a slow day, a busy network).
    if ! have_photos && [ "$GARDEN_MODE" = custom ] && [ "${ASSUME_YES:-0}" != 1 ] && find_download "$PHOTOS_ZIP" >/dev/null &&
        ! confirm "Load the photos now (a few minutes)? Or later: menu 6, then 11" y; then
        note "Photos left for later: menu 6, then 11 (or: bash garden.sh photos). The garden works without them."
    elif ! have_photos; then
        photos_addon_load || {
            note "No photos yet: they are a separate download, $PHOTOS_ZIP (Google Drive)."
            if [ -n "${GARDEN_REMOTE_INSTALL:-}" ]; then
                note "The garden works without them. Later: put the zip next to Start-Here-Windows.cmd"
                note "(in the garden folder on your PC) and choose INSTALL again in the Windows menu."
            else
                note "The garden works without them. Later: leave the zip in your Home folder and"
                note "choose menu 6, then 11 (or: bash garden.sh photos)."
            fi
        }
    fi

    explain "Starting everything" \
        "Starts the website$(has_part beds && echo ', the garden tools')$(has_part wiki && echo ', the wiki')$(has_part n8n && echo ', n8n'). See them any time with: docker ps" \
        "Each one listens ONLY on this laptop (127.0.0.1). Nothing is reachable from outside yet." ""
    run dc up -d || { fail "something did not start:  bash garden.sh logs"; return 1; }
    wait_site || return 1
    source_fingerprint > .garden-built   # remember: this code is built and running
    if has_part wiki; then wiki_setup || warn "the wiki setup did not finish; run it again later: bash garden.sh wiki"; fi
}

# ---------- the separate downloads (add-ons) ----------------------------------------
# Jun's garden comes as up to three zips from Google Drive:
#   junsgarden-for-jun.zip         the program, the website and the plants database
#   junsgarden-photos-addon.zip    the photos (big; can be downloaded again any time)
#   junsgarden-wiki-addon.zip      the wiki's pages (only if you want the wiki)
# Their fingerprints are in this folder, so a damaged or different download is
# refused. Where the menu looks for them (leave them zipped):
find_download() {   # find_download FILE-NAME  -> prints the path, or returns 1
    local d
    # xdg-user-dir gives the real folder names: in a Korean Ubuntu they are
    # ~/다운로드 and ~/바탕화면, not ~/Downloads and ~/Desktop.
    for d in "$HERE" "$HOME" "$(xdg-user-dir DOWNLOAD 2>/dev/null)" "$(xdg-user-dir DESKTOP 2>/dev/null)" \
             "$HOME/Downloads" "$HOME/Desktop" "$(dirname "$HERE")"; do
        [ -n "$d" ] && [ -f "$d/$1" ] && { printf '%s\n' "$d/$1"; return 0; }
    done
    return 1
}
need_unzip() { command -v unzip >/dev/null 2>&1 || run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq unzip >/dev/null; }

# The photos: unpack junsgarden-photos-addon.zip into your photos folder.
# Photos you already have are kept as they are (nothing is overwritten).
PHOTOS_ZIP=junsgarden-photos-addon.zip
photos_addon_load() {
    local zip want got dir
    zip=$(find_download "$PHOTOS_ZIP") || return 1
    want=$(cut -d' ' -f1 docker/tools/photos.sha256 2>/dev/null)
    need_unzip
    note "Found the photos download: $zip. Checking its fingerprint (about a minute)..."
    got=$(unzip -p "$zip" images.tar | sha256sum | cut -d' ' -f1)
    if [ -z "$want" ] || [ "$got" != "$want" ]; then
        warn "that photos download does not match its fingerprint (damaged, or a different one). Download it again."
        return 1
    fi
    dir=$(env_get IMAGES_DIR); dir=${dir:-./local/images}
    mkdir -p "$dir"
    note "It matches. Unpacking the photos into $dir (photos you already have are kept)..."
    unzip -p "$zip" images.tar | tar -x --skip-old-files -C "$dir" || { fail "unpacking the photos failed"; return 1; }
    chmod -R a+rwX "$dir" 2>/dev/null || true
    note "Photos ready: $(find "$dir" -type f | wc -l) files."
}
have_photos() { [ -n "$(find "$(env_get IMAGES_DIR)" -type f -print -quit 2>/dev/null)" ]; }

# Menu 6, 11 (or: bash garden.sh photos): load the photos download, now or again.
do_photos() {
    explain "Your photos (a separate download)" \
        "The photos are a separate download from the same Google Drive folder: $PHOTOS_ZIP (about 800 MB). This unpacks it into your photos folder. Photos you already have, and photos you added yourself, are kept." \
        "So the first download stays small, and you can fetch the photos when your internet is quiet, or again later." \
        "START-HERE/13-backups-and-folders.md"
    if photos_addon_load; then redo_renames; return 0; fi
    note "Not found. Download $PHOTOS_ZIP from the Google Drive folder and leave it zipped in your"
    note "Home folder (or Downloads, or Desktop), then choose this again (menu 6, then 11)."
    return 1
}

# Menu 6, 12: keep a copy of the downloads (the zips) on your NAS.
nas_copy_downloads() {
    local to f n=0
    to=$(env_get BACKUP_NAS_DIR)
    [ -n "$to" ] && [ -d "$to" ] || { fail "connect your NAS first (menu 6, then 4)"; return 1; }
    mkdir -p "$to/downloads"
    for f in junsgarden-for-jun.zip "$PHOTOS_ZIP" junsgarden-wiki-addon.zip; do
        f=$(find_download "$f") || continue
        run cp -u "$f" "$to/downloads/" && n=$((n + 1))
    done
    note "$n download(s) kept on the NAS in $to/downloads (copied only when new or changed)."
}

# The garden's wiki pages come in a separate download, the wiki add-on
# (junsgarden-wiki-addon.zip), so nobody waits for wiki data they do not want.
# This finds it, checks its fingerprint against docker/wiki/seed/wiki-pages.sha256,
# and puts the pages where the helper reads them. Returns 0 when they are ready.
WIKI_SEED=docker/wiki/seed/wiki-pages.json.gz
wiki_addon_ready() {
    local want zip
    want=$(cut -d' ' -f1 docker/wiki/seed/wiki-pages.sha256 2>/dev/null)
    if [ -f "$WIKI_SEED" ] && [ "$(sha256sum "$WIKI_SEED" | cut -d' ' -f1)" = "$want" ]; then return 0; fi
    zip=$(find_download junsgarden-wiki-addon.zip) || return 1
    need_unzip
    note "Found the wiki add-on: $zip"
    unzip -p "$zip" wiki-pages.json.gz > "$WIKI_SEED.new" 2>/dev/null
    if [ -n "$want" ] && [ "$(sha256sum "$WIKI_SEED.new" | cut -d' ' -f1)" = "$want" ]; then
        mv "$WIKI_SEED.new" "$WIKI_SEED"; note "Its fingerprint matches: the garden's wiki pages are ready."; return 0
    fi
    rm -f "$WIKI_SEED.new"
    warn "that wiki add-on does not match its fingerprint (damaged download, or a different one). Download it again."
    return 1
}

# shellcheck disable=SC2046  # $(profile_args) is meant to split into words
wiki_setup() {
    has_part wiki || return 0
    if ! wiki_addon_ready; then
        note "The wiki will start EMPTY: the garden's wiki pages are a separate download,"
        note "junsgarden-wiki-addon.zip (same Google Drive folder). Put it, still zipped, in this"
        note "folder or in Downloads, then:  bash garden.sh wiki   (it adds the pages then)."
    fi
    explain "Your wiki: first-time setup and the garden pages" \
        "A one-time helper (docker/wiki/wiki_setup.py) finishes Wiki.js's own setup with YOU as its admin ($(env_get WIKI_ADMIN_EMAIL)), then adds all the garden pages, tags and the menu from docker/wiki/seed/. Nothing from the old wiki's logins or keys is in there." \
        "So the encyclopedia is yours. The first time takes about 30 minutes (the pages go in one by one). If it stops, run it again: it only adds what is missing." \
        "docker/wiki/README.md  and  https://docs.requarks.io"
    # shellcheck disable=SC2046  # the profile list is meant to split into words
    run dc $(profile_args) --profile wiki-setup build wiki-setup >/dev/null || { fail "the wiki helper did not build"; return 1; }
    run dc up -d wiki >/dev/null
    # The helper first checks whether your wiki is already set up; only if it
    # is not does it ask for the wiki admin password (the first time only; it
    # is never saved here and can differ from your garden password).
    if [ -n "${GARDEN_TEST_WIKI_PASSWORD:-}" ]; then
        # Only for automatic tests (nobody types): the password comes from this variable.
        printf '%s\n' "$GARDEN_TEST_WIKI_PASSWORD" | dc $(profile_args) --profile wiki-setup run --rm -T wiki-setup
    elif [ -t 0 ] && [ "${ASSUME_YES:-0}" != 1 ]; then
        dc $(profile_args) --profile wiki-setup run --rm wiki-setup
    else
        # Nobody at the keyboard (an automatic run): no password can be typed,
        # so a wiki that is not set up yet is left for later, not waited on forever.
        dc $(profile_args) --profile wiki-setup run --rm -T wiki-setup </dev/null \
            || { warn "the wiki setup needs you to type its admin password. Later, in a terminal: bash garden.sh wiki"; return 1; }
    fi
}

# Install step 13: one cron line that backs up every night at 2:30 am.
nightly_backup() {
    explain "Nightly backup" \
        "cron is Ubuntu's alarm clock. One line in YOUR cron runs 'garden.sh backup nightly' at 2:30 am. Nightly backups older than 7 days are deleted; ones you make yourself are kept." \
        "Backups you never have to remember. See the line: crontab -l" \
        "https://manpages.ubuntu.com/manpages/noble/man5/crontab.5.html"
    local line="30 2 * * * cd \"$HERE\" && bash garden.sh backup nightly >> backups/backup.log 2>&1"
    ( crontab -l 2>/dev/null | grep -v -e 'garden.sh backup' -e 'docker/backup.sh'; echo "$line" ) | crontab -
    note "Scheduled: $line"
}

# Install step 14: a 'Jun's Garden' icon in the app list and on the desktop.
make_icon() {
    explain "Desktop icon" "A 'Jun's Garden' icon that opens this menu." "One double-click instead of typing." ""
    local apps="$HOME/.local/share/applications" desk file
    desk=$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")
    mkdir -p "$apps"
    file="$apps/junsgarden.desktop"
    cat > "$file" <<EOF
[Desktop Entry]
Type=Application
Name=Jun's Garden
Comment=Start, stop, back up and check your garden website
Exec=bash "$HERE/garden.sh"
Icon=$HERE/site/ic_launcher_playstore.png
Terminal=true
Categories=Utility;
EOF
    chmod +x "$file"
    if [ -d "$desk" ]; then
        cp "$file" "$desk/junsgarden.desktop" && chmod +x "$desk/junsgarden.desktop"
        gio set "$desk/junsgarden.desktop" metadata::trusted true 2>/dev/null || true
        note "Desktop icon: $desk/junsgarden.desktop"
        note "If it shows a red X, right-click it and choose 'Allow Launching'."
    fi
    note "Also in the app list: press the Super (Windows) key and type: garden"
}

# The install steps, in order: "function|what it does". Every step is safe to
# run again, so after a stop the install simply carries on.
INSTALL_STEPS=(
    "check_system|Check this computer"
    "confirm_location|Confirm the folder"
    "install_packages|Ubuntu updates and tools"
    "install_ssh|SSH server"
    "firewall|Firewall"
    "install_docker|Docker"
    "make_swap|Swap (small laptops)"
    "make_env|Settings file"
    "choose_parts|Choose the optional parts"
    "make_folders|Folders"
    "git_start|Change history (git)"
    "build_and_start|Build, load your garden, start"
    "nightly_backup|Nightly backup"
    "make_icon|Desktop icon"
)
# How long each step usually takes (a normal laptop with ordinary home internet;
# an old laptop or a Raspberry Pi can take up to twice as long). Shown before
# each step, so you can decide whether there is time now: every step can be
# stopped and carried on later (menu 1 picks up where it stopped).
declare -A STEP_TIME=(
    [check_system]="under a minute"          [confirm_location]="under a minute"
    [install_packages]="3 to 10 minutes"     [install_ssh]="1 to 2 minutes"
    [firewall]="under a minute"              [install_docker]="5 to 10 minutes"
    [make_swap]="1 to 3 minutes"             [make_env]="under a minute"
    [choose_parts]="under a minute"          [make_folders]="under a minute"
    [git_start]="under a minute"
    [build_and_start]="15 to 45 minutes"
    [nightly_backup]="under a minute"        [make_icon]="under a minute"
)
# Never run: it only lists the time words above for the Korean translation
# (lang/extract_strings.py finds sentences written next to Lt).
step_time_words() { Lt "under a minute"; Lt "1 to 2 minutes"; Lt "1 to 3 minutes"; Lt "3 to 10 minutes"; Lt "5 to 10 minutes"; Lt "15 to 45 minutes"; }
PROGRESS=logs/install-progress.txt

# Writes one line per step: STARTED / DONE / FAILED with the time. If the
# install ever hangs or stops, this file shows exactly where.
progress() { mkdir -p logs; printf '%s  %-7s step %2s/%s  %s\n' "$(date '+%Y-%m-%d %H:%M:%S')" "$1" "$2" "${#INSTALL_STEPS[@]}" "$3" >> "$PROGRESS"; }

# Read logs/install-progress.txt: the number of the last step that finished.
last_done_step() {   # the number of the last step that finished, 0 if none
    [ -f "$PROGRESS" ] || { echo 0; return; }
    awk '$3=="DONE"{n=$5+0} $3=="STARTED" && /step +1\//{n=0} END{print n+0}' "$PROGRESS"
}

# Where should the garden live? Asked first, so the choice is clear.
choose_destination() {
    say "Where should your garden run?"
    Lblock <<EOF
   The garden is a small server: it should run on a computer that stays on.

   1) THIS computer ($(hostname), $(hostname -I 2>/dev/null | awk '{print $1}'))
      Everything is installed here. Later, the router sends internet
      visitors (ports 80 and 443 only) to THIS computer's address.

   2) ANOTHER computer at home: an Ubuntu laptop or a Raspberry Pi
      You need: that computer switched on, with Ubuntu (or Raspberry Pi OS)
      and the SSH server, and its local IP address (on it, type: hostname -I).
      This program copies the garden there and installs everything there.
      Later, the router sends ports 80 and 443 to THAT computer's address,
      and to no other.

   0) Cancel
EOF
    local c; c=$(ask "Type 1, 2 or 0" 1)
    case "$c" in 1) return 0 ;; 2) do_remote; return 2 ;; *) return 3 ;; esac
}

# EASY or CUSTOM. Easy: the necessary parts plus the garden tools (not the wiki),
# chosen for you, and only your name, email and passwords asked. Custom: you
# choose every part and can start at any step. Remembered in .garden-mode.
GARDEN_MODE=$(cat .garden-mode 2>/dev/null || echo easy)
choose_mode() {
    if [ "${ASSUME_YES:-0}" = 1 ] || [ -n "${GARDEN_MODE_SET:-}" ]; then return 0; fi
    Lblock <<'EOF'

   How do you want to install?

   1) EASY (recommended)
      Everything the garden needs, plus the garden tools, chosen for you.
      You only type your name, your email and your passwords. Every step
      still explains itself. Extra parts (the wiki, going online, email, NAS,
      Google Drive, n8n, subdomains) come later, only if you want them:
      menu 9 (Parts) or menu M.

   2) CUSTOM
      You choose every optional part, and you can start at any step.

   0) Cancel
EOF
    case "$(ask "Type 1, 2 or 0" 1)" in
        1) GARDEN_MODE=easy ;; 2) GARDEN_MODE=custom ;; *) return 1 ;;
    esac
    echo "$GARDEN_MODE" > .garden-mode
}

# A fingerprint of the code that gets built (website, garden tools, helpers).
# If it is the same as at the last good build, nothing needs building again.
source_fingerprint() {
    find rust beds docker -type f -not -path '*/target/*' -print0 2>/dev/null | sort -z |
        xargs -0 sha256sum 2>/dev/null | sha256sum | cut -c1-16
}

# Has the wiki got its admin yet (its setup wizard finished)? Until then the
# wiki must not be reachable from the internet: whoever finished the wizard
# first would become its admin. (The same test the wiki helper uses: /graphql
# only answers once the wizard is done.)
wiki_has_admin() {
    [ "$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 -H 'Content-Type: application/json' \
        -d '{"query":"{ __typename }"}' http://127.0.0.1:3000/graphql 2>/dev/null)" = 200 ]
}

# Has the wiki's one-time setup finished? (It leaves a note in the wiki's database.)
wiki_done() {
    # shellcheck disable=SC2016  # the password is read inside the database container
    [ "$(dk exec garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" mariadb -uroot -N -e "SELECT COUNT(*) FROM junsgarden_wiki.settings WHERE \`key\`=\"junsgardenSeed\""' 2>/dev/null)" = 1 ]
}

# Is this install step already done AND working? Then the install skips it,
# so nothing is installed over and over. Each check only LOOKS.
step_already_ok() {
    case "$1" in
        install_packages) for c in git curl openssl rsync; do command -v "$c" >/dev/null 2>&1 || return 1; done ;;
        install_ssh)      systemctl is-active --quiet ssh 2>/dev/null ;;
        firewall)         sudo -n ufw status 2>/dev/null | has 'Status: active' && sudo -n ufw status | has -E '^22' ;;
        install_docker)   dk compose version >/dev/null 2>&1 && { if [ "$(docker_mode)" = desktop ]; then dk info 2>/dev/null | has 'Docker Desktop'; else systemctl is-active --quiet docker 2>/dev/null; fi; } ;;
        choose_parts)     [ "$GARDEN_MODE" = easy ] && [ "$(env_get PARTS_CHOSEN)" = yes ] ;;
        git_start)        [ -d .git ] ;;
        build_and_start)  [ "$(cat .garden-built 2>/dev/null)" = "$(source_fingerprint)" ] &&
                              { dc up -d >/dev/null 2>&1; true; } &&
                              curl -fsS --max-time 10 "$(site_url)/healthz" >/dev/null 2>&1 &&
                              { ! has_part beds || curl -fsS --max-time 5 http://127.0.0.1:8025/healthz >/dev/null 2>&1; } &&
                              { ! has_part wiki || wiki_done; } ;;
        nightly_backup)   crontab -l 2>/dev/null | has -F "cd \"$HERE\" && bash garden.sh backup nightly" ;;
        make_icon)        grep -qF "$HERE/garden.sh" "$HOME/.local/share/applications/junsgarden.desktop" 2>/dev/null ;;
        *) return 1 ;;
    esac
}

# Menu 1: the whole install, step by step, carrying on where it stopped last time.
do_install() {
    if [ "${ASSUME_YES:-0}" != 1 ] && [ -z "${GARDEN_REMOTE_INSTALL:-}" ] && [ ! -f .env ]; then
        choose_destination; case $? in 0) ;; *) return 0 ;; esac
    fi
    start_log install
    say "Jun's Garden: install"
    choose_mode || return 0
    show_plan
    local from=1 last n=0 entry fn what pick t0 took
    last=$(last_done_step)
    say "The install steps"
    for entry in "${INSTALL_STEPS[@]}"; do
        n=$((n + 1))
        if [ "$n" -le "$last" ]; then printf '   %2d) %-34s \033[32mdone\033[0m\n' "$n" "${entry#*|}"
        else printf '   %2d) %s\n' "$n" "${entry#*|}"; fi
    done
    n=0
    [ "$last" -eq 0 ] && ask_webname_first   # a fresh install: keep the web name, or a new one?
    [ -f "$PROGRESS" ] && { note "Last lines of the progress file ($PROGRESS):"; tail -2 "$PROGRESS" | sed 's/^/      /'; }
    if [ "$last" -ge "${#INSTALL_STEPS[@]}" ]; then from=1
    else from=$((last + 1)); fi
    if [ "${ASSUME_YES:-0}" != 1 ] && [ "$GARDEN_MODE" = custom ]; then
        pick=$(ask "Start at which step? Type a number, or press Enter for step $from (0 = cancel)" "$from")
        case "$pick" in 0) return 0 ;; *[!0-9]*|"") fail "please type a step number"; return 1 ;; esac
        [ "$pick" -ge 1 ] && [ "$pick" -le "${#INSTALL_STEPS[@]}" ] || { fail "there is no step $pick"; return 1; }
        from=$pick
    fi
    for entry in "${INSTALL_STEPS[@]}"; do
        n=$((n + 1)); fn=${entry%%|*}; what=${entry#*|}
        # Steps before 'from' are skipped, except the ones that only ask or
        # check (they are quick and later steps need their answers).
        if [ "$n" -lt "$from" ] && [ "$fn" != check_system ]; then continue; fi
        say "$(Lt STEP) $n / ${#INSTALL_STEPS[@]}: $(Lt "$what")"
        # Already installed AND working? Then skip it instead of doing it again.
        if step_already_ok "$fn"; then
            note "Already installed and working: skipped."
            progress DONE "$n" "$what (already OK)"
            continue
        fi
        note "$(Lt "Usually takes"): $(Lt "${STEP_TIME[$fn]}")"
        progress STARTED "$n" "$what (usually ${STEP_TIME[$fn]})"
        t0=$SECONDS
        if "$fn"; then
            took=$(( (SECONDS - t0 + 30) / 60 ))
            note "$(Lt "This step took") $took $(Lt "min")."
            progress DONE "$n" "$what (took $took min)"
        else
            progress FAILED "$n" "$what"
            say "The install stopped at step $n: $what"
            note "Nothing is lost. Fix what the red line says, then choose menu 1 again:"
            note "it carries on from this step."
            note "Stuck? Menu H makes a help file (logs, no passwords) to show to someone."
            return 1
        fi
    done
    do_check >/dev/null 2>&1 || true
    say "ALL DONE"
    note "Your garden:   $(site_url)"
    has_part wiki && note "Your wiki:     http://localhost:3000"
    note "The checklist was saved to CHECKLIST.txt; this whole install is in $LOG"
    [ -f 00-WELCOME.html ] && note "Every guide, one click away: open 00-WELCOME.html in the garden folder (double-click it)."
    [ "${ASSUME_YES:-0}" = 1 ] && return 0
    if [ "$GARDEN_MODE" = easy ]; then
        say "Last thing: your password"
        note "One password for the website login and your sign-in page. At least 10 characters."
        do_password
        offer_email "Set up email (Gmail) now, for 'forgot password' links?"
        setup_keys_now
        say "EASY install finished"
        note "Your garden runs on this computer. When you want more, run the install again"
        note "and choose CUSTOM, or use menu M (More): putting it online, email, NAS and"
        note "Google Drive backups, n8n, subdomains, local AI."
        return 0
    fi
    say "Three short follow-ups (each can also be done later)"
    if confirm "1. Set your password now (needed to edit your garden)?" y; then do_password; fi
    setup_keys_now
    offer_email "2. Set up email (Gmail) now, for 'forgot password' links?"
    if confirm "3. Reserve this laptop's address in your router now?" n; then do_network; fi
    note "Later, when you are ready to put the garden on the internet: menu M, then 1."
}

# ---------- the front door: Caddy, the sign-in page, the web name ---------------
write_authelia() {   # write_authelia NAME
    local name=$1 dir user pw notifier duck duck_rule='' duck_cookie='' factor=one_factor
    dir=$(env_get AUTH_DIR); dir=${dir:-./local/authelia}
    # Switched to a phone code by hand (guide 07)? Then it stays that way when
    # this file is written again (front door, email, subdomains).
    grep -qs "policy: 'two_factor'" "$dir/configuration.yml" && factor=two_factor
    # With DuckDNS subdomains: one more rule and one more cookie for that name.
    duck=$(duck_root || true)
    if [ -n "$duck" ] && env_get SUBDOMAINS | has ':gate'; then
        duck_rule="    - domain: '*.$duck'
      subject: 'group:gardeners'
      policy: '$factor'"
        duck_cookie="    - domain: '$duck'
      authelia_url: 'https://auth.$duck/auth'"
    fi
    mkdir -p "$dir"
    # A restore (or the garden tools, which run as the administrator) can leave
    # this folder owned by root; then the new settings could not be written.
    # Give it back to you first. The sign-in page can still read it.
    [ -w "$dir" ] && [ -w "$dir/configuration.yml" -o ! -e "$dir/configuration.yml" ] \
        || sudo chown -R "$(id -u):$(id -g)" "$dir"
    chmod 700 "$dir"
    user=$(env_get SMTP_USER); pw=$(env_get SMTP_PASSWORD)
    if [ -n "$user" ] && [ -n "$pw" ]; then
        notifier="  smtp:
    address: 'submission://$(env_get SMTP_HOST):$(env_get SMTP_PORT)'
    username: '$user'
    password: '$pw'
    sender: \"Jun's Garden <$user>\""
    else
        notifier="  filesystem:
    filename: '/config/notification.txt'"
    fi
    ( umask 077
      cat > "$dir/configuration.yml" <<EOF
# Authelia: Jun's sign-in page.               (made by garden.sh public)
# Docs: https://www.authelia.com/configuration/prologue/introduction/
# The three secrets (jwt, session, storage key) come from .env through Docker.
server:
  # Served under https://$name/auth, the way the old site's sign-in was.
  address: 'tcp://:9091/auth'
log:
  level: 'info'
theme: 'auto'
authentication_backend:
  file:
    path: '/config/users.yml'
    watch: true          # notices a new password straight away
    password:
      algorithm: 'argon2'
access_control:
  default_policy: 'deny'
  rules:
    # The garden tools need a signed-in person. one_factor = password.
    # Change to two_factor to also ask for a phone code (guide 07); it is kept
    # when the menu writes this file again.
    # Only the group "gardeners" may (guests can sign in, but not here).
    - domain: '$name'
      resources:
        - '^/beds([/?].*)?$'
      subject: 'group:gardeners'
      policy: '$factor'
$duck_rule
session:
  cookies:
    - domain: '$name'
      authelia_url: 'https://$name/auth'
      default_redirection_url: 'https://$name/beds/plan'
$duck_cookie
regulation:
  # 5 wrong passwords in 2 minutes locks that name for 10 minutes.
  max_retries: 5
  find_time: '2m'
  ban_time: '10m'
storage:
  local:
    path: '/config/db.sqlite3'
notifier:
$notifier
EOF
    ) || { fail "could not write the sign-in page's settings in $dir"; return 1; }
    [ -f "$dir/users.yml" ] || printf 'users:\n' > "$dir/users.yml"
    chmod 600 "$dir/configuration.yml" "$dir/users.yml"
}

# ---------- subdomains with DuckDNS (advanced, optional) ------------------------------
# A free No-IP name like junsgarden.zapto.org cannot have sub-names. A free
# DuckDNS name can: wiki.NAME.duckdns.org, n8n.NAME.duckdns.org, ... all point
# to your home, and Caddy gets a padlock certificate for each one.
# SUBDOMAINS in .env lists them as  sub:port:gate|open  separated by commas.
DUCK_DIR="$HOME/.config/garden-duckdns"

duck_root() { local d; d=$(env_get DUCKDNS_DOMAIN); [ -n "$d" ] && printf '%s.duckdns.org' "$d"; }

# The Caddy blocks for every subdomain (added under the main site by write_caddyfile).
subdomain_blocks() {
    local root entry sub port mode any_gate=0
    root=$(duck_root) || return 0
    [ -n "$root" ] || return 0
    for entry in $(env_get SUBDOMAINS | tr ',' ' '); do
        IFS=: read -r sub port mode <<<"$entry"
        [ -n "$sub" ] && [ -n "$port" ] || continue
        # The wiki goes online only once it has its admin (see wiki_has_admin).
        if [ "$port" = 3000 ] && ! wiki_has_admin; then
            warn "$sub.$root is left out for now: the wiki's setup is not finished (bash garden.sh wiki)." >&2
            continue
        fi
        printf '\n# --- %s.%s -> 127.0.0.1:%s (%s) ---\n%s.%s {\n\tencode zstd gzip\n' "$sub" "$root" "$port" \
            "$( [ "$mode" = gate ] && echo 'signed-in gardeners only' || echo 'open; it has its own login')" "$sub" "$root"
        # "route" keeps the order: drop the visitor's own identity headers first,
        # then (for a gated one) ask the sign-in page, then pass the visit on.
        printf '\troute {\n\t\trequest_header -Remote-User\n\t\trequest_header -Remote-Groups\n\t\trequest_header -Remote-Email\n\t\trequest_header -Remote-Name\n\t\trequest_header -X-Auth-Email\n'
        if [ "$mode" = gate ]; then
            any_gate=1
            printf '\t\tforward_auth 127.0.0.1:9091 {\n\t\t\turi /auth/api/authz/forward-auth\n\t\t\tcopy_headers Remote-User Remote-Groups Remote-Email Remote-Name\n\t\t}\n'
        fi
        printf '\t\treverse_proxy 127.0.0.1:%s {\n\t\t\timport appproxy\n\t\t}\n\t}\n}\n' "$port"
    done
    # Gated subdomains sign in at auth.NAME.duckdns.org (same sign-in page).
    if [ "$any_gate" = 1 ]; then
        printf '\n# --- auth.%s: the sign-in page for the subdomains ---\nauth.%s {\n\treverse_proxy 127.0.0.1:9091 {\n\t\timport appproxy\n\t}\n}\n' "$root" "$root"
    fi
}

duck_update_now() {   # tell DuckDNS this home's address now; prints OK or KO
    # The token travels inside curl's settings on standard input (-K -), the way
    # DuckDNS's own install page shows, so it is never on a command line.
    printf 'url="https://www.duckdns.org/update?domains=%s&token=%s&ip="\n' \
        "$(env_get DUCKDNS_DOMAIN)" "$(cat "$DUCK_DIR/token" 2>/dev/null)" | curl -s -K -
}

do_duckdns_setup() {
    explain "DuckDNS: a free name that CAN have subdomains" \
        "DuckDNS (duckdns.org) is a free dynamic DNS service, like No-IP. Your DuckDNS name (for example junsgarden.duckdns.org) and every subdomain under it (wiki.junsgarden.duckdns.org, ...) point to your home. A small updater (one cron line, every 5 minutes) keeps it up to date." \
        "For extra parts on their own addresses: the wiki, n8n, a status page. Each gets its own padlock from Caddy. Your main site stays at junsgarden.zapto.org." \
        "https://www.duckdns.org/install.jsp  and  https://www.duckdns.org/faqs.jsp"
    Lblock <<'EOF'
   1. Open https://www.duckdns.org and sign in (Google, GitHub, ... any of them).
   2. Under "domains", type a name (for example junsgarden) and click "add domain".
      (If it is taken, choose another.)
   3. At the top of that page is your TOKEN (a long code). Copy it. Treat it like a password.
EOF
    local name tok
    name=$(ask "Your DuckDNS name (only the first part, e.g. junsgarden)" "$(env_get DUCKDNS_DOMAIN)")
    case "$name" in *[!a-z0-9-]*|"") fail "lowercase letters, numbers and - only"; return 1 ;; esac
    copy_once_note
    tok=$(ask_secret "Paste your DuckDNS token")
    [ -n "$tok" ] || { fail "the token is needed"; return 1; }
    mkdir -p "$DUCK_DIR"; chmod 700 "$DUCK_DIR"
    ( umask 077; printf '%s' "${tok//[[:space:]]/}" > "$DUCK_DIR/token" ); unset tok
    env_set DUCKDNS_DOMAIN "$name"
    note "Token saved in $DUCK_DIR/token (only you can read it)."
    if [ "$(duck_update_now)" = OK ]; then note "DuckDNS says OK: $name.duckdns.org points home now."
    else fail "DuckDNS did not accept it (KO). Check the name and the token, then try again."; return 1; fi
    # The updater: every 5 minutes, exactly as DuckDNS's install page suggests.
    cat > "$DUCK_DIR/duck.sh" <<EOF
#!/bin/sh
# Jun's Garden: tell DuckDNS this home's address. Runs every 5 minutes (cron).
# Docs: https://www.duckdns.org/install.jsp
printf 'url="https://www.duckdns.org/update?domains=%s&token=%s&ip="\n' "$name" "\$(cat "$DUCK_DIR/token")" | curl -s -K - -o "$DUCK_DIR/duck.log"
EOF
    chmod 700 "$DUCK_DIR/duck.sh"
    ( crontab -l 2>/dev/null | grep -v garden-duckdns; echo "*/5 * * * * $DUCK_DIR/duck.sh >/dev/null 2>&1" ) | crontab -
    note "Updater scheduled every 5 minutes (see: crontab -l). Its last answer is in $DUCK_DIR/duck.log"
}

apply_subdomains() {
    [ -n "$(env_get PUBLIC_NAME)" ] || { fail "do the front door first (menu M, then 1)"; return 1; }
    say "Writing Caddy's settings and the sign-in page's rules for your subdomains"
    write_authelia "$(env_get PUBLIC_NAME)"
    reload_signin
    sudo cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak
    write_caddyfile "$(env_get PUBLIC_NAME)" "$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)" \
        "$(env_get WIKI_NAME)" "$(case "$(env_get PUBLIC_NAME)" in *.test) echo 1 ;; *) echo 0 ;; esac)" | sudo tee /etc/caddy/Caddyfile >/dev/null
    sudo caddy fmt --overwrite /etc/caddy/Caddyfile >/dev/null 2>&1 || true
    local out
    if ! out=$(sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile 2>&1); then
        echo "$out" | grep -iE "error" | tail -5
        sudo cp /etc/caddy/Caddyfile.bak /etc/caddy/Caddyfile
        fail "that did not work, so the old settings were put back"; return 1
    fi
    run sudo systemctl reload-or-restart caddy
    note "Done. Each new address gets its padlock on the first visit (give it a minute)."
    do_addresses
}

do_subdomains() {
    local c sub port mode list
    while true; do
        say "Subdomains (DuckDNS)"
        note "DuckDNS name: $(duck_root || echo 'not set up yet')"
        note "Subdomains:   $(env_get SUBDOMAINS | tr ',' ' ')"
        Lblock <<'EOF'
   1) Set up DuckDNS (your name, token and the 5-minute updater)
   2) Add a subdomain
   3) Remove a subdomain
   4) Write the settings again (Caddy + sign-in page) and reload
   0) Back
EOF
        c=$(ask "Number" 0)
        case "$c" in
            0) return 0 ;;
            1) do_duckdns_setup ;;
            2) [ -n "$(duck_root)" ] || { note "Set up DuckDNS first (1)."; continue; }
               Lblock <<'EOF'
   Which program? (the port is filled in for you)
     wiki   -> your wiki (port 3000; has its own login, so it can be open)
     n8n    -> n8n automations (port 5678; put it behind the sign-in page)
     status -> Uptime Kuma, from Project 1 (port 3001; behind the sign-in page)
     home   -> Home Assistant, from Project 3 (port 8123; behind the sign-in page too)
     or any name, then type its port yourself
EOF
               sub=$(ask "Subdomain name (letters, numbers, -)" "wiki")
               case "$sub" in *[!a-z0-9-]*|"") note "lowercase letters, numbers and - only"; continue ;; esac
               case "$sub" in wiki) port=3000; mode=open ;; n8n) port=5678; mode=gate ;; status) port=3001; mode=gate ;; home) port=8123; mode=gate ;; *) port=""; mode=gate ;; esac
               port=$(ask "Port of the program on this computer" "$port")
               case "$port" in *[!0-9]*|"") note "the port is a number"; continue ;; esac
               if confirm "Put $sub behind your sign-in page (recommended unless it has its own login)?" "$( [ "$mode" = gate ] && echo y || echo n )"; then mode=gate; else mode=open; fi
               list=$(env_get SUBDOMAINS | tr ',' '\n' | grep -v "^$sub:" | paste -sd, -)
               env_set SUBDOMAINS "${list:+$list,}$sub:$port:$mode"
               [ "$sub" = wiki ] && { env_set WIKI_SITE_URL "https://wiki.$(duck_root)"; note "Wiki address set to https://wiki.$(duck_root)"; }
               apply_subdomains ;;
            3) sub=$(ask "Subdomain to remove" "")
               env_set SUBDOMAINS "$(env_get SUBDOMAINS | tr ',' '\n' | grep -v "^$sub:" | paste -sd, -)"
               apply_subdomains ;;
            4) apply_subdomains ;;
            *) note "Type one of the numbers." ;;
        esac
    done
}

# ---------- AI cost control: LiteLLM (advanced, optional) ------------------------------
# LiteLLM is an open-source "AI gateway": one private address on this computer
# (http://localhost:4000) that your OWN projects (n8n, coding tools, scripts)
# call instead of calling Google or OpenRouter directly. It holds the keys,
# so your projects never see them, and it stops at a spending limit you set.
# The garden website and garden tools keep using their own free keys directly.
do_llm() {
    explain "LiteLLM: one AI address with a spending limit" \
        "An open-source program (by BerriAI) that sits between your own projects and the AI services. It uses the keys you saved with menu K, and refuses requests once the monthly limit you choose is reached." \
        "If you ever pay for AI, this keeps the cost under control and in one place. With free keys only, you do not need it." \
        "https://docs.litellm.ai/docs/simple_proxy  and  https://docs.litellm.ai/docs/proxy/users"
    if has_part llm; then llm_manage; return; fi
    confirm "Switch LiteLLM on?" y || return 0
    local budget dir
    budget=$(ask "Monthly spending limit in US dollars (0 = free models only)" "$(env_get LITELLM_BUDGET || echo 5)")
    case "$budget" in *[!0-9.]*|"") fail "type a number, like 5"; return 1 ;; esac
    env_set LITELLM_BUDGET "$budget"
    [ -n "$(env_get LITELLM_MASTER_KEY)" ] || env_set LITELLM_MASTER_KEY "sk-$(openssl rand -hex 24)"
    dir=./local/litellm; mkdir -p "$dir"
    # The settings file. Keys are NOT written here: LiteLLM reads them from
    # .env through Docker ("os.environ/..."), so this file is safe to read.
    cat > "$dir/config.yaml" <<EOF
# LiteLLM settings for Jun's Garden (made by garden.sh, menu M, 15).
# Docs: https://docs.litellm.ai/docs/proxy/configs
model_list:
  - model_name: gemini-flash          # the name your projects ask for
    litellm_params:
      model: gemini/gemini-2.5-flash
      api_key: os.environ/GEMINI_API_KEY
  - model_name: free-openrouter
    litellm_params:
      model: openrouter/meta-llama/llama-3.3-70b-instruct:free
      api_key: os.environ/OPENROUTER_API_KEY
litellm_settings:
  max_budget: $budget        # US dollars, then requests are refused
  budget_duration: 30d       # the limit starts again every 30 days
EOF
    # Port 4000 may already be used by another program: then take the next free one.
    local port=${LITELLM_PORT_IN:-$(env_get LITELLM_PORT)}; port=${port:-4000}
    while [ -n "$(ss -ltnH "( sport = :$port )" 2>/dev/null)" ] && ! dk ps --format '{{.Names}}' | has -x garden-llm; do
        note "Port $port is already used by another program; trying $((port + 1))."
        port=$((port + 1))
    done
    env_set LITELLM_PORT "$port"
    part_on llm
    run dc up -d litellm || { fail "LiteLLM did not start: docker logs garden-llm"; return 1; }
    note "LiteLLM runs at http://localhost:$port (this computer only)."
    note "Your projects use it like OpenAI: base URL http://localhost:$port, model gemini-flash,"
    note "and the key: menu M, 15, then 6 shows it (see guide 16)."
    do_addresses >/dev/null
}

# LiteLLM is on: manage its models, limit and key. Every change rewrites
# local/litellm/config.yaml (no keys in it) and restarts LiteLLM.
LLM_CONF=./local/litellm/config.yaml
llm_manage() {
    local c name model base
    while true; do
        say "LiteLLM: your AI gateway at http://localhost:$(env_get LITELLM_PORT) (limit \$$(env_get LITELLM_BUDGET) per 30 days)"
        note "Models your projects can ask for:"
        grep -E '^  - model_name:' "$LLM_CONF" | sed -E 's/^  - model_name: *([^ #]+).*/      \1/'
        Lblock <<'EOF'
   1) Add a Gemini model (uses your Gemini key, menu K)
   2) Add an OpenRouter model (uses your OpenRouter key; ":free" models cost nothing)
   3) Add a local model from Ollama (your own AI computer, guide 15)
   4) Remove a model
   5) Change the monthly spending limit
   6) Show the gateway key (to paste into one of your projects)
   7) Switch LiteLLM off
   0) Back
EOF
        c=$(ask "Number" 0)
        case "$c" in
            0) return 0 ;;
            1) note "Model names: https://ai.google.dev/gemini-api/docs/models"
               model=$(ask "Gemini model id" "gemini-2.5-flash"); name=$(ask "Short name your projects will use" "${model//./-}")
               llm_add "$name" "gemini/$model" "os.environ/GEMINI_API_KEY" "" ;;
            2) note "Model ids: https://openrouter.ai/models (filter: free)"
               model=$(ask "OpenRouter model id" "meta-llama/llama-3.3-70b-instruct:free"); name=$(ask "Short name" "openrouter-${model##*/}")
               llm_add "$name" "openrouter/$model" "os.environ/OPENROUTER_API_KEY" "" ;;
            3) base=$(ask "Address of your AI computer (Ollama), e.g. http://192.168.1.60:11434" "$(env_get OLLAMA_URL | sed 's#/api/generate##')")
               model=$(ask "Ollama model name (on that computer: ollama list)" "qwen2.5-coder:7b"); name=$(ask "Short name" "local-${model%%:*}")
               llm_add "$name" "ollama_chat/$model" "" "$base" ;;
            4) name=$(ask "Short name of the model to remove" "")
               awk -v n="  - model_name: $name" 'BEGIN{s=0} index($0,n)==1{s=1; next} s && /^  - model_name:|^[a-z_]+:/{s=0} !s' "$LLM_CONF" > "$LLM_CONF.new" && mv "$LLM_CONF.new" "$LLM_CONF"
               run dc restart litellm >/dev/null; note "Removed $name." ;;
            5) local b; b=$(ask "New monthly limit in US dollars" "$(env_get LITELLM_BUDGET)")
               case "$b" in *[!0-9.]*|"") note "Type a number."; continue ;; esac
               env_set LITELLM_BUDGET "$b"; sed -i -E "s/^  max_budget: [0-9.]+/  max_budget: $b/" "$LLM_CONF"
               run dc restart litellm >/dev/null; note "Limit is now \$$b per 30 days." ;;
            6) note "Gateway key (treat it like a password; it only works on this computer):"
               printf '      %s\n' "$(env_get LITELLM_MASTER_KEY)" ;;
            7) confirm "Switch LiteLLM off? (its settings are kept)" n && { part_off llm; run dc rm -sf litellm >/dev/null; note "Off."; return 0; } ;;
            *) note "Type one of the numbers." ;;
        esac
    done
}
llm_add() {   # llm_add NAME MODEL KEYREF APIBASE : add one model to the settings
    local name=$1 model=$2 key=$3 base=$4 tmp
    case "$name" in *[!A-Za-z0-9._-]*|"") note "Short name: letters, numbers, - . _ only."; return 1 ;; esac
    tmp=$(mktemp)
    # Put the new model at the end of model_list (just before litellm_settings).
    awk -v n="$name" -v m="$model" -v k="$key" -v b="$base" '
        /^litellm_settings:/ {
            print "  - model_name: " n; print "    litellm_params:"; print "      model: " m
            if (k != "") print "      api_key: " k
            if (b != "") print "      api_base: " b
        }
        { print }' "$LLM_CONF" > "$tmp" && cat "$tmp" > "$LLM_CONF"; rm -f "$tmp"
    run dc restart litellm >/dev/null && note "Added $name. Your projects ask for model \"$name\"."
}

# ---------- my addresses: every web address and what it is ---------------------------
# Written to MY-ADDRESSES.txt every time the front door or subdomains change,
# and shown with menu A. So you always know which address does what, and
# which ones need the sign-in page.
do_addresses() {
    local name duck entry sub port mode out=MY-ADDRESSES.txt
    name=$(env_get PUBLIC_NAME); duck=$(duck_root || true)
    {
        echo "JUN'S GARDEN: MY ADDRESSES   (made $(date '+%Y-%m-%d %H:%M'); menu A shows it again)"
        echo
        if [ -n "$name" ]; then
            echo "ON THE INTERNET (through Caddy, the front door; router forwards 80 and 443 only)"
            echo "  https://$name/                 the garden website (anyone)"
            echo "  https://$name/login.php        log in to edit (your website login)"
            echo "  https://$name/streamlit/       the plant admin (your website login)"
            echo "  https://$name/apps/junsgarden.apk   the phone app download"
            echo "  https://$name/auth/            YOUR SIGN-IN PAGE (Authelia)"
            echo "  https://$name/beds/plan        garden tools: bed planner   (sign-in page, gardeners only)"
            echo "  https://$name/beds/checkup     garden tools: plant checkup (sign-in page)"
            echo "  https://$name/beds/diagnose    garden tools: diagnose      (sign-in page)"
            echo "  https://$name/beds/tag         garden tools: tag scan      (sign-in page)"
            echo "  https://$name/beds/spend       garden tools: spending      (sign-in page)"
            echo "  https://$name/beds/wiki-queue  garden tools: wiki drafts   (sign-in page)"
            echo "  Short ways in: /checkup /spend /diagnose /tagscan /wiki-queue /beds  (they jump to the ones above)"
            [ -n "$(env_get WIKI_NAME)" ] && echo "  https://$(env_get WIKI_NAME)/   your wiki (its own login for editing)"
            if [ -n "$duck" ]; then
                for entry in $(env_get SUBDOMAINS | tr ',' ' '); do
                    IFS=: read -r sub port mode <<<"$entry"
                    printf '  https://%s.%s/   -> port %s  (%s)\n' "$sub" "$duck" "$port" \
                        "$( [ "$mode" = gate ] && echo "sign-in page at https://auth.$duck/auth/, gardeners only" || echo 'open; its own login')"
                done
            fi
            echo "  Caddy's settings file: /etc/caddy/Caddyfile   (made from docker/caddy/Caddyfile.template)"
        elif [ "$(env_get HOME_ONLY)" = yes ]; then
            echo "HOME ONLY (devices in your house; not on the internet)"
            echo "  http://$(hostname).local/   and   http://$(hostname -I 2>/dev/null | awk '{print $1}')/   the garden website"
            echo "  (the garden tools /beds open once the garden is online: menu M, 1, choice 3)"
        else
            echo "NOT ON THE INTERNET YET (the front door is menu M, then 1)"
        fi
        echo
        echo "ON THIS COMPUTER ONLY (127.0.0.1: nobody else can reach these)"
        echo "  http://localhost:$(site_port)/      the website"
        has_part wiki && echo "  http://localhost:3000/      the wiki"
        has_part beds && echo "  127.0.0.1:8025              garden tools (use them through /beds on the website)"
        has_part gate && echo "  127.0.0.1:9091              the sign-in page (use it through /auth)"
        has_part n8n  && echo "  http://localhost:5678/      n8n"
        has_part llm  && echo "  http://localhost:$(env_get LITELLM_PORT)/      LiteLLM AI gateway"
        echo "  http://localhost:8082/      Adminer, only while it runs (menu M, 8)"
        echo
        echo "From another computer at home, use an SSH tunnel (the Windows menu does it: menu 4)."
    } > "$out"
    cat "$out"
    note "Saved in $HERE/$out"
}

# HOME ONLY: Caddy as a system service, plain http, for devices in the house.
# No internet, no router change, no sign-in page (the garden tools stay private).
do_home_only() {
    local host ip net
    host="$(hostname).local"; ip=$(hostname -I 2>/dev/null | awk '{print $1}'); net=$(home_subnet)
    explain "Not on the internet yet: your home Wi-Fi only" \
        "Caddy runs as a system service (it starts by itself with the laptop) and shows the garden at http://$host and http://$ip to phones and computers on your home network. '.local' names work through Avahi, which Ubuntu already has." \
        "To use the garden at home before your No-IP name is ready. Nothing is opened to the internet; the firewall lets port 80 in from your home network only ($net)." \
        "https://caddyserver.com/docs/caddyfile/concepts#addresses"
    # Port 80 must be free (CADDY_HTTP_PORT in .env changes it; normally empty).
    local hp who; hp=$(env_get CADDY_HTTP_PORT || true); hp=${hp:-80}
    who=$(sudo ss -ltnpH "( sport = :$hp )" 2>/dev/null | grep -o 'users:(("[^"]*' | cut -d'"' -f2 | sort -u | tr '\n' ' ')
    if [ -n "$(sudo ss -ltnH "( sport = :$hp )" 2>/dev/null)" ] && [ "${who% }" != caddy ]; then
        fail "port $hp is already used by: ${who:-another program}. Only one program can use it."; return 1
    fi
    install_caddy || return 1
    [ -f /etc/caddy/Caddyfile ] && run sudo cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak
    sudo tee /etc/caddy/Caddyfile >/dev/null <<EOF
# Jun's Garden: HOME ONLY (made by garden.sh, front door, choice 1).
# Plain http for devices in the house. Not reachable from the internet.
# The garden tools (/beds) need the sign-in page, so they are not shown here.
{
	http_port $hp
}
http://$host, http://$ip, http://localhost {
	encode zstd gzip
	@private path /beds* /auth*
	# Its own handle block: Caddy runs "handle" before "respond", so a bare
	# respond here would never be reached.
	handle @private {
		respond "The garden tools open once the garden is online (front door, choice 3)." 403
	}
	handle {
		request_header -X-Auth-Email
		reverse_proxy 127.0.0.1:$(site_port)
	}
}
EOF
    local out
    if ! out=$(sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile 2>&1); then
        echo "$out" | grep -i error | tail -3; fail "the settings did not validate"; return 1; fi
    run sudo systemctl enable caddy >/dev/null 2>&1
    run sudo systemctl reload-or-restart caddy || { fail "Caddy did not start: sudo journalctl -u caddy -n 20"; return 1; }
    if [ -n "$net" ]; then run sudo ufw allow from "$net" to any port "$hp" proto tcp comment 'garden, home only' >/dev/null; fi
    env_set HOME_ONLY yes
    local suffix=""; [ "$hp" = 80 ] || suffix=":$hp"
    note "Open it on a phone or computer at home:  http://$host$suffix   or   http://$ip$suffix"
    note "Next step, to put it on the internet (www):"
    note "  1. Sign up at https://www.noip.com for your web name and its DDNS key (guide 04, Part A)."
    note "  2. Optional: a DuckDNS name for extra sub-names (guide 04, Advanced)."
    note "  3. Menu M, then 1, then choose 3. That also installs the No-IP updater (DUC),"
    note "     which keeps your web name pointing at this computer."
    do_addresses >/dev/null
}

# Take the garden off the internet again (it keeps running at home).
do_public_off() {
    explain "Turn the front door off" \
        "Stops Caddy (so nothing answers on ports 80 and 443), closes those two doors in the firewall, stops the No-IP and DuckDNS updaters, and switches the sign-in page off." \
        "To take the garden off the internet for a while, or before moving it. Your garden keeps running at home; menu M, 1 puts it back online." ""
    confirm "Take the garden off the internet now?" n || return 0
    run sudo systemctl disable --now caddy 2>/dev/null
    sudo ufw delete allow 80/tcp >/dev/null 2>&1; sudo ufw delete allow 443/tcp >/dev/null 2>&1
    sudo systemctl disable --now noip-duc >/dev/null 2>&1
    ( crontab -l 2>/dev/null | grep -v garden-duckdns ) | crontab -
    part_off gate; dc rm -sf authelia >/dev/null 2>&1
    local hp; hp=$(env_get CADDY_HTTP_PORT || true)
    env_set HOME_ONLY ""; sudo ufw delete allow from "$(home_subnet)" to any port "${hp:-80}" proto tcp >/dev/null 2>&1
    env_set PUBLIC_URL "http://localhost:$(site_port)"; env_set PUBLIC_NAME ""
    run dc up -d >/dev/null
    note "Off the internet. Remember: in the router you can also remove the 80 and 443 forwarding."
    do_addresses >/dev/null
}

# Fill in docker/caddy/Caddyfile.template with your web name (and wiki name).
write_caddyfile() {   # write_caddyfile NAME EMAIL WIKINAME PRACTICE(0/1)
    local name=$1 email=$2 wiki=$3 practice=$4 local_certs="" wiki_block=""
    [ "$practice" = 1 ] && local_certs=$'\t# PRACTICE MODE: certificates from Caddy itself, not Let\'s Encrypt.\n\tlocal_certs\n'
    # Different ports (only if your internet company blocks 80/443, or for testing):
    # set CADDY_HTTP_PORT and CADDY_HTTPS_PORT in .env. Normally they stay empty.
    local hp sp
    hp=$(env_get CADDY_HTTP_PORT || true); sp=$(env_get CADDY_HTTPS_PORT || true)
    [ -n "$hp" ] && local_certs+=$(printf '\thttp_port %s\n' "$hp")$'\n'
    [ -n "$sp" ] && local_certs+=$(printf '\thttps_port %s\n' "$sp")$'\n'
    if [ -n "$wiki" ] && ! wiki_has_admin; then
        # (On stderr: stdout of this function is the Caddyfile itself.)
        warn "the wiki is not online yet: its setup is not finished (bash garden.sh wiki), then run this step again." >&2
        wiki=""
    fi
    if [ -n "$wiki" ]; then
        wiki_block="# --- The wiki, on its own web name (Wiki.js cannot live under a path) ---
$wiki {
	encode zstd gzip
	reverse_proxy 127.0.0.1:3000 {
		import appproxy
	}
}"
    fi
    awk -v n="$name" -v e="$email" -v lc="$local_certs" -v wb="$wiki_block" '
        { gsub(/__NAME__/, n); gsub(/__EMAIL__/, e) }
        /^__LOCAL_CERTS__/ { sub(/^__LOCAL_CERTS__/, ""); printf "%s", lc }
        /^__WIKI_BLOCK__$/ { print wb; next }
        { print }' docker/caddy/Caddyfile.template
    subdomain_blocks
}

# Install Caddy (the HTTPS front door) from its official Ubuntu source.
install_caddy() {
    explain "Caddy (the front door)" \
        "A web server that answers on ports 80 and 443, gets the HTTPS padlock certificate by itself (from Let's Encrypt), and passes each visit to the right program." \
        "One door for everything, with the same gates as the old site: the sign-in page in front of the garden tools, fake identity headers removed." \
        "https://caddyserver.com/docs/install#debian-ubuntu-raspbian"
    if command -v caddy >/dev/null 2>&1; then note "Caddy is already installed: $(caddy version | cut -d' ' -f1)"; return 0; fi
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq debian-keyring debian-archive-keyring apt-transport-https gpg >/dev/null
    note "Adding Caddy's signing key and official download source (dl.cloudsmith.io/public/caddy):"
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor --yes -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
    curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list >/dev/null
    run sudo apt-get -o DPkg::Lock::Timeout=900 update -qq
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq caddy >/dev/null || { fail "Caddy did not install"; return 1; }
}

# Install the No-IP updater and save your DDNS key privately in /etc/default/noip-duc.
install_noip() {
    explain "No-IP updater (DUC)" \
        "Your home internet address can change. This small program tells No-IP your current address every few minutes, so your web name always points home." \
        "Without it the name stops working the day your internet company changes your address." \
        "https://www.noip.com/support/knowledgebase/install-linux-3-x-dynamic-update-client-duc"
    note "You need YOUR No-IP account, with your web name in it, and a DDNS KEY for it"
    note "(guide 04, Part A: free account at noip.com -> Dynamic DNS -> add the name -> DDNS Keys)."
    note "The key can only update your names. It is saved in /etc/default/noip-duc, readable only by the administrator."
    note "Free No-IP names must be confirmed every 30 days: No-IP emails you a link."
    if systemctl list-unit-files 2>/dev/null | has '^noip-duc.service'; then
        note "The No-IP updater is already installed."
        if ! confirm "Enter a new DDNS key anyway?" n; then
            # Keep the key, but make sure the updater runs (menu M, 14, then 3 switches it off).
            run sudo systemctl enable --now noip-duc >/dev/null 2>&1 || warn "the No-IP updater did not start: sudo systemctl status noip-duc"
            return 0
        fi
    else
        local work deb
        work=$(mktemp -d)
        run wget --content-disposition -q -P "$work" https://www.noip.com/download/linux/latest || { fail "download from noip.com failed"; return 1; }
        tar xf "$work"/noip-duc_*.tar.gz -C "$work"
        deb=$(find "$work" -name "noip-duc_*_$(dpkg --print-architecture).deb" | head -1)
        [ -n "$deb" ] || { fail "no No-IP package for this processor"; return 1; }
        run sudo apt-get -o DPkg::Lock::Timeout=900 install -y -qq "$deb" >/dev/null || { fail "No-IP updater did not install"; return 1; }
        rm -rf "$work"
    fi
    local u p
    read -r -p "DDNS Key username: " u
    copy_once_note
    read -r -s -p "DDNS Key password (hidden): " p; echo
    [ -n "$u" ] && [ -n "$p" ] || { fail "both are needed"; return 1; }
    ( umask 077; printf 'NOIP_USERNAME=%s\nNOIP_PASSWORD=%s\nNOIP_HOSTNAMES=all.ddnskey.com\n' "$u" "$p" | sudo tee /etc/default/noip-duc >/dev/null )
    unset u p
    run sudo chmod 600 /etc/default/noip-duc
    run sudo systemctl enable --now noip-duc && run sudo systemctl restart noip-duc
    sudo systemctl --no-pager status noip-duc | head -5
}

# Menu M,1: the front door: Caddy, the sign-in page, firewall 80/443, No-IP, router steps.
do_public() {
    start_log public
    say "The front door: HTTPS, your sign-in page and your web name"
    Lblock <<'EOF'
   Three ways, from smallest to biggest:

   1) NOT ON THE INTERNET YET (your home Wi-Fi only, no www)
      Phones and computers on your home Wi-Fi open the garden by this laptop's
      name. Nobody outside your house can reach it; the router is not touched.
      Next step, to put it on the internet: sign up at No-IP for your web name
      (guide 04, Part A); a DuckDNS name is optional (extra sub-names). Then
      come back here and choose 3.
   2) PRACTICE
      Everything (HTTPS, your sign-in page, the gates) on this laptop only,
      with a practice certificate and a made-up name ending in .test.
   3) THE REAL WEB NAME (junsgarden.zapto.org, or your own)
      Needs your own No-IP account with that name (guide 04, Part A). Opens
      ports 80 and 443 in the firewall; then YOU forward those two ports in
      the router to this laptop. They are the only ports ever opened.
   0) Back
EOF
    local name email wiki practice=0 way
    if [ -n "${PUBLIC_NAME_IN:-}" ]; then way=3; else way=$(ask "Type 1, 2, 3 or 0" 1); fi
    # Choices 1 and 3 need other devices (or the internet) to reach this computer.
    # Windows' own Ubuntu (WSL) has a private inside address they cannot reach.
    if { [ "$way" = 1 ] || [ "$way" = 3 ]; } && grep -qi microsoft /proc/version 2>/dev/null; then
        warn "this is Windows' own Ubuntu (WSL): other devices and the internet cannot reach it this way."
        note "For this choice, use a separate garden computer (an Ubuntu laptop or a Pi), guide 00b."
        note "On this PC, choice 2 (practice) works. More: https://learn.microsoft.com/windows/wsl/networking"
        confirm "Carry on anyway?" n || return 0
    fi
    case "$way" in
        1) do_home_only; return ;;
        2) name=${PUBLIC_NAME_IN:-$(ask "Practice name (ends in .test)" "garden.test")} ;;
        3) name=${PUBLIC_NAME_IN:-$(ask "Web name" "$(env_get PUBLIC_NAME || echo junsgarden.zapto.org)")} ;;
        *) return 0 ;;
    esac
    case "$name" in *[!A-Za-z0-9.-]*|.*|*..*|"") fail "that does not look like a web name"; return 1 ;; esac
    case "$name" in *.test) practice=1 ;; esac
    if [ "$practice" = 0 ]; then
        note "Before going on: $name must be in YOUR No-IP account (or point at this home),"
        note "and you need its DDNS key. Not done yet? Choose 0 now, do guide 04 Part A, come back."
        confirm "Is $name in your own No-IP account?" y || return 0
    fi
    email=$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)
    wiki=""
    if has_part wiki; then
        note "The wiki needs its OWN web name (for example a free DuckDNS name). Leave empty to keep it private."
        wiki=${WIKI_NAME_IN-$(ask "Wiki web name (empty = private only)" "$(env_get WIKI_NAME || true)")}
    fi
    env_set PUBLIC_NAME "$name"; env_set WIKI_NAME "$wiki"; env_set PUBLIC_URL "https://$name"; env_set HOME_ONLY ""

    # Ports 80 and 443 must be free for Caddy (another web server may hold them).
    # (CADDY_HTTP_PORT / CADDY_HTTPS_PORT in .env change them; normally empty.)
    local p who hp sp
    hp=$(env_get CADDY_HTTP_PORT || true); sp=$(env_get CADDY_HTTPS_PORT || true)
    for p in "${hp:-80}" "${sp:-443}"; do
        who=$(sudo ss -ltnpH "( sport = :$p )" 2>/dev/null | grep -o 'users:(("[^"]*' | cut -d'"' -f2 | sort -u | tr '\n' ' ')
        if [ -n "$(sudo ss -ltnH "( sport = :$p )" 2>/dev/null)" ] && [ "${who% }" != caddy ]; then
            fail "port $p is already used by: ${who:-another program}. Only one program can use it."
            note "If it is Apache:  sudo systemctl disable --now apache2   (the learning Apache uses 8081 instead)."
            note "Then run this step again."
            return 1
        fi
    done

    install_caddy || return 1

    explain "Your sign-in page (Authelia)" \
        "A small program that IS your login page, at https://$name/auth. Caddy asks it 'is this person signed in?' before opening the garden tools. Your password is stored only as a scrambled hash." \
        "The same job the old site's sign-in did. Wrong passwords 5 times in 2 minutes locks the name for 10 minutes." \
        "https://www.authelia.com/integration/proxies/caddy/"
    write_authelia "$name"
    part_on gate
    run dc up -d || { fail "something did not start:  bash garden.sh logs"; return 1; }
    wait_signin

    explain "Writing Caddy's settings (/etc/caddy/Caddyfile)" \
        "Made from docker/caddy/Caddyfile.template: website at /, sign-in page at /auth, garden tools at /beds behind the sign-in$( [ -n "$wiki" ] && echo ", wiki at $wiki")." \
        "You can read it any time: cat /etc/caddy/Caddyfile. The old one is kept as Caddyfile.bak." \
        "https://caddyserver.com/docs/caddyfile"
    [ -f /etc/caddy/Caddyfile ] && run sudo cp /etc/caddy/Caddyfile /etc/caddy/Caddyfile.bak
    write_caddyfile "$name" "$email" "$wiki" "$practice" | sudo tee /etc/caddy/Caddyfile >/dev/null
    run sudo caddy fmt --overwrite /etc/caddy/Caddyfile >/dev/null 2>&1 || true
    note "Checking the settings file (caddy validate)..."
    local out
    if out=$(sudo caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile 2>&1); then note "The settings are valid."
    else echo "$out" | grep -E "error|Error" | tail -5; fail "the Caddyfile has a mistake (the line is named above)"; return 1; fi
    run sudo systemctl enable caddy >/dev/null 2>&1
    run sudo systemctl reload-or-restart caddy || { fail "Caddy did not start: sudo journalctl -u caddy -n 20"; return 1; }

    explain "Firewall: ports 80 and 443" \
        "Allows web visits in. 80 is plain web (Caddy only uses it to move people to HTTPS and to get certificates); 443 is HTTPS." \
        "These two and SSH-from-home are the only open doors." "https://ubuntu.com/server/docs/how-to/security/firewalls/"
    run sudo ufw allow 80/tcp comment 'Caddy http' >/dev/null
    run sudo ufw allow 443/tcp comment 'Caddy https' >/dev/null

    if [ "$practice" = 1 ]; then
        note "Practice mode: pointing $name at this laptop in /etc/hosts (this laptop only)."
        grep -q " $name\$" /etc/hosts || echo "127.0.0.1 $name" | sudo tee -a /etc/hosts >/dev/null
        [ -n "$wiki" ] && { grep -q " $wiki\$" /etc/hosts || echo "127.0.0.1 $wiki" | sudo tee -a /etc/hosts >/dev/null; }
        note "Open https://$name in Firefox on this laptop. It warns about the practice certificate: that is expected."
    else
        install_noip || warn "the No-IP updater is not set up; run this step again"
        do_network
        Lblock <<EOF

   LAST STEP, IN YOUR ROUTER (only you can do this):
   Port forwarding: send ONLY these two ports to this laptop ($(env_get LAPTOP_IP || true)):
       TCP 80  -> $(env_get LAPTOP_IP || echo 'this laptop'):80
       TCP 443 -> $(env_get LAPTOP_IP || echo 'this laptop'):443
   Nothing else. Then test from a phone with Wi-Fi OFF: https://$name
   Guide: START-HERE/04-go-public-no-ip-caddy.md
EOF
    fi
    do_addresses >/dev/null; note "Every address is listed in MY-ADDRESSES.txt (menu A)."
    say "Set the password for your sign-in page (the same one as the website)"
    do_password "$(env_get GARDEN_USERS | cut -d= -f1)"
}

# ---------- remote: install on another computer over SSH ------------------------
do_remote() {
    say "Install Jun's Garden on ANOTHER computer, over SSH"
    note "That computer needs: Ubuntu, the internet, and the SSH server."
    note "If you are not sure it has the SSH server, type this ON that computer once:"
    note "    sudo apt install -y openssh-server"
    local ip user last_ip="" last_user="$USER" key="$HOME/.ssh/id_ed25519" backup
    # shellcheck disable=SC1090
    [ -f "$SETTINGS" ] && . "./$SETTINGS"
    ip=${REMOTE_IP:-$(ask "IP address of the other computer (example 192.168.1.50)" "$last_ip")}
    user=${REMOTE_USER:-$(ask "Your user name on that computer" "$last_user")}
    [ -n "$ip" ] && [ -n "$user" ] || { fail "need both the address and the user name"; return 1; }
    printf 'last_ip=%q\nlast_user=%q\n' "$ip" "$user" > "$SETTINGS"

    say "1. Can we reach $ip on the SSH port (22)?"
    timeout 5 bash -c "</dev/tcp/$ip/22" 2>/dev/null ||
        { fail "no answer from $ip port 22. Check the address (on that computer: hostname -I) and that openssh-server is installed."; return 1; }
    note "yes"

    explain "An SSH key" \
        "A pair of files: a private key that stays on THIS computer (~/.ssh/id_ed25519) and a public key copied to the other one. Like a house key and its lock." \
        "You type the other computer's password once; after that the key opens it." \
        "https://www.ssh.com/academy/ssh/keygen"
    [ -f "$key" ] || run ssh-keygen -q -t ed25519 -N "" -f "$key" -C "garden-$(hostname)"
    note "Type your password for $user@$ip when asked (it is not shown)."
    run ssh-copy-id -o StrictHostKeyChecking=accept-new -i "$key.pub" "$user@$ip" >/dev/null ||
        { fail "the key was not accepted (wrong password or user name?)"; return 1; }

    say "3. Copying the garden folder (not your .env or private data)"
    # shellcheck disable=SC2029  # REMOTE_DIR is meant to expand on this computer
    tar -czf - --exclude=./local --exclude=./.env --exclude=./.git --exclude=./backups --exclude=./logs \
        --exclude="./$SETTINGS" --exclude='./CHECKLIST*.txt' --exclude=./checklist-history.log . |
        ssh "$user@$ip" "mkdir -p ~/$REMOTE_DIR/backups && tar -xzf - -C ~/$REMOTE_DIR" ||
        { fail "copying failed"; return 1; }
    backup=$(newest_backup)
    if [ -n "$backup" ]; then
        note "Copying your newest backup: $backup"
        run scp -q "$(env_get BACKUP_DIR)/$backup" "$user@$ip:$REMOTE_DIR/backups/" || warn "the backup did not copy"
    fi

    say "4. Running the install ON $ip (it asks for that computer's password once, for sudo)"
    ssh -t "$user@$ip" "cd ~/$REMOTE_DIR && GARDEN_REMOTE_INSTALL=1 bash garden.sh install" || { fail "the install on $ip stopped. Read the lines above."; return 1; }

    say "Finished. To see the garden from THIS computer, run:"
    note "ssh -N -L 8030:127.0.0.1:8030 $user@$ip"
    note "leave that window open, and browse to http://localhost:8030"
    note "To work on that computer:  ssh $user@$ip"
}

# ---------- everyday jobs ----------------------------------------------------------
do_backup() {
    # Switched off (More 14): nothing changes, so the nightly backup rests too.
    if [ "${1:-}" = nightly ] && garden_is_off; then note "The garden is switched off: no nightly backup needed."; return 0; fi
    say "Backing up the database, photos, data, cache$(has_part wiki && echo ', wiki')$(has_part n8n && echo ', n8n')$(has_part gate && echo ', sign-in page')"
    tools tools backup "${1:-}" || return 1
    mkdir -p logs
    offsite_copy
    [ -z "$(env_get BACKUP_NAS_DIR)$(env_get BACKUP_CLOUD)" ] && note "Tip: a backup on the same computer is only half a backup. Menu 6 connects your NAS or Google Drive."
    return 0
}

# ---------- 3-2-1: copies of every backup on a NAS and in Google Drive ---------------
# 3 copies, on 2 kinds of storage, 1 away from home:
#   1. backups/ on this computer (every night)
#   2. your NAS, a network drive at home (every night, newest 14 kept)
#   3. Google Drive, away from home (once a week, newest 4 kept: each backup
#      is about 800 MB and free Google Drive has 15 GB)
NAS_MOUNT=/mnt/garden-nas
NAS_CRED=/etc/garden-nas.credentials
# Up to 3 network places (a NAS, a second NAS, a PC's shared folder). Place 1
# uses the names above; places 2 and 3 add -2 / -3 (/mnt/garden-nas-2,
# /etc/garden-nas-2.credentials, BACKUP_NAS_DIR_2 in .env).
NAS_SLOTS="1 2 3"
nas_mount_of() { if [ "$1" = 1 ]; then echo "$NAS_MOUNT"; else echo "$NAS_MOUNT-$1"; fi; }
nas_cred_of()  { if [ "$1" = 1 ]; then echo "$NAS_CRED"; else echo "/etc/garden-nas-$1.credentials"; fi; }
nas_key()      { if [ "$2" = 1 ]; then echo "$1"; else echo "${1}_$2"; fi; }   # nas_key BACKUP_NAS_DIR 2
nas_dir_of()   { env_get "$(nas_key BACKUP_NAS_DIR "$1")"; }
nas_free_slot() { local n; for n in $NAS_SLOTS; do [ -z "$(nas_dir_of "$n")" ] && { echo "$n"; return 0; }; done; return 1; }
# Disconnect one network place: its fstab line, its login file, its .env entry. Its files stay.
nas_disconnect() {
    local n=$1 mnt; mnt=$(nas_mount_of "$n")
    sudo umount "$mnt" 2>/dev/null
    sudo sed -i "\# $mnt cifs #d" /etc/fstab
    sudo rm -f "$(nas_cred_of "$n")"
    env_set "$(nas_key BACKUP_NAS_DIR "$n")" ""; env_set "$(nas_key BACKUP_NAS_HOST "$n")" ""
}

# The full path of the newest backup file.
newest_backup_path() { local d; d=$(env_get BACKUP_DIR); printf '%s/%s' "${d:-./backups}" "$(newest_backup)"; }

# Called after every backup: copy the newest one to the NAS and (weekly) to Drive.
offsite_copy() {
    local nas cloud f stamp n tag
    f=$(newest_backup_path); [ -f "$f" ] || return 0
    cloud=$(env_get BACKUP_CLOUD)
    # Every connected network place gets the copy (place 1 is "NAS", the others "NAS2", "NAS3").
    for n in $NAS_SLOTS; do
        nas=$(nas_dir_of "$n"); [ -n "$nas" ] || continue
        tag=NAS; [ "$n" = 1 ] || tag=NAS$n
        if [ -d "$nas" ] && [ -w "$nas" ]; then
            cp "$f" "$nas/" && note "$tag copy: $nas/$(basename "$f")"
            # Keep the newest 14 garden backups there; delete older ones.
            find "$nas" -maxdepth 1 -name 'junsgarden-backup_*.tar' -printf '%T@ %p\n' | sort -rn | tail -n +15 | cut -d' ' -f2- | xargs -r rm -f
            echo "$(date '+%Y-%m-%d %H:%M') $tag OK $(basename "$f")" >> logs/offsite.log
            # The weekly copy of the WHOLE garden folder (place 1 only), if you chose it.
            if [ "$n" = 1 ] && [ "$(env_get NAS_FULL_COPY)" = yes ]; then
                local fs; fs=$(grep " FULL OK " logs/offsite.log 2>/dev/null | tail -1 | cut -c1-10)
                if [ -z "$fs" ] || [ $(( ( $(date +%s) - $(date -d "$fs" +%s) ) / 86400 )) -ge 6 ]; then nas_full_copy; fi
            fi
        else
            warn "the network place $nas is not reachable (switched off, or not connected). Menu 6, then 4 (or 13)."
            echo "$(date '+%Y-%m-%d %H:%M') $tag FAILED (not reachable)" >> logs/offsite.log
        fi
    done
    if [ -n "$cloud" ] && command -v rclone >/dev/null 2>&1; then
        stamp=$(grep ' CLOUD OK ' logs/offsite.log 2>/dev/null | tail -1 | cut -c1-10)
        if [ -z "$stamp" ] || [ "${FORCE_CLOUD:-0}" = 1 ] || [ $(( ( $(date +%s) - $(date -d "$stamp" +%s) ) / 86400 )) -ge 6 ]; then
            if rclone copy "$f" "$cloud" 2>>logs/offsite.log; then
                note "Google Drive copy: $cloud/$(basename "$f")"
                # Keep the newest 4 there.
                rclone lsf "$cloud" --include 'junsgarden-backup_*.tar' 2>/dev/null | sort -r | tail -n +5 |
                    while read -r old; do rclone deletefile "$cloud/$old"; done
                echo "$(date '+%Y-%m-%d %H:%M') CLOUD OK $(basename "$f")" >> logs/offsite.log
            else
                warn "the Google Drive copy failed (see logs/offsite.log). Menu 6, then 5 to reconnect."
                echo "$(date '+%Y-%m-%d %H:%M') CLOUD FAILED" >> logs/offsite.log
            fi
        fi
    fi
    return 0
}

# Look for NAS boxes on the home network: every address that answers on the
# file-sharing port (445, SMB). It only knocks; it never logs in anywhere.
nas_find() {
    local net base i found=()
    net=$(home_subnet); base=${net%.*}
    [ -n "$base" ] || { note "Could not work out your home network."; return 1; }
    note "Searching $base.1 to $base.254 for file-sharing devices (about 10 seconds)..."
    mapfile -t found < <(for i in $(seq 1 254); do
        ( timeout 1 bash -c "</dev/tcp/$base.$i/445" 2>/dev/null && echo "$base.$i" ) &
    done; wait)
    [ "${#found[@]}" -gt 0 ] || { note "None found. Is the NAS switched on and on the same network?"; return 1; }
    NAS_FOUND=()
    for i in "${found[@]}"; do
        local nm; nm=$(nmblookup -A "$i" 2>/dev/null | awk '/<00>/ && !/GROUP/ {print $1; exit}')
        [ -n "$nm" ] || nm=$(getent hosts "$i" | awk '{print $2}')
        NAS_FOUND+=("$i ${nm:-(no name)}")
    done
    local n=0 entry
    for entry in "${NAS_FOUND[@]}"; do n=$((n + 1)); printf '   %d) %s\n' "$n" "$entry"; done
    note "(A Windows PC that shares files shows up here too.)"
}

# Backups menu 4: connect your NAS shared folder and copy every backup there.
# do_nas [PLACE]: place 1 (menu 4) or 2 / 3 (menu 13, another network place).
do_nas() {
    local slot=${1:-1}
    explain "Your NAS (network drive) as a second backup place" \
        "A NAS is a storage box on your home network. Ubuntu connects to its shared folder with SMB (the same way Windows does, up to SMB 3.1.1), using Ubuntu's cifs-utils program. The folder appears at $(nas_mount_of "$slot") and reconnects by itself after a restart." \
        "If this laptop's disk dies, your garden is still on the NAS. After every nightly backup a copy goes there (the newest 14 are kept). You can also keep a copy of the WHOLE garden folder there." \
        "https://manpages.ubuntu.com/manpages/noble/man8/mount.cifs.8.html"
    Lblock <<'EOF'
   You need on the NAS (Synology: Control Panel > Shared Folder, and Control Panel > User;
   other makers have the same two things under similar names):
   - a shared folder for backups, e.g. "backups"
   - a NAS user name and password that may write to that folder
   - SMB switched on (it usually is). SMB 3 is used; the newest is 3.1.1.
EOF
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq cifs-utils smbclient >/dev/null ||
        { fail "cifs-utils / smbclient did not install"; return 1; }
    local ip share sub user pw pick shares
    if confirm "Search your home network for the NAS?" y && nas_find; then
        pick=$(ask "Number of your NAS (or type its address)" 1)
        case "$pick" in *[!0-9]*) ip=$pick ;; *) ip=${NAS_FOUND[$((pick - 1))]%% *} ;; esac
    else
        ip=$(ask "NAS address, e.g. 192.168.1.20" "")
    fi
    [ -n "$ip" ] || { fail "the NAS address is needed"; return 1; }
    timeout 5 bash -c "</dev/tcp/$ip/445" 2>/dev/null || { fail "no answer from $ip on the file-sharing port (445). Is the NAS on, and SMB switched on?"; return 1; }
    user=$(ask "NAS user name" "")
    pw=$(ask_secret "NAS password for $user")
    [ -n "$user" ] && [ -n "$pw" ] || { fail "the user name and password are needed"; return 1; }
    # List the shared folders, so you can pick one instead of guessing its name.
    # The password goes to smbclient in its PASSWD variable, not on the command line.
    mapfile -t shares < <(PASSWD="$pw" smbclient -L "//$ip" -U "$user" -g 2>/dev/null | awk -F'|' '$1=="Disk" && $2 !~ /\$$/ {print $2}')
    if [ "${#shares[@]}" -gt 0 ]; then
        note "Shared folders on $ip:"
        local n=0 s; for s in "${shares[@]}"; do n=$((n + 1)); printf '   %d) %s\n' "$n" "$s"; done
        pick=$(ask "Number of the folder for backups (or type its name)" 1)
        case "$pick" in *[!0-9]*) share=$pick ;; *) share=${shares[$((pick - 1))]:-} ;; esac
    else
        note "Could not list the shared folders (wrong password, or the NAS hides them)."
        share=$(ask "Shared folder name on the NAS" "backups")
    fi
    sub=$(ask "Folder inside it for the garden" "junsgarden")
    [ -n "$share" ] || { fail "the shared folder name is needed"; return 1; }
    nas_connect "$ip" "$share" "$sub" "$user" "$pw" "$slot" || return 1
    unset pw
    note "Undo: menu 6, then 8. Change to another NAS: menu 6, then 9. Add another place: menu 6, then 13."
    confirm "Copy the newest backup there now?" y && offsite_copy
    if [ "$slot" = 1 ] && confirm "Also keep a copy of the WHOLE garden folder on the NAS (weekly)?" y; then
        env_set NAS_FULL_COPY yes; nas_full_copy
    fi
}

# Backups menu 13: another network place (a second NAS, a PC's shared folder),
# so every nightly backup also goes there. Up to 3 places in all.
do_nas_add() {
    local slot n
    say "Another network place for the backups"
    for n in $NAS_SLOTS; do [ -n "$(nas_dir_of "$n")" ] && note "Place $n: $(nas_dir_of "$n")"; done
    slot=$(nas_free_slot) || { note "All 3 places are in use. Menu 6, then 8 stops one first."; return 0; }
    note "This becomes place $slot. Every nightly backup goes to each place (the newest 14 kept in each)."
    do_nas "$slot"
}

# Backups menu 8: stop copying to one network place, or to Google Drive.
do_offsite_stop() {
    local n any=0 pick
    [ -n "$(for n in $NAS_SLOTS; do nas_dir_of "$n"; done)" ] && note "The network places that get every backup:"
    for n in $NAS_SLOTS; do
        [ -n "$(nas_dir_of "$n")" ] && { note "$n) $(nas_dir_of "$n")  ($(env_get "$(nas_key BACKUP_NAS_HOST "$n")"))"; any=1; }
    done
    if [ "$any" = 1 ]; then
        pick=$(ask "Number of the network place to stop (0 = none)" 0)
        case " $NAS_SLOTS " in
            *" $pick "*) if [ -n "$(nas_dir_of "$pick")" ]; then nas_disconnect "$pick"; note "Stopped. Its line is out of /etc/fstab (the files on it stay)."; fi ;;
        esac
    fi
    if [ -n "$(env_get BACKUP_CLOUD)" ] && confirm "Stop copying to Google Drive?" n; then
        env_set BACKUP_CLOUD ""; note "Stopped (the Drive files stay; rclone config: rclone config delete garden-gdrive)."
    fi
}

# Connect one NAS share as network place 1, 2 or 3 (used by do_nas and do_nas_change).
nas_connect() {   # nas_connect IP SHARE SUBFOLDER USER PASSWORD [PLACE]
    local ip=$1 share=$2 sub=$3 user=$4 pw=$5 slot=${6:-1} line mnt cred
    mnt=$(nas_mount_of "$slot"); cred=$(nas_cred_of "$slot")
    note "Saving the NAS login in $cred (only the administrator can read it):"
    ( umask 077; printf 'username=%s\npassword=%s\n' "$user" "$pw" | sudo tee "$cred" >/dev/null )
    run sudo chmod 600 "$cred"
    run sudo mkdir -p "$mnt"
    note "Adding one line to /etc/fstab (the list of drives Ubuntu connects at start-up):"
    line="//$ip/$share $mnt cifs credentials=$cred,uid=$(id -u),gid=$(id -g),iocharset=utf8,_netdev,nofail,x-systemd.automount 0 0"
    sudo cp /etc/fstab /etc/fstab.bak-garden
    ( grep -v " $mnt cifs " /etc/fstab; echo "$line" ) | sudo tee /etc/fstab.new >/dev/null && sudo mv /etc/fstab.new /etc/fstab
    note "   $line"
    run sudo systemctl daemon-reload
    run sudo mount "$mnt" 2>/dev/null || ls "$mnt" >/dev/null 2>&1
    if ! mkdir -p "$mnt/$sub" 2>/dev/null || ! touch "$mnt/$sub/.garden-test" 2>/dev/null; then
        fail "connected, but cannot write there. Check the user may write to '$share' (and the password)."; return 1
    fi
    rm -f "$mnt/$sub/.garden-test"
    env_set "$(nas_key BACKUP_NAS_DIR "$slot")" "$mnt/$sub"
    env_set "$(nas_key BACKUP_NAS_HOST "$slot")" "$ip"
    note "Connected: $(mount | grep " $mnt " | grep -o 'vers=[0-9.]*' || true). Backups go to $mnt/$sub (on the NAS: $share/$sub)."
}

# A copy of the WHOLE garden folder (code, settings, photos, backups) on the
# NAS, so everything is in one place there. The database files themselves are
# left out because they change while the garden runs; the newest backup
# inside the copy holds the database safely. Keeps the newest 2 copies.
nas_full_copy() {
    local nas dest name
    nas=$(env_get BACKUP_NAS_DIR)
    [ -n "$nas" ] && [ -d "$nas" ] && [ -w "$nas" ] || { warn "the NAS is not connected (menu 6, then 4)"; return 1; }
    dest="$nas/whole-garden-folder"; mkdir -p "$dest"
    name="junsgarden-whole-folder_$(date +%Y-%m-%d_%H%M).tar"
    note "Copying the whole garden folder to the NAS (this includes your settings file .env:"
    note "keep the NAS private). This can take a while..."
    sudo tar -cf "$dest/$name" --exclude=./local/db --exclude=./logs -C "$HERE" . &&
        sudo chown "$(id -u):$(id -g)" "$dest/$name" 2>/dev/null
    find "$dest" -maxdepth 1 -name 'junsgarden-whole-folder_*.tar' -printf '%T@ %p\n' | sort -rn | tail -n +3 | cut -d' ' -f2- | xargs -r rm -f
    echo "$(date '+%Y-%m-%d %H:%M') FULL OK $name" >> logs/offsite.log
    note "Done: $dest/$name"
}

# Move to a different NAS (moved, or a new model or maker). The old one is
# disconnected cleanly; if it is still reachable, the newest backups come across.
do_nas_change() {
    say "Change to a different NAS"
    local old_ip old_dir tmpcred
    old_ip=$(env_get BACKUP_NAS_HOST); old_dir=$(env_get BACKUP_NAS_DIR)
    note "Now connected: ${old_ip:-nothing}  ($old_dir)"
    confirm "Disconnect it and connect a new one?" y || return 0
    # Keep the old login for a moment, to copy the newest backups across.
    # sudo READS the protected file; the copy goes into our own private temp file (mode 600).
    tmpcred=$(mktemp); chmod 600 "$tmpcred"
    # shellcheck disable=SC2024
    sudo cat "$NAS_CRED" > "$tmpcred" 2>/dev/null
    local oldline; oldline=$(grep " $NAS_MOUNT cifs " /etc/fstab 2>/dev/null | awk '{print $1}')
    sudo umount "$NAS_MOUNT" 2>/dev/null
    sudo sed -i "\# $NAS_MOUNT cifs #d" /etc/fstab
    env_set BACKUP_NAS_DIR ""
    note "Old NAS disconnected (its files stay on it)."
    do_nas || { rm -f "$tmpcred"; return 1; }
    if [ -n "$oldline" ] && [ -s "$tmpcred" ] && confirm "Copy the newest 3 garden backups from the old NAS to the new one?" y; then
        sudo mkdir -p /mnt/garden-nas-old
        if sudo mount -t cifs "$oldline" /mnt/garden-nas-old -o "credentials=$tmpcred,uid=$(id -u),gid=$(id -g)" 2>/dev/null; then
            find "/mnt/garden-nas-old/${old_dir##*/}" -maxdepth 1 -name 'junsgarden-backup_*.tar' -printf '%T@ %p\n' 2>/dev/null |
                sort -rn | head -3 | cut -d' ' -f2- | while read -r f; do cp "$f" "$(env_get BACKUP_NAS_DIR)/" && note "copied $(basename "$f")"; done
            sudo umount /mnt/garden-nas-old
        else
            note "The old NAS did not answer; the new one starts with today's backup."
        fi
    fi
    rm -f "$tmpcred"
}

# Backups menu 5: connect Google Drive with rclone and copy a backup there weekly.
do_cloud() {
    explain "Google Drive as the away-from-home copy (rclone)" \
        "rclone is a well-known free program for copying files to cloud storage. It is installed from Ubuntu's own app store. You sign in to Google in your browser; rclone only gets permission for files IT creates (it cannot see the rest of your Drive)." \
        "If the house has a fire or a flood, a copy far away survives. Once a week the newest backup goes to the Drive folder 'JunsGarden-backups' (the newest 4 are kept, about 3.2 GB)." \
        "https://rclone.org/drive/"
    run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 install -y -qq rclone >/dev/null || { fail "rclone did not install"; return 1; }
    if ! rclone listremotes 2>/dev/null | has -x 'garden-gdrive:'; then
        # rclone's shared Google "client ID" is being retired during 2026
        # (https://rclone.org/drive/#making-your-own-client-id), so the weekly
        # copy would stop working one day. Your own free one does not.
        Lblock <<'EOF'
   First, your own Google "client ID" (free, about 10 minutes, once).
   rclone's shared one is being retired during 2026; with your own, the weekly
   copy keeps working. Official steps: https://rclone.org/drive/#making-your-own-client-id
   1. Open https://console.developers.google.com/ with YOUR Google account.
   2. Create a project (any name, e.g. "Junsgarden backups").
   3. "Enable APIs and services": search "Drive", enable "Google Drive API".
   4. Left side: "Credentials". Click "Configure consent screen": app name
      "rclone", your email, audience "External", your contact email.
      Scope to add: https://www.googleapis.com/auth/drive.file
      (the garden only uses this one: files rclone itself makes).
   5. "Audience": "+ Add users": add your own email.
   6. "Overview": "Create OAuth client": type "Desktop app": Create.
      Copy the client ID and the client secret it shows.
   7. "Audience": click "PUBLISH APP". (Without this, Google asks you to
      sign in again every week.)
EOF
        local cid csec
        cid=$(ask "Your client ID (ends in .apps.googleusercontent.com; empty = use rclone's shared one for now)" "")
        if [ -n "$cid" ]; then
            copy_once_note
            csec=$(ask_secret "Your client secret")
            [ -n "$csec" ] || { fail "the client secret is needed with your own client ID"; return 1; }
        else
            warn "using rclone's shared client ID. It stops working during 2026: come back here (menu 6, then 5) with your own."
        fi
        Lblock <<'EOF'
   Next, a Google sign-in page opens in your browser. Choose your Google account
   and click Allow. (If you are on SSH with no browser, rclone prints what to do:
   it asks you to run "rclone authorize drive" on a computer that has a browser,
   and paste back what it prints.)
EOF
        # Not through "run": that would print the client secret on the screen.
        echo "   > rclone config create garden-gdrive drive scope=drive.file${cid:+ client_id=... client_secret=(hidden)}"
        rclone config create garden-gdrive drive scope=drive.file ${cid:+client_id="$cid"} ${csec:+client_secret="$csec"} \
            || { unset csec; fail "Google sign-in did not finish. Try again: menu 6, then 5."; return 1; }
        unset csec
    fi
    run rclone mkdir "garden-gdrive:JunsGarden-backups" || { fail "cannot make the Drive folder"; return 1; }
    env_set BACKUP_CLOUD "garden-gdrive:JunsGarden-backups"
    note "Connected. The rclone settings (with its Google permission) are in ~/.config/rclone/rclone.conf (only you can read it)."
    if confirm "Copy the newest backup to Google Drive now? (about 800 MB)" y; then FORCE_CLOUD=1 offsite_copy; fi
    note "In Google Drive you will see a folder named JunsGarden-backups."
}

# Menu 6: everything about backups (now, restore, check, NAS, Google Drive).
do_backups_menu() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    local c
    while true; do
        say "Backups (3-2-1: this computer, your NAS, Google Drive)"
        note "This computer: $(env_get BACKUP_DIR)  newest: $(newest_backup)"
        note "NAS copies:    $(for n in $NAS_SLOTS; do nas_dir_of "$n"; done | paste -sd' ' -)"
        note "Google Drive:  $(env_get BACKUP_CLOUD || true)"
        note "Last copies:   $(tail -2 logs/offsite.log 2>/dev/null | tr '\n' ' ')"
        Lblock <<'EOF'

   1) Back up now (and copy to the NAS / Drive)
   2) Restore a backup
   3) Check the newest backup (fingerprints)
   4) Connect your NAS (network drive)
   5) Connect Google Drive
   6) Copy the newest backup to the NAS / Drive now
   7) Show the backup schedule (cron)
   8) Stop copying to the NAS or Google Drive
   9) Change to a different NAS (moved, new model or maker)
  10) Copy the WHOLE garden folder to the NAS now
  11) Load the photos download (now, or again)
  12) Keep a copy of the downloads (the zips) on the NAS
  13) Add another network place (a second NAS, or a PC's shared folder)
   0) Back
EOF
        c=$(ask "Number" 0)
        case "$c" in
            0) return 0 ;;
            1) do_backup ;;
            2) do_restore ;;
            3) tools tools verify latest ;;
            4) do_nas ;;
            5) do_cloud ;;
            6) FORCE_CLOUD=1 offsite_copy ;;
            7) crontab -l 2>/dev/null | grep garden || note "No schedule. Menu 1 (install) adds it." ;;
            8) do_offsite_stop ;;
            9) do_nas_change ;;
            10) nas_full_copy ;;
            11) do_photos ;;
            12) nas_copy_downloads ;;
            13) do_nas_add ;;
            *) note "Type one of the numbers." ;;
        esac
    done
}

# Menu 7: put a backup back (a safety backup of what is there now is made first).
do_restore() {
    local dir file n pick
    dir=$(env_get BACKUP_DIR); dir=${dir:-./backups}
    file=${1:-}
    if [ -z "$file" ]; then
        mapfile -t list < <(find "$dir" -maxdepth 1 -name 'junsgarden-backup_*.tar' -printf '%f\n' | sort -r)
        [ "${#list[@]}" -gt 0 ] || { fail "there are no backups in $dir"; return 1; }
        say "Which backup? (newest first)"
        for n in "${!list[@]}"; do printf '  %2d) %s\n' $((n + 1)) "${list[$n]}"; done
        pick=$(ask "Number" 1)
        file=${list[$((pick - 1))]:-}
        [ -n "$file" ] || { fail "no backup number $pick"; return 1; }
    fi
    confirm "Restore $file? What is there now is saved first (label before-restore)" y || return 0
    say "Stopping the programs while the backup goes back"
    dc stop site >/dev/null 2>&1
    has_part beds && dc stop beds >/dev/null 2>&1
    has_part wiki && dc stop wiki >/dev/null 2>&1
    dc up -d db >/dev/null
    tools tools restore "$file" || { warn "restore stopped; starting the programs again as they were"; dc up -d; return 1; }
    dc up -d >/dev/null
    # The wiki and the sign-in page keep settings in memory: restart them so
    # they read what was just restored.
    has_part wiki && dc restart wiki >/dev/null 2>&1
    reload_signin
    wait_site
    # A backup from before a rename has the old names: offer to rename them again.
    redo_renames
}

# Set a password for the website AND the sign-in page (typed hidden, stored as a hash).
do_password() {
    local name=${1:-}
    [ -n "$name" ] || name=$(ask "Login name" "$(env_get GARDEN_USERS | cut -d= -f1)")
    note "One password for both the website login and your sign-in page."
    if [ -t 0 ]; then tools_tty tools set-password "$name"; else tools tools set-password "$name"; fi
    reload_signin
}

# Email for "forgot password" links, offered at the end of the install (EASY and
# CUSTOM). Skipping is fine, but it asks once more: without email, nobody can
# reset a forgotten password by email.
offer_email() {   # offer_email "the question"
    if confirm "$1" n; then do_email
    elif ! confirm "Are you sure? Without email, you and friends you add later cannot reset a forgotten password by email (menu 8, then 3 adds it any time). Skip email for now?" y; then
        do_email
    fi
}

# Set up Gmail for sending email (address + app password), then send a test email.
do_email() {
    explain "Sending email with your Gmail" \
        "The website (password links), your sign-in page and n8n send email through Gmail's own mail server (smtp.gmail.com, port 587, encrypted)." \
        "Email sent straight from a home internet address is treated as spam, so a real mail service is the reliable way. Gmail needs an APP PASSWORD (16 letters), not your normal password." \
        "https://support.google.com/accounts/answer/185833  (Google: sign in with app passwords)"
    Lblock <<'EOF'
   Nothing is installed for this: Gmail's own mail server sends the emails.
   Tip: a Gmail just for your website keeps your personal inbox apart
   (free: https://accounts.google.com/signup ; guide 00b).
   Get the app password first (5 minutes, free, in your web browser):
   1. Turn on 2-Step Verification (Google asks for it before app passwords):
        https://myaccount.google.com/signinoptions/twosv
   2. Open  https://myaccount.google.com/apppasswords , type the name  garden ,
      click Create. Google shows 16 letters ONCE: copy them NOW, before you
      close that page. If they are lost, make a new one (step 2 again).
   3. Come back here, paste them and press Enter (nothing shows while you
      paste). The garden saves them in the right place by itself (.env, which
      only you can read) and uses them from then on.
   No email yet? Your password can always be set here instead: menu U, then 5.

   Two different passwords (do not mix them up):
   - Your GMAIL password: what you type to sign in to Google in your browser
     (to open the pages above). The garden never asks for it.
   - The APP password: the 16 letters Google makes in step 2. The garden uses
     it only to send email (password links for you and your friends).
   Note both in your password manager. You can remove the app password at
   any time on the app passwords page, and you can make a new one there.
EOF
    local user pw
    user=${SMTP_USER_IN:-$(ask "Gmail address" "$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)")}
    read -r -s -p "Gmail app password (16 letters, hidden): " pw; echo
    pw=${pw// /}
    [ -n "$user" ] && [ -n "$pw" ] || { fail "both are needed"; return 1; }
    env_set SMTP_HOST smtp.gmail.com; env_set SMTP_PORT 587; env_set SMTP_USER "$user"; env_set SMTP_PASSWORD "$pw"
    unset pw
    note "Saved in .env (private)."
    if [ -n "$(env_get PUBLIC_NAME)" ]; then write_authelia "$(env_get PUBLIC_NAME)"; fi
    run dc up -d >/dev/null
    if confirm "Send a test email to $user now?" y; then
        dc run --rm -T -e SMTP_HOST="$(env_get SMTP_HOST)" -e SMTP_PORT=587 -e SMTP_USER="$user" \
            -e SMTP_PASSWORD="$(env_get SMTP_PASSWORD)" tools test-email "$user"
    fi
}

# The parts screen: every optional service with ON/OFF and a number.
PARTS_LIST=(
    "beds|Garden tools (/beds): planner, checkup, diagnose, tag scan, spend|beds/README.md"
    "wiki|Wiki: your garden encyclopedia (Wiki.js)|https://docs.requarks.io"
    "n8n|n8n automations (advanced)|https://docs.n8n.io"
    "gate|Sign-in page (Authelia); set up by the front door step|https://www.authelia.com"
)
# Menu 9: switch optional parts on or off, then build/start/stop what changed.
do_parts() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    local entry key n c changed=0
    while true; do
        say "Parts: type a number to switch it ON or OFF, then A to apply"
        n=0
        for entry in "${PARTS_LIST[@]}"; do
            n=$((n + 1)); key=${entry%%|*}
            if has_part "$key"; then printf '   %d) [\033[32m ON \033[0m] ' "$n"; else printf '   %d) [ off ] ' "$n"; fi
            printf '%s\n          read: %s\n' "$(echo "$entry" | cut -d'|' -f2)" "$(echo "$entry" | cut -d'|' -f3)"
        done
        echo "   A) Apply the changes      0) Back without changing"
        c=$(ask "Your choice" "A")
        case "$c" in
            0) return 0 ;;
            a|A) break ;;
            *[!0-9]*|"") note "Type a number, A or 0." ;;
            *)  entry=${PARTS_LIST[$((c - 1))]:-}
                [ -n "$entry" ] || { note "There is no part $c."; continue; }
                key=${entry%%|*}
                if [ "$key" = gate ] && [ -z "$(env_get PUBLIC_NAME)" ]; then
                    note "The sign-in page is switched on by the front door step (menu M, then 1)."; continue
                fi
                if has_part "$key"; then part_off "$key"; else part_on "$key"; fi
                changed=1 ;;
        esac
    done
    [ "$changed" = 1 ] || { note "Nothing changed."; return 0; }
    make_folders
    say "Applying: building what is new, starting what is ON, stopping what is OFF"
    note "Switching a part OFF stops it but keeps its data, so switching it ON again brings it back."
    # shellcheck disable=SC2046  # the profile list is meant to split into words
    run dc $(profile_args) build || return 1
    for entry in "${PARTS_LIST[@]}"; do
        key=${entry%%|*}
        has_part "$key" || run dc --profile "$key" rm -sf "$(part_services "$key")" >/dev/null 2>&1
    done
    run dc up -d || return 1
    has_part wiki && wiki_setup
    note "Parts on now: core,$(env_get COMPOSE_PROFILES)"
}
# ---------- API keys: paste, test, replace, remove (never shown) ---------------------
# One line per key:  NAME_IN_ENV|what it is|who uses it|where to get one|containers to restart
KEYS_LIST=(
    "GEMINI_API_KEY|Google Gemini key (AI Studio)|the website's AI boxes and the garden tools (tag reading, identify, diagnose)|https://aistudio.google.com/apikey|site beds"
    "OPENROUTER_API_KEY|OpenRouter key (a backup when Gemini is busy)|the website's AI boxes and the garden tools|https://openrouter.ai/keys|site beds"
    "PERENUAL_KEY|Perenual plant-data key|plant facts in the plant admin and the garden tools' lookup|https://perenual.com/docs/api|site beds"
    "SMTP_PASSWORD|Gmail app password (for sending email)|password-reset emails and the sign-in page|https://myaccount.google.com/apppasswords|site authelia"
    "WIKIJS_API_KEY|Wiki API key (your own wiki)|the garden tools' wiki queue, to publish drafts to your wiki|http://localhost:3000/a/api|beds"
)
KEYS_LOG=logs/key-changes.log   # dates of changes only, never a key

# Step-by-step: how to get each kind of key, from its official page.
key_how_to() {   # step-by-step: how to get each key
    case "$1" in
        GEMINI_API_KEY) Lblock <<'EOF'
   How to get a Gemini key (free; about 2 minutes):
   1. On any computer, open  https://aistudio.google.com/apikey  and sign in with YOUR Google account.
   2. Click "Create API key". Choose "Create API key in new project" if it asks.
   3. Google shows the key. Click the copy button next to it.
      (You can see it again later in AI Studio; but treat it like a password.)
   4. Come back here and paste it (Ctrl+Shift+V in a terminal). Nothing shows while you paste.
   Free keys have daily limits. If you ever want more, guide 16 shows how to add
   billing to YOUR Google project with a spending limit. That is your choice.
EOF
        ;;
        OPENROUTER_API_KEY) Lblock <<'EOF'
   What OpenRouter is: a service that passes an AI request (for example "read the
   words on this plant tag photo") to one of many AI models, with ONE key. It is a
   backup for when Gemini is busy. It is YOUR account: nobody else can see it, and
   it never sees your computer, only what the garden sends it to read. Free models
   may let their makers keep what you send, so the garden only sends plant photos
   and plant names, never passwords or personal files.
   Privacy settings: https://openrouter.ai/settings/privacy   Docs: https://openrouter.ai/docs

   How to get an OpenRouter key (free models only):
   1. Open  https://openrouter.ai  and sign in (Google login works).
   2. Open  https://openrouter.ai/keys  and click "Create Key". Give it a name like garden.
   3. The key is shown ONCE. Copy it now (if you lose it, delete it and make a new one).
   4. Paste it here. The garden only asks OpenRouter for models whose name ends in ":free".
EOF
        ;;
        PERENUAL_KEY) Lblock <<'EOF'
   How to get a Perenual key (free plan):
   1. Open  https://perenual.com/docs/api  and sign up.
   2. Your key is on your account page. Copy it.
   3. Paste it here.
EOF
        ;;
        WIKIJS_API_KEY) Lblock <<'EOF'
   How to make a key in YOUR wiki (so the garden tools can publish drafts there):
   1. Open your wiki (http://localhost:3000 on the garden computer) and sign in as admin.
   2. Go to  Administration > API Access  (or open http://localhost:3000/a/api).
   3. Switch "Enable API" on and click Apply.
   4. Click "New API Key". Name: garden-tools. Expiration: 1 year. Access: Full Access.
   5. The key is shown ONCE. Copy it, then paste it here.
   Wiki.js documentation: https://docs.requarks.io/dev/api
EOF
        ;;
        SMTP_PASSWORD) Lblock <<'EOF'
   How to get a Gmail app password:
   1. Your Google account needs 2-Step Verification on: https://myaccount.google.com/security
   2. Open  https://myaccount.google.com/apppasswords , type the name  garden , click Create.
   3. Google shows 16 letters ONCE. Copy them (spaces do not matter).
   4. Paste here. (Settings > Email also asks for the Gmail address and sends a test.)
EOF
        ;;
    esac
}

# Ask the key's own service whether the saved key works (the key is never shown).
key_test() {   # key_test NAME: ask the service if the key works. The key is never shown.
    local name=$1 k code
    k=$(env_get "$name")
    [ -n "$k" ] || { note "Not set, nothing to test."; return 1; }
    case "$name" in
        # The key goes in a header read from standard input, so it never
        # appears in a command line or on screen.
        GEMINI_API_KEY)
            code=$(printf 'x-goog-api-key: %s\n' "$k" | curl -s -o /dev/null -w '%{http_code}' -H @- \
                "https://generativelanguage.googleapis.com/v1beta/models") ;;
        OPENROUTER_API_KEY)
            code=$(printf 'Authorization: Bearer %s\n' "$k" | curl -s -o /dev/null -w '%{http_code}' -H @- \
                "https://openrouter.ai/api/v1/key") ;;
        PERENUAL_KEY)
            code=$(curl -s -o /dev/null -w '%{http_code}' "https://perenual.com/api/species-list?key=$k&q=tomato") ;;
        WIKIJS_API_KEY)
            # Ask your own wiki for its title, using the key. 200 + a title = it works.
            if printf 'Authorization: Bearer %s\n' "$k" | curl -s -H @- -H 'Content-Type: application/json' \
                --data '{"query":"{ site { config { title } } }"}' http://127.0.0.1:3000/graphql | has '"title"'; then
                code=200; else code=401; fi ;;
        SMTP_PASSWORD)
            dc run --rm -T -e SMTP_HOST="$(env_get SMTP_HOST)" -e SMTP_PORT="$(env_get SMTP_PORT)" \
                -e SMTP_USER="$(env_get SMTP_USER)" -e SMTP_PASSWORD="$k" tools test-email "$(env_get SMTP_USER)"
            return ;;
    esac
    case "$code" in
        200) note "Works: the service accepted the key." ;;
        400|401|403) note "The service says this key is WRONG or switched off ($code). Paste it again, or make a new one." ; return 1 ;;
        429) note "The key is right, but today's free limit is used up ($code). It resets tomorrow." ;;
        *) note "No clear answer (code ${code:-none}). Check the internet, then test again." ; return 1 ;;
    esac
}

# Say exactly which file and line a key is saved in.
key_where() {   # exactly where a key is saved, and who reads it
    local name=$1 line
    line=$(grep -n "^$name=" .env | cut -d: -f1)
    note "Saved in: $HERE/.env   line ${line:-?}   ($name=...)"
    note "Only you can read that file (permission 600). Menu: never open it, use this screen."
}

# Menu K: paste, test, replace or remove API keys and app passwords.
do_keys() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    local c i entry name label users url svc when k
    while true; do
        say "API keys and app passwords (never shown on screen)"
        i=0
        for entry in "${KEYS_LIST[@]}"; do
            i=$((i + 1)); IFS='|' read -r name label users url svc <<<"$entry"
            when=$(grep " $name " "$KEYS_LOG" 2>/dev/null | tail -1 | cut -d' ' -f1)
            if [ -n "$(env_get "$name")" ]; then k="set${when:+, changed $when}"; else k="not set"; fi
            printf '   %d) %-44s [%s]\n' "$i" "$(Lt "$label")" "$(Lt "$k")"
        done
        L "   N) No-IP DDNS key (for your web name)  [saved in /etc/default/noip-duc]"
        L "   0) Back"
        c=$(ask "Number" 0)
        case "$c" in
            0) return 0 ;;
            n|N) install_noip; continue ;;
            *[!0-9]*|"") note "Type a number."; continue ;;
        esac
        entry=${KEYS_LIST[$((c - 1))]:-}; [ -n "$entry" ] || { note "No such number."; continue; }
        IFS='|' read -r name label users url svc <<<"$entry"
        say "$label"
        note "Used by: $users"
        key_where "$name"
        L "   1) Paste a new key (or replace the old one)"
        L "   2) Test the key that is saved now"
        L "   3) How to get a key, step by step"
        L "   4) Remove this key"
        L "   0) Back"
        case "$(ask "Number" 1)" in
            1)  [ "$name" = SMTP_PASSWORD ] && { do_email; continue; }
                key_how_to "$name"
                copy_once_note
                k=$(ask_secret "Paste the key now")
                k=${k//[[:space:]]/}
                [ -n "$k" ] || { note "Nothing pasted; nothing changed."; continue; }
                env_set "$name" "$k"; unset k
                mkdir -p logs; echo "$(date +%Y-%m-%d) $name changed" >> "$KEYS_LOG"
                note "Saved."; key_where "$name"
                restart_for "$svc" "$name"
                key_test "$name" || note "Not working yet? Choose 1 again and paste it once more." ;;
            2)  key_test "$name" ;;
            3)  key_how_to "$name" ;;
            4)  confirm "Remove the $label?" n || continue
                env_set "$name" ""; echo "$(date +%Y-%m-%d) $name removed" >> "$KEYS_LOG"
                restart_for "$svc" "$name"; note "Removed. Remember to delete it at the service too: $url" ;;
        esac
    done
}

# During the install: put in YOUR OWN keys straight away (each one optional).
# Nobody else's keys are in this copy; every key here is one you make.
setup_keys_now() {
    say "Your own AI keys (optional; you can also do this later with menu K)"
    note "This copy has NO keys from anybody else. The AI boxes work once you add your own free key."
    local entry name label users url svc k
    for entry in "${KEYS_LIST[@]}"; do
        IFS='|' read -r name label users url svc <<<"$entry"
        case "$name" in GEMINI_API_KEY|OPENROUTER_API_KEY) ;; *) continue ;; esac
        [ -n "$(env_get "$name")" ] && { note "$label: already set."; continue; }
        confirm "Add your $label now?" "$( [ "$name" = GEMINI_API_KEY ] && echo y || echo n )" || continue
        key_how_to "$name"
        copy_once_note
        k=$(ask_secret "Paste the key now (or just press Enter to skip)")
        k=${k//[[:space:]]/}
        [ -n "$k" ] || { note "Skipped. Menu K adds it later."; continue; }
        env_set "$name" "$k"; unset k
        mkdir -p logs; echo "$(date +%Y-%m-%d) $name set during install" >> "$KEYS_LOG"
        key_where "$name"
        restart_for "$svc" "$name"
        key_test "$name" || note "Not accepted yet? Menu K, then the key, then 1 to paste it again."
    done
}

# Restart the programs that use a key, then check each one really has it now.
restart_for() {   # restart_for "site beds" NAME
    local svc=$1 name=$2 s c var
    for s in $svc; do
        case "$s" in beds) has_part beds || continue ;; authelia) has_part gate || continue ;; esac
        run dc up -d "$s" >/dev/null 2>&1
    done
    for s in $svc; do
        case "$s" in
            site) c=garden-site; var="JUNSGARDEN_$name" ;;
            beds) has_part beds || continue; c=junsgarden-ingest; var=$name ;;
            *) continue ;;
        esac
        [ "$name" = SMTP_PASSWORD ] && var=JUNSGARDEN_SMTP_PASSWORD
        # shellcheck disable=SC2016
        if dk exec "$c" sh -c "test -n \"\${$var:-}\"" 2>/dev/null; then note "$c has the key now."
        else note "$c does not have a key now."; fi
    done
}

# ---------- settings: change anything without opening a file ---------------------
show_value() {   # secrets are never shown, only whether they are set
    local v; v=$(env_get "$1")
    case "$1" in *PASSWORD*|*SECRET*|*_KEY) [ -n "$v" ] && echo "(set, hidden)" || echo "(not set)" ;;
                 *) echo "${v:-(not set)}" ;; esac
}
# Ask for a password or key with nothing shown while typing; prints the answer.
# Said before every key or token is pasted: most sites show a key only once.
copy_once_note() {
    note "Most sites show a key only ONCE: copy it before you leave their page, or make a new one there."
    note "Paste it here and press Enter: the garden saves it in the right place, private to you."
}
ask_secret() {   # ask_secret "what to type" -> prints the answer, nothing shows while typing
    local a; read -r -s -p "$(Lt "$1") $(Lt "(hidden while you type)"): " a; echo >&2; printf '%s' "$a"
}

# Menu 8: change any setting by number, without opening a file.
do_settings() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    local c v v2
    while true; do
        say "Settings (passwords are never shown). Type a number, or 0 to go back."
        printf '   1) Your login name and email ........ %s\n' "$(show_value GARDEN_USERS)"
        printf '   2) Time zone ......................... %s\n' "$(show_value TZ)"
        printf '   3) Email (Gmail) ..................... %s / app password %s\n' "$(show_value SMTP_USER)" "$(show_value SMTP_PASSWORD)"
        L "   4) API keys (Gemini, OpenRouter, Perenual, Gmail, No-IP): paste, test, replace"
        L "   5) Your password (website + sign-in page)"
        printf '   6) %s ...... %s\n' "$(Lt "Who may use the garden tools")" "$(show_value BEDS_ALLOWED_EMAILS)"
        printf '   7) %s ............. %s\n' "$(Lt "Web name (front door)")" "$(show_value PUBLIC_NAME)"
        L "   8) Rename or move a file or folder"
        L "   9) Renew the internal database passwords"
        printf '  10) %s ................. %s\n' "$(Lt "Local AI computer")" "$(show_value OLLAMA_URL)"
        L "   (Every setting is saved in $HERE/.env; this screen changes it for you.)"
        c=$(ask "Number" 0)
        case "$c" in
            0) return 0 ;;
            1)  v=$(ask "Your first name (letters only)" "$(env_get GARDEN_USERS | cut -d= -f1)")
                case "$v" in *[!A-Za-z]*|"") note "Letters only, please."; continue ;; esac
                v2=$(ask "Your email address" "$(env_get GARDEN_USERS | cut -d= -f2 | cut -d, -f1)")
                case "$v2" in *@*.*) ;; *) note "That does not look like an email."; continue ;; esac
                env_set GARDEN_USERS "$v=$v2"; env_set GARDEN_UPLOADER_NAMES "$v"
                note "Saved. If you changed the NAME, set a password for it: Settings, 5." ;;
            2)  note "Examples: America/New_York  America/Chicago  America/Denver  America/Los_Angeles  Asia/Seoul"
                v=$(ask "Time zone" "$(env_get TZ)")
                [ -f "/usr/share/zoneinfo/$v" ] || { note "Unknown time zone: $v"; continue; }
                env_set TZ "$v" ;;
            3)  do_email; continue ;;
            4)  do_keys; continue ;;
            5)  do_password; continue ;;
            6)  v=$(ask "Emails allowed in the garden tools, comma between" "$(env_get BEDS_ALLOWED_EMAILS)")
                env_set BEDS_ALLOWED_EMAILS "${v// /}" ;;
            7)  do_public; continue ;;
            8)  do_rename; continue ;;
            9)  do_renew_db_passwords; continue ;;
            10) do_ai; continue ;;
            *)  note "Type one of the numbers."; continue ;;
        esac
        note "Saved in .env. Restarting what uses it..."
        run dc up -d >/dev/null && note "Done."
    done
}

# ---------- rename or move a file or folder (Settings 8, More 18, garden.sh rename) -----
# A name the garden uses is written in several places: the code, the pages, the
# photo lists, the database. Renaming only the folder would leave those pointing
# at nothing. This finds every place by the EXACT old name (as the Website name
# swap does), shows each one, keeps copies, stops the programs, changes it all,
# starts them again and checks the website. Plain search and replace: no AI,
# nothing guessed. Every rename and move is written in logs/renames.tsv.
RENAME_LOG=logs/renames.tsv
# The picture folders the website came with (IMAGE_FOLDERS in .env after a rename).
IMAGE_FOLDERS_FIRST="Database Gallery post blog plan HeroBanner"

do_rename() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    say "Rename or move a file or folder"
    Lblock <<'EOF'
   Before you start:
   - The guides and the walkthrough show the names the garden came with (for
     example local/images/Gallery). After a rename or a move they show the OLD
     names. Every change is listed in logs/renames.tsv: what it was, what it is now.
   - This is a plain script, no AI. It looks for the exact old name, shows you
     every place it found, and changes nothing until you say yes.
   - It offers a full backup first, and stops the programs while it works (that
     also frees the files), then starts them again and checks the website.

   1) A file or folder of the website (a photo folder, a page, a data file):
      renamed, and every place that uses it is changed with it
   2) One of the garden's big folders (photos, database, backups...) to
      another place or another disk
   3) The WHOLE garden folder to another place
   4) Show what was renamed or moved so far
   0) Back
EOF
    case "$(ask "Number" 0)" in
        1) rename_item ;;
        2) do_move_folder ;;
        3) move_garden ;;
        4) show_renames ;;
        *) return 0 ;;
    esac
}

# Choice 4: the list of renames and moves (logs/renames.tsv).
show_renames() {
    if [ -s "$RENAME_LOG" ]; then
        column -t -s $'\t' "$RENAME_LOG" 2>/dev/null || tr '\t' ' ' < "$RENAME_LOG"
    else
        note "Nothing renamed or moved yet: everything has the names the guides show."
    fi
}
# Write one line in logs/renames.tsv:  when, what (renamed / moved), old, new.
rename_log() {
    mkdir -p logs
    [ -s "$RENAME_LOG" ] || printf 'when\twhat\told name or place\tnew name or place\n' > "$RENAME_LOG"
    printf '%s\t%s\t%s\t%s\n' "$(date '+%Y-%m-%d %H:%M')" "$1" "$2" "$3" >> "$RENAME_LOG"
}

# Ask for a path, with Tab completing folder and file names as you type.
ask_path() { local a; read -r -e -p "$(Lt "$1"): " a; printf '%s' "$a"; }
# Tidy a typed path: ~ means your home folder; no ./ at the front; no / at the end.
tidy_path() {
    local p=$1
    # shellcheck disable=SC2088  # the ~ is matched as typed, then swapped for $HOME
    case "$p" in "~") p=$HOME ;; "~/"*) p=$HOME/${p#"~/"} ;; esac
    while [ "${p#./}" != "$p" ]; do p=${p#./}; done
    while [ "${p%/}" != "$p" ] && [ "$p" != / ]; do p=${p%/}; done
    printf '%s' "$p"
}
# The kind of disk a place is on (ext2/ext3 means ext4 too), and whether it is a
# Linux disk. The database and the programs need one: Windows and USB disks
# (NTFS, exFAT, FAT) and network drives cannot keep Linux owners and permissions.
disk_type() { local d=$1; while [ ! -d "$d" ]; do d=$(dirname "$d"); done; stat -f -c %T "$d" 2>/dev/null; }
linux_disk() { case "$(disk_type "$1")" in ext2/ext3|ext4|btrfs|xfs|zfs|f2fs) return 0 ;; *) return 1 ;; esac; }

# Stop the garden's programs, so nothing is using the files ("unlocks" them),
# then list anything else that still has a file open in PATH.
free_files() {   # free_files PATH [keep-db]
    note "Stopping the garden's programs, so nothing is using the files:"
    if [ "${2:-}" = keep-db ]; then
        # Everything except the database, which stays on for the changes in its rows.
        local -a others
        mapfile -t others < <(dc ps --services --status running 2>/dev/null | grep -vx db)
        [ "${#others[@]}" -gt 0 ] && run dc stop "${others[@]}" >/dev/null 2>&1
        wait_db || { fail "the database does not answer"; return 1; }
    else
        run dc stop >/dev/null 2>&1
    fi
    command -v lsof >/dev/null 2>&1 || return 0
    local busy
    # This menu itself (and the terminal that started it) are left out of the list.
    busy=$(cd / && sudo lsof -w +D "$1" 2>/dev/null | awk -v me=$$ -v up=$PPID 'NR > 1 && $2 != me && $2 != up {print "      " $1 " (process " $2 ")"}' | sort -u)
    [ -z "$busy" ] && return 0
    warn "these programs still have files open in there:"
    printf '%s\n' "$busy"
    note "Close them first: a file window, an editor, or another terminal that is inside that folder (type cd ~ there)."
    confirm "Carry on anyway?" n
}

# Which website area holds a path? Prints "setting<TAB>its folder<TAB>the path inside it".
# Only the website's own files can be renamed: site/, the photos and the data.
rename_area() {
    local p k root
    p=$(realpath -m -- "$1")
    for k in IMAGES_DIR SITE_DIR DATA_DIR; do
        root=$(realpath -m -- "$(env_get "$k")")
        case "$p" in "$root"/*) printf '%s\t%s\t%s\n' "$k" "$root" "${p#"$root"/}"; return 0 ;; esac
    done
    return 1
}

# The exact-name search and change (perl, like the Website name swap). The pairs
# come in REN_OLD1 / REN_NEW1 / REN_STRICT1, REN_OLD2 ... Modes: "show" prints each
# line with an old name, "count" only counts, "apply" changes the files. Each file
# with a match ends with:  COUNT<TAB>number<TAB>file.
# A name counts only when it is not part of a longer name (Gallery is not Gallery2,
# app.js is not app.js.map). STRICT names (the short names of site and data files)
# must also come right after / " ' ( = or `, so a word in a sentence is never changed.
read -r -d '' RENAME_PERL <<'PERL'
use strict; use warnings;
my ($mode, @files) = @ARGV;
my @pairs;
for (my $i = 1; defined $ENV{"REN_OLD$i"}; $i++) {
    my $t = quotemeta $ENV{"REN_OLD$i"};
    my $re = $ENV{"REN_STRICT$i"}
        ? qr{(?<=[/"'(=`])$t(?=[/"'?#)`>,;\s]|$)}m
        : qr{(?<![A-Za-z0-9_.-])$t(?![A-Za-z0-9_-]|\.[A-Za-z0-9])};
    push @pairs, [$re, $ENV{"REN_NEW$i"}];
}
for my $f (@files) {
    open(my $in, '<:raw', $f) or next;
    my $s = do { local $/; <$in> };
    close $in;
    my $n = 0;
    if ($mode eq 'apply') {
        for my $p (@pairs) { my $new = $p->[1]; my $c = ($s =~ s/$p->[0]/$new/g) || 0; $n += $c; }
        if ($n) {
            open(my $out, '>:raw', $f) or die "cannot write $f: $!\n";
            print $out $s;
            close $out or die "cannot write $f: $!\n";
        }
    } else {
        my $ln = 0;
        for my $line (split /\n/, $s) {
            $ln++;
            my $hit = 0;
            for my $p (@pairs) { $hit++ while $line =~ /$p->[0]/g; }
            next unless $hit;
            $n += $hit;
            (my $t = $line) =~ s/^\s+//;
            print "      $f:$ln: ", substr($t, 0, 100), "\n" if $mode eq 'show';
        }
    }
    print "COUNT\t$n\t$f\n" if $n;
}
PERL

# The same exact-name rule for the database (MariaDB's REGEXP), written to go
# inside '...' in SQL. (?-i) makes it match capitals exactly, like Linux does.
db_regex() {
    local t; t=$(printf '%s' "$1" | sed 's/[.}]/\\\\&/g')
    case "$1" in
        /*) printf '(?-i)%s(?![A-Za-z0-9_-]|\\\\.[A-Za-z0-9])' "$t" ;;
        *)  printf '(?-i)(?<![A-Za-z0-9_.-])%s(?![A-Za-z0-9_-]|\\\\.[A-Za-z0-9])' "$t" ;;
    esac
}
# SQL as the database's administrator (the SQL comes on standard input). The
# password is read INSIDE the database container, so it never appears here.
# shellcheck disable=SC2016
db_root() { dk exec -i garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" exec mariadb -uroot -N -B'; }
# Start the database if it is not running, and wait (up to 2 minutes) until it answers.
wait_db() {
    local i
    dc up -d db >/dev/null 2>&1
    for i in $(seq 1 40); do
        echo "SELECT 1;" | db_root >/dev/null 2>&1 && return 0
        [ "$i" = 1 ] && note "waiting for the database to start..."
        sleep 3
    done
    return 1
}
# The text columns of the garden's and the wiki's tables: "database<TAB>table<TAB>column".
db_text_columns() {
    echo "SELECT c.table_schema, c.table_name, c.column_name FROM information_schema.columns c
          JOIN information_schema.tables t ON t.table_schema = c.table_schema AND t.table_name = c.table_name
          WHERE t.table_type = 'BASE TABLE' AND c.table_schema IN ('juns_garden_db', 'junsgarden_wiki')
          AND c.data_type IN ('char', 'varchar', 'tinytext', 'text', 'mediumtext', 'longtext');" | db_root
}

# Choice 1: rename (or move inside its own area) one file or folder the website
# uses, and change every place that names it: code and pages (rust/, site/, beds/,
# docker/), the photo and data lists (like the gallery's tags_cache.csv), and the
# rows of the garden's and the wiki's database.
rename_item() {   # rename_item [OLD NEW after-restore]   (with OLD and NEW: no path questions)
    local old=${1:-} new=${2:-} again=${3:-} key root rel nkey nrel web nweb dbold='' dbnew='' other site f
    local keep i n sum code_n=0 data_n=0 db_n=0 rebuilt='' check
    local -a code_dirs code_files data_files code_hits=() data_hits=() db_hits=() ren_env=() db_toks=() db_news=() changed=()
    if [ -z "$old" ]; then
        Lblock <<'EOF'
   Which file or folder? Type its path inside the garden folder. Press Tab to
   complete the names as you type. Examples:
      local/images/Gallery        the gallery's photos
      local/images/HeroBanner     the big pictures at the top of the pages
      site/privacy.html           a page
EOF
        old=$(ask_path "File or folder (empty = back)")
    fi
    old=$(tidy_path "$old"); [ -n "$old" ] || return 0
    [ -e "$old" ] || { fail "$old is not there. Check the spelling (Tab helps)."; return 1; }
    if ! IFS=$'\t' read -r key root rel < <(rename_area "$old"); then
        Lblock <<'EOF'
   Only the website's own files can be renamed here: what is inside site/,
   local/images/ and local/data/ (or where choice 2 moved those). The programs
   themselves (rust/, beds/, docker/, garden.sh ...) keep their names: renaming
   them would break the garden. To move one of those big folders, use choice 2.
EOF
        return 1
    fi
    case "$key" in IMAGES_DIR) web=images/$rel ;; DATA_DIR) web=data/$rel ;; *) web=$rel ;; esac
    # Names the programs use by themselves, or that phones already look for.
    case "$web" in
        images/Database/.thumbs*|images/plan/web|images/plan/web/*|images/plan/tags|images/plan/tags/*|apps|apps/*)
            fail "$old is used by name inside the programs (or by the phone app), so it keeps its name."; return 1 ;;
    esac
    [[ "$rel" =~ ^[A-Za-z0-9._/-]+$ ]] || { fail "$old has a space or a sign in its name. Rename it by hand (it cannot be in any web address as it is)."; return 1; }

    if [ -z "$new" ]; then
        note "Type the new name. Just a name renames it where it is (example: Photos)."
        note "Letters, numbers, - _ and . only: no spaces (web addresses break on spaces)."
        new=$(ask_path "New name (empty = back)")
        new=$(tidy_path "$new"); [ -n "$new" ] || return 0
        case "$new" in */*) ;; *) new=$(dirname "$old")/$new; new=${new#./} ;; esac
    fi
    new=$(tidy_path "$new")
    [ -e "$new" ] && { fail "$new already exists. Pick a name that is not used."; return 1; }
    IFS=$'\t' read -r nkey _ nrel < <(rename_area "$new")
    [ "${nkey:-}" = "$key" ] || { fail "the new place must stay in the same area as the old one ($root)."; return 1; }
    [[ "$nrel" =~ ^[A-Za-z0-9._/-]+$ ]] && [[ ! "/$nrel/" =~ /\.\.?/ ]] && [[ "$nrel" != *//* ]] ||
        { fail "\"$nrel\": use letters, numbers, - _ and . only (no spaces)."; return 1; }
    case "/$nrel/" in "/$rel/"*) fail "a folder cannot go inside itself."; return 1 ;; esac
    case "$key" in IMAGES_DIR) nweb=images/$nrel ;; DATA_DIR) nweb=data/$nrel ;; *) nweb=$nrel ;; esac
    if [ -f "$old" ] && [ "${old##*.}" != "${new##*.}" ]; then
        warn "the ending changes (.${old##*.} to .${new##*.}). Only the name changes; the file itself is not converted."
        confirm "Use this name anyway?" n || return 0
    fi
    # A photo in a photo folder: the database keeps only its name inside that
    # folder (abc.webp), so it must stay in its folder, and the name must be unique.
    if [ "$key" = IMAGES_DIR ] && [ -f "$old" ] && [[ "$rel" == */* ]]; then
        [ "${rel%%/*}" = "${nrel%%/*}" ] || { fail "a photo can be renamed inside its own photo folder (${rel%%/*}), not moved to another one."; return 1; }
        dbold=${rel#*/}; dbnew=${nrel#*/}
        for other in "$root"/*/"$dbold"; do
            [ -e "$other" ] && [ "$other" != "$root/$rel" ] &&
                { fail "another photo folder has a file with the same name ($other). The database cannot tell them apart, so this one keeps its name."; return 1; }
        done
    fi

    # What to look for. Photos: images/NAME (and images}/NAME in docker-compose.yml).
    # Site and data files: their short name, only right after / " ' ( = or `.
    site=$(env_get SITE_DIR)
    case "$key" in
        IMAGES_DIR) ren_env=(REN_OLD1="$web" REN_NEW1="$nweb" REN_STRICT1= REN_OLD2="images}/$rel" REN_NEW2="images}/$nrel" REN_STRICT2=)
                    code_dirs=(rust/junsgarden/src rust/junsgarden/templates rust/common "$site" beds docker docker-compose.yml android-app/app/src)
                    db_toks=("$web"); db_news=("$nweb") ;;
        SITE_DIR)   ren_env=(REN_OLD1="$rel" REN_NEW1="$nrel" REN_STRICT1=1)
                    code_dirs=(rust/junsgarden/src rust/junsgarden/templates rust/common "$site" docker/caddy android-app/app/src)
                    db_toks=("/$rel"); db_news=("/$nrel") ;;
        DATA_DIR)   ren_env=(REN_OLD1="$rel" REN_NEW1="$nrel" REN_STRICT1=1)
                    code_dirs=(rust/junsgarden/src rust/junsgarden/templates rust/common "$site" docker)
                    db_toks=("/$web"); db_news=("/$nweb") ;;
    esac
    [ -n "$dbold" ] && { db_toks+=("$dbold"); db_news+=("$dbnew"); }

    say "Looking for every place that uses $old"
    mapfile -d '' -t code_files < <(find "${code_dirs[@]}" -type d \( -name target -o -name build -o -name .gradle -o -name node_modules -o -name .git \) -prune \
        -o -type f -size -2M ! -name '*.md' -print0 2>/dev/null)
    mapfile -d '' -t data_files < <(find "$(env_get IMAGES_DIR)" "$(env_get DATA_DIR)" "$(env_get CACHE_DIR)" -type d -name .thumbs -prune \
        -o -type f -size -50M \( -name '*.csv' -o -name '*.json' -o -name '*.txt' -o -name '*.html' \) -print0 2>/dev/null)
    while IFS=$'\t' read -r _ n f; do code_hits+=("$f"); code_n=$((code_n + n)); done < <(env "${ren_env[@]}" perl -e "$RENAME_PERL" -- count "${code_files[@]}")
    while IFS=$'\t' read -r _ n f; do data_hits+=("$f"); data_n=$((data_n + n)); done < <(env "${ren_env[@]}" perl -e "$RENAME_PERL" -- count "${data_files[@]}")
    note "Code and pages: $code_n place(s) in ${#code_hits[@]} file(s):"
    if [ "${#code_hits[@]}" -gt 0 ]; then
        env "${ren_env[@]}" perl -e "$RENAME_PERL" -- show "${code_hits[@]}" | grep -v '^COUNT' | head -40
        [ "$code_n" -gt 40 ] && note "   ... and more (all of them are changed the same way)"
    fi
    note "Photo and data lists: $data_n place(s) in ${#data_hits[@]} file(s):"
    for f in "${data_hits[@]}"; do printf '      %s\n' "$f"; done
    # The database: every text column of every table, garden and wiki.
    local where db t c tok sql=
    wait_db || note "The database does not answer yet."
    while IFS=$'\t' read -r db t c; do
        where=
        for tok in "${db_toks[@]}"; do where="${where:+$where OR }\`$c\` REGEXP '$(db_regex "$tok")'"; done
        sql+=$(printf "SELECT '%s.%s.%s', COUNT(*) FROM \`%s\`.\`%s\` WHERE %s;" "$db" "$t" "$c" "$db" "$t" "$where")$'\n'
    done < <(db_text_columns)
    if [ -n "$sql" ]; then
        while IFS=$'\t' read -r f n; do [ "${n:-0}" -gt 0 ] && { db_hits+=("$f"); db_n=$((db_n + n)); }; done < <(printf '%s' "$sql" | db_root)
    else
        warn "could not look inside the database (is it running? menu 2 starts the garden)."
        confirm "Carry on without the database?" n || return 0
    fi
    note "Database: $db_n row(s) in ${#db_hits[@]} column(s):"
    for f in "${db_hits[@]}"; do printf '      %s\n' "$f"; done
    [ $((code_n + data_n + db_n)) -eq 0 ] && note "Nothing names it: only the name itself changes. (Pictures found by looking in a folder, like the hero banner pictures, need nothing else.)"

    echo
    note "Rename: $old  ->  $new"
    [ -z "$again" ] && warn "the guides will still show the old name. logs/renames.tsv will list this change."
    confirm "Change all of this now?" y || { note "Nothing changed."; return 0; }
    if [ -z "$again" ] && confirm "Make a full backup first (recommended, a few minutes)?" y; then
        do_backup before-rename || { fail "the backup did not finish, so nothing was changed"; return 1; }
    fi

    free_files "$old" keep-db || { note "Nothing changed."; dc up -d >/dev/null; return 0; }
    keep="logs/rename_$(date +%Y-%m-%d_%H%M%S)"
    mkdir -p "$keep"
    note "Copies of everything it changes go to $keep first."
    for f in "${code_hits[@]}" "${data_hits[@]}"; do
        if ! { mkdir -p "$keep/files/$(dirname "${f#/}")" && cp -p "$f" "$keep/files/${f#/}"; }; then
            fail "could not keep a copy of $f, so nothing was changed"; dc up -d >/dev/null; return 1
        fi
    done
    if [ "${#db_hits[@]}" -gt 0 ]; then
        for db in juns_garden_db junsgarden_wiki; do
            mapfile -t changed < <(printf '%s\n' "${db_hits[@]}" | awk -F. -v d="$db" '$1 == d {print $2}' | sort -u)
            [ "${#changed[@]}" -gt 0 ] || continue
            # shellcheck disable=SC2016
            dk exec garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" exec mariadb-dump -uroot --single-transaction "$@"' sh "$db" "${changed[@]}" | gzip > "$keep/$db-before.sql.gz" ||
                { fail "could not keep a copy of the database rows, so nothing was changed"; dc up -d >/dev/null; return 1; }
        done
        chmod -R go-rwx "$keep"
    fi

    # 1. The file or folder itself.
    mkdir -p "$(dirname "$new")"
    if ! mv -- "$old" "$new" 2>/dev/null && ! run sudo mv -- "$old" "$new"; then
        fail "the rename failed; nothing was changed"; dc up -d >/dev/null; return 1
    fi
    note "Renamed: $old -> $new"
    # 2. The code, the pages and the lists.
    if [ "${#code_hits[@]}" -gt 0 ] || [ "${#data_hits[@]}" -gt 0 ]; then
        env "${ren_env[@]}" perl -e "$RENAME_PERL" -- apply "${code_hits[@]}" "${data_hits[@]}" >/dev/null ||
            warn "some files could not be changed. The copies are in $keep."
        note "Changed $((code_n + data_n)) place(s) in $(( ${#code_hits[@]} + ${#data_hits[@]} )) file(s)."
    fi
    # 3. The database rows.
    if [ "${#db_hits[@]}" -gt 0 ]; then
        sql=
        for f in "${db_hits[@]}"; do
            IFS=. read -r db t c <<< "$f"
            for i in "${!db_toks[@]}"; do
                sql+=$(printf "UPDATE \`%s\`.\`%s\` SET \`%s\` = REGEXP_REPLACE(\`%s\`, '%s', '%s') WHERE \`%s\` REGEXP '%s';" \
                    "$db" "$t" "$c" "$c" "$(db_regex "${db_toks[$i]}")" "${db_news[$i]}" "$c" "$(db_regex "${db_toks[$i]}")")$'\n'
            done
        done
        if printf '%s' "$sql" | db_root; then note "Changed $db_n database row(s)."
        else warn "the database did not take every change. Its rows from before are in $keep (gzip)."; fi
    fi
    # 4. A renamed picture folder is made under its new name by a later install.
    if [ "$key" = IMAGES_DIR ] && [[ "$rel" != */* ]]; then
        sum=$(env_get IMAGE_FOLDERS); sum=${sum:-$IMAGE_FOLDERS_FIRST}
        # shellcheck disable=SC2086  # one folder name per line
        env_set IMAGE_FOLDERS "$(printf '%s\n' $sum | sed "s/^$rel\$/$nrel/" | paste -sd' ' -)"
    fi
    rename_log renamed "$old" "$new"
    # For the Windows menu: which garden files changed, so it can update its own copy.
    { for f in "${code_hits[@]}"; do case "$f" in /*) ;; *) printf 'edit\t%s\n' "${f#./}" ;; esac; done
      case "$old" in /*) ;; site/*) printf 'move\t%s\t%s\n' "$old" "$new" ;; esac; } > logs/last-rename.txt

    # 5. Start again. Rust and garden-tool changes need a rebuild (a few minutes).
    printf '%s\n' "${code_hits[@]}" | has '^rust/' && rebuilt=site
    printf '%s\n' "${code_hits[@]}" | has '^beds/' && has_part beds && rebuilt="$rebuilt beds"
    if [ -n "$rebuilt" ]; then
        note "The website's code changed, so it is built again (a few minutes):"
        # shellcheck disable=SC2086  # the list of programs is meant to split into words
        run dc up -d --build $rebuilt || warn "the build stopped. The old copies are in $keep; rename it back (this menu, names swapped)."
    fi
    printf '%s\n' "${db_hits[@]}" | has '^junsgarden_wiki\.' && has_part wiki && dc up -d --force-recreate wiki >/dev/null 2>&1
    dc up -d >/dev/null
    wait_site || return 1
    # 6. Check: the website hands out the file (or a file inside the folder) under the new name.
    if [ "$key" != DATA_DIR ]; then
        if [ -d "$new" ]; then
            check=$(find "$new" -type f \( -name '*.webp' -o -name '*.jpg' -o -name '*.png' -o -name '*.html' -o -name '*.css' -o -name '*.js' \) -print -quit)
            [ -n "$check" ] || check=$(find "$new" -type f ! -name '.*' -print -quit)
        else check=$new; fi
        if [ -n "$check" ]; then
            check=${check#"$new"}; check=$nweb$check
            if curl -fsS -o /dev/null --max-time 10 "$(site_url)/$check"; then note "Checked: $(site_url)/$check answers."
            else warn "$(site_url)/$check does not answer. Look at it in the browser; to undo, rename it back."; fi
        fi
    fi
    if [ -d .git ] && [ "${#code_hits[@]}" -gt 0 ]; then
        git add -- "${code_hits[@]}" 2>/dev/null && git commit -q -m "Renamed $old to $new (menu M, 18)" -- "${code_hits[@]}" 2>/dev/null \
            && note "Saved in your git history (git show shows every changed line)."
    fi
    printf '%s\n' "${code_hits[@]}" | has '^android-app/' && note "The phone app's code changed too: build the app again (android-app/README.md) for phones."
    note "Done. Press Ctrl+Shift+R in the browser if a page still shows the old picture."
    note "To undo: choose this again and rename it back (new name -> old name)."
}

# After a restore or a photos download: a backup made BEFORE a rename brings the
# old name back. Offer to do those renames again, in the order they were made
# (the code already uses the new names; the files and the database follow).
redo_renames() {
    [ -s "$RENAME_LOG" ] || return 0
    local -a whens=() olds=() news=()
    local when what old new i j name
    while IFS=$'\t' read -r when what old new; do
        [ "$what" = renamed ] || continue
        whens+=("$when"); olds+=("$old"); news+=("$new")
    done < "$RENAME_LOG"
    for i in "${!olds[@]}"; do
        # Follow the later renames to today's name (A -> B, later B -> C: today it is C).
        name=${news[$i]}
        for ((j = i + 1; j < ${#olds[@]}; j++)); do [ "${olds[$j]}" = "$name" ] && name=${news[$j]}; done
        old=${olds[$i]}
        [ "$name" = "$old" ] && continue          # renamed and later renamed back: nothing to do
        [ -e "$old" ] || continue
        if [ -e "$name" ]; then
            warn "both $old and $name are here now (the old name came back with a backup or download)."
            note "Move what you want to keep from $old into $name, then delete $old."
            continue
        fi
        warn "$old has its old name again (it is called $name since ${whens[$i]})."
        confirm "Rename it again, with everything that uses it?" y && rename_item "$old" "$name" after-restore
    done
}

# Choice 2: move one of the garden's big folders (its line in .env changes too).
do_move_folder() {
    say "Move one of the garden's big folders to another place or disk"
    local keys=(SITE_DIR IMAGES_DIR DATA_DIR CACHE_DIR DB_DIR N8N_DIR AUTH_DIR BACKUP_DIR) i c key old new
    for i in "${!keys[@]}"; do printf '   %d) %-11s %s\n' $((i + 1)) "${keys[$i]}" "$(env_get "${keys[$i]}")"; done
    c=$(ask "Which one (0 = back)" 0); [ "$c" = 0 ] && return 0
    key=${keys[$((c - 1))]:-}; [ -n "$key" ] || { note "No such number."; return 0; }
    old=$(env_get "$key")
    note "Type the new place. Inside the garden folder, start with ./  (example ./my-photos)."
    note "On another disk, type the whole path (example /mnt/bigdisk/garden-photos). Tab completes names."
    new=$(ask_path "New place for $key (empty = back)")
    case "$new" in "") return 0 ;; /*|"~"*) new=$(tidy_path "$new") ;; *) new=./$(tidy_path "$new") ;; esac
    [ "$new" = "$old" ] && { note "Same as before; nothing to do."; return 0; }
    [ -e "$new" ] && { fail "$new already exists. Pick a name that is not used."; return 1; }
    case "$(realpath -m -- "$new")/" in "$(realpath -m -- "$old")/"*) fail "a folder cannot go inside itself."; return 1 ;; esac
    if [ "$key" != BACKUP_DIR ] && ! linux_disk "$new"; then
        fail "that place is on a $(disk_type "$new") disk. The garden's programs need a Linux disk (ext4): Windows and USB disks (NTFS, exFAT) cannot keep Linux owners and permissions. Backups can go there; this folder cannot."
        return 1
    fi
    note "The guides show where this folder was ($old). logs/renames.tsv will list the move."
    confirm "Stop the garden, move $old to $new, and start again?" y || return 0
    free_files "$old" || { note "Nothing changed."; dc up -d >/dev/null; return 0; }
    mkdir -p "$(dirname "$new")" 2>/dev/null || run sudo mkdir -p "$(dirname "$new")"
    if [ "$key" = DB_DIR ] || [ "$key" = AUTH_DIR ]; then run sudo mv "$old" "$new"; else run mv "$old" "$new" || run sudo mv "$old" "$new"; fi || { fail "the move failed; nothing was changed"; run dc up -d; return 1; }
    env_set "$key" "$new"
    rename_log moved "$old" "$new"
    run dc up -d >/dev/null
    [ "$key" = BACKUP_DIR ] && nightly_backup
    wait_site && note "Moved. The checklist (menu 5) confirms every folder is found."
}

# Choice 3: move the WHOLE garden folder (for example to a bigger disk), then fix
# what points at it: the nightly backup schedule, the desktop icon, settings that
# name the folder, and Docker's programs (they are made again from the new place).
move_garden() {
    local old=$HERE new parent before after k v
    explain "Move the whole garden folder" \
        "Moves the garden folder (the programs, your photos, the database, the backups inside it) to a new place, then fixes what points at it: the nightly backup schedule, the desktop icon and Docker's programs." \
        "For example to a bigger disk. A signpost (a link) can stay at the old place, so the Windows menu and your habits still find it." ""
    note "Now: $old"
    note "Type the new place, the whole path (example: /mnt/bigdisk/junsgarden). Tab completes names."
    new=$(tidy_path "$(ask_path "New place (empty = back)")")
    [ -n "$new" ] || return 0
    case "$new" in /*) ;; *) fail "type the whole path, starting with / (or ~ for your home folder)."; return 1 ;; esac
    # Moving back to where it was: the signpost left there is replaced by the garden itself.
    local back=''
    if [ -L "$new" ] && [ "$(readlink -f "$new")" = "$old" ]; then back=1
    elif [ -e "$new" ]; then fail "$new already exists. Pick a name that is not used."; return 1; fi
    parent=$(dirname "$new")
    # (The parent's real place plus the name: a signpost link there is not followed.)
    case "$(realpath -m -- "$parent")/$(basename "$new")/" in "$old/"*) fail "the garden folder cannot go inside itself."; return 1 ;; esac
    linux_disk "$parent" || { fail "that place is on a $(disk_type "$parent") disk. The garden needs a Linux disk (ext4): Windows and USB disks (NTFS, exFAT) cannot keep Linux owners and permissions."; return 1; }
    if [ "$(stat -c %d "$old")" != "$(stat -c %d "$(realpath -m -- "$parent" | sed 's#/*$##')" 2>/dev/null || echo x)" ]; then
        local need free
        need=$(sudo du -sk "$old" | cut -f1)
        k=$parent; while [ ! -d "$k" ]; do k=$(dirname "$k"); done
        free=$(df -Pk "$k" | awk 'NR == 2 {print $4}')
        [ "$need" -lt "$free" ] || { fail "not enough space there: the garden needs $((need / 1024)) MB, the disk has $((free / 1024)) MB free."; return 1; }
        note "Another disk: everything is copied there ($((need / 1024)) MB), then removed here. This takes a while."
    fi
    if grep -qi microsoft /proc/version 2>/dev/null; then
        warn "this is Windows' own Ubuntu. The Windows menu looks for the garden in ~/$REMOTE_DIR: keep the signpost (next question) so it still finds it."
    fi
    note "The guides show the garden folder in its first place. logs/renames.tsv will list the move."
    confirm "Stop the garden, move it to $new, and start it again there?" y || return 0
    free_files "$old" || { note "Nothing changed."; dc up -d >/dev/null; return 0; }
    run dc down >/dev/null 2>&1
    before=$(sudo find "$old" | wc -l)
    [ -d "$parent" ] || mkdir -p "$parent" 2>/dev/null || run sudo mkdir -p "$parent"
    cd / || return 1
    [ -n "$back" ] && { rm -f "$new" 2>/dev/null || sudo rm -f "$new"; }
    if ! run sudo mv -- "$old" "$new"; then
        cd "$old" 2>/dev/null || true; fail "the move failed; the garden stays where it was"; dc up -d >/dev/null; return 1
    fi
    HERE=$new; cd "$HERE" || return 1
    after=$(sudo find "$new" | wc -l)
    if [ "$before" = "$after" ]; then note "All $after files and folders arrived."
    else warn "$before files and folders before, $after after. Look before deleting anything."; fi
    # Settings that name the old place in full move along with it.
    for k in SITE_DIR IMAGES_DIR DATA_DIR CACHE_DIR DB_DIR N8N_DIR AUTH_DIR BACKUP_DIR; do
        v=$(env_get "$k"); case "$v" in "$old"/*) env_set "$k" "$new/${v#"$old"/}" ;; esac
    done
    # The schedules (nightly backup, automatic check-up) start in the new place.
    crontab -l 2>/dev/null | OLD=$old NEW=$new perl -pe 's/\Qcd "$ENV{OLD}"\E/cd "$ENV{NEW}"/g' | crontab -
    [ -f "$HOME/.local/share/applications/junsgarden.desktop" ] && make_icon
    rename_log moved "$old" "$new"
    if confirm "Leave a signpost at the old place (a link to the new one; the Windows menu needs it)?" y; then
        ln -s "$new" "$old" 2>/dev/null || run sudo ln -s "$new" "$old"
        note "Signpost: $old -> $new"
    fi
    run dc up -d >/dev/null
    wait_site || return 1
    note "Done. The garden now lives in $new."
    note "This terminal is still in the old place: when the menu closes, type:  cd $new"
}

# Settings 9: new random passwords for the database's own accounts.
do_renew_db_passwords() {
    explain "Renew the internal database passwords" \
        "Makes new long random passwords for the database's own accounts, changes them INSIDE the database, saves them in .env and restarts the programs." \
        "Good practice once a year, or if .env was ever shown to anybody." ""
    confirm "Renew them now?" y || return 0
    local newroot newapp
    newroot=$(openssl rand -hex 24); newapp=$(openssl rand -hex 24)
    # The SQL travels on standard input, so no password appears in a command line.
    # shellcheck disable=SC2016
    printf "ALTER USER 'dbadmin'@'%%' IDENTIFIED BY '%s'; ALTER USER 'root'@'%%' IDENTIFIED BY '%s'; ALTER USER 'root'@'localhost' IDENTIFIED BY '%s'; FLUSH PRIVILEGES;\n" \
        "$newapp" "$newroot" "$newroot" |
        dk exec -i garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" exec mariadb -uroot' ||
        { fail "the database did not accept the change; nothing was saved"; return 1; }
    env_set DB_PASSWORD "$newapp"; env_set DB_ROOT_PASSWORD "$newroot"
    unset newroot newapp
    run dc up -d --force-recreate db site >/dev/null && dc up -d >/dev/null
    wait_site && note "Renewed. The new passwords are only in .env."
}

# The sign-in page reads its user list when it starts: restart it after any
# change to people or passwords, so the change counts straight away.
reload_signin() { has_part gate && dc restart authelia >/dev/null 2>&1 && wait_signin; return 0; }
# The sign-in page needs a few seconds after (re)starting: wait until it answers.
wait_signin() {
    local i
    for i in $(seq 1 30); do
        curl -fs -o /dev/null --max-time 2 http://127.0.0.1:9091/auth/api/health && return 0
        sleep 2
    done
    warn "the sign-in page did not answer within a minute: docker logs garden-auth"
    return 1
}

# The Docker service names that belong to each optional part.
part_services() {   # the compose service names that belong to a part
    case "$1" in beds) echo beds ;; wiki) echo "wiki wiki-db-init" ;; n8n) echo n8n ;; gate) echo authelia ;; llm) echo litellm ;; esac
}

# Settings 10 / More 9: connect (or disconnect) a local AI computer running Ollama.
do_ai() {
    explain "Local AI on another computer (optional, advanced)" \
        "Ollama runs AI models on a computer with a strong graphics card. The garden's AI boxes can ask it instead of a cloud service. NOT on this laptop: it is too small." \
        "Private and free once set up. Guide: START-HERE/15-local-ai.md" \
        "https://docs.ollama.com/faq"
    local ip model url
    ip=$(ask "IP address of the AI computer (empty = turn local AI off)" "")
    if [ -z "$ip" ]; then env_set OLLAMA_URL ""; env_set OLLAMA_MODEL ""; run dc up -d site; note "Local AI is off."; return 0; fi
    url="http://$ip:11434"
    if curl -fsS --max-time 5 "$url/api/tags" >/tmp/garden-models.json; then
        note "It answers. Models it has: $(grep -o '"name":"[^"]*"' /tmp/garden-models.json | cut -d'"' -f4 | tr '\n' ' ')"
    else
        fail "no answer from $url. On that computer Ollama must listen on the network (guide 15, step 3)."; return 1
    fi
    model=$(ask "Which model should the garden use" "$(grep -o '"name":"[^"]*"' /tmp/garden-models.json | head -1 | cut -d'"' -f4)")
    env_set OLLAMA_URL "$url/api/generate"; env_set OLLAMA_MODEL "$model"
    run dc up -d site
    note "Done. The garden's AI boxes now ask $model on $ip."
}

# More 2: this computer's IP and MAC address, and how to reserve the IP in the router.
do_network() {
    say "This computer on your home network"
    local dev ip mac gw saved
    dev=$(ip route show default 2>/dev/null | awk '{print $5; exit}')
    gw=$(ip route show default 2>/dev/null | awk '{print $3; exit}')
    ip=$(ip -4 -o addr show "$dev" 2>/dev/null | awk '{print $4}' | cut -d/ -f1 | head -1)
    mac=$(cat "/sys/class/net/$dev/address" 2>/dev/null)
    saved=$(env_get LAPTOP_IP || true)
    note "Network card:      ${dev:-?}"
    note "IP address:        ${ip:-?}"
    note "MAC address:       ${mac:-?}   (the card's fixed ID; the router uses it)"
    note "Router address:    ${gw:-?}"
    # Inside Windows' own Ubuntu (WSL) these are WSL's private inside numbers:
    # the router and other devices cannot reach them. Say so, plainly.
    if grep -qi microsoft /proc/version 2>/dev/null; then
        warn "this is Windows' own Ubuntu (WSL). The numbers above are WSL's private inside address."
        note "The router and phones cannot reach it, so do not reserve or forward it. On this PC the garden"
        note "is for using on this PC. To show it to other devices or put it online, use a separate garden"
        note "computer (an Ubuntu laptop or a Pi): guide 00b. More: https://learn.microsoft.com/windows/wsl/networking"
        return 0
    fi
    Lblock <<EOF

   RESERVE THIS ADDRESS so it never changes (the router's port forwarding
   and your SSH both depend on it):
   1. On a phone or computer at home, open  http://${gw:-your-router-address}
      (or the router's phone app: Google Home, Deco, Nighthawk, Spectrum...).
   2. Log in with the router's admin password (often on a sticker on the router).
   3. Find "DHCP reservation", "Address reservation" or "Static / Fixed IP".
   4. Add: MAC ${mac:-?}  ->  IP ${ip:-?}  and save.
   5. Restart this laptop once and run this again: the IP should be the same.
   Full steps: START-HERE/14-reserve-your-ip.md
EOF
    if [ -n "$ip" ] && [ "$saved" != "$ip" ] && confirm "Have you reserved $ip for this laptop in the router?" n; then
        env_set LAPTOP_IP "$ip"; note "Saved LAPTOP_IP=$ip. The checklist will warn you if it ever changes."
    fi
}

# More 8: start Adminer, a web page for looking inside the database.
do_dbweb() {
    explain "Adminer (database viewer)" \
        "A one-page web program for looking inside the database: tables, rows, searches." \
        "Easier than typing SQL. It runs only while you need it, only on this laptop." \
        "https://www.adminer.org"
    run dc --profile dbweb up -d adminer || return 1
    note "Open http://localhost:8082"
    note "System: MySQL   Server: db   Username: dbadmin   Database: juns_garden_db"
    note "Password: the DB_PASSWORD line in your .env (open it with: nano .env)"
    note "Stop it when you are done:  bash garden.sh dbweb-stop"
}

# A database command line (SQL) inside the database container.
do_db() {
    say "Database command line (type SQL, end each line with ;  leave with: exit)"
    note "Try:  SHOW TABLES;   SELECT COUNT(*) FROM data_source;"
    # The single quotes are on purpose: the password is read INSIDE the
    # database container, so it never appears on this computer.
    # shellcheck disable=SC2016
    dk exec -it garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" exec mariadb -uroot juns_garden_db'
}

# More 7: rebuild the website after you edited Rust files or templates.
do_rebuild() {
    say "Rebuilding the website from the Rust files"
    note "Only the files you changed are compiled again, so this is quicker than the first time."
    run dc up -d --build site || { fail "the build failed. The red lines above say which file and line. Undo with: git checkout -- FILE"; return 1; }
    wait_site
    note "If the page looks old, press Ctrl+Shift+R in the browser."
}

# ---------- people: who can sign in, and taking access away ------------------------
me() { env_get GARDEN_USERS | cut -d, -f1 | cut -d= -f1; }

# Menu U: who can sign in; add, remove, guests, passwords, SSH keys, lock down.
do_users() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    local c name email
    while true; do
        say "Users and access. You ($(me)) are the administrator."
        Lblock <<'EOF'
   1) Who can sign in right now
   2) Add a person who may EDIT (website + garden tools)
   3) Add a GUEST (an account on the sign-in page only; sees nothing private)
   4) Remove a person (takes away every login they have)
   5) Change someone's password (yours too)
   6) Sign everybody out now
   7) SSH keys: which computers can open this computer's terminal
   8) LOCK DOWN: only me, new passwords and secrets, everyone signed out
   0) Back
EOF
        c=$(ask "Number" 0)
        case "$c" in
            0) return 0 ;;
            1) tools tools list-users
               note "Garden tools (/beds) allowed emails: $(env_get BEDS_ALLOWED_EMAILS)" ;;
            2) name=$(ask "Their first name (letters only)" "")
               case "$name" in *[!A-Za-z]*|"") note "Letters only."; continue ;; esac
               email=$(ask "Their email address" "")
               case "$email" in *@*.*) ;; *) note "That does not look like an email."; continue ;; esac
               env_set GARDEN_USERS "$(env_get GARDEN_USERS),$name=$email"
               env_set GARDEN_UPLOADER_NAMES "$(env_get GARDEN_UPLOADER_NAMES),$name"
               env_set BEDS_ALLOWED_EMAILS "$(env_get BEDS_ALLOWED_EMAILS),$email"
               run dc up -d site beds >/dev/null 2>&1 || run dc up -d site >/dev/null
               wait_site >/dev/null
               note "Now type a first password for $name (tell them in person; they can change it later):"
               do_password "$name" ;;
            3) name=$(ask "Guest's first name (letters only)" "")
               email=$(ask "Guest's email (optional)" "")
               if [ -t 0 ]; then tools_tty tools add-guest "$name" "$email"; else tools tools add-guest "$name" "$email"; fi
               reload_signin ;;
            4) name=$(ask "Name of the person to remove" "")
               [ -n "$name" ] || continue
               [ "$name" = "$(me)" ] && { note "That is you, the administrator. Not removed."; continue; }
               confirm "Remove every login $name has?" y || continue
               remove_person "$name"
               note "Done. Also check: menu U, 7 (their SSH key, if any)." ;;
            5) do_password ;;
            6) tools tools logout-all; env_set AUTHELIA_SESSION_SECRET "$(openssl rand -hex 32)"
               has_part gate && run dc up -d authelia >/dev/null
               note "Everyone must sign in again." ;;
            7) do_ssh_keys ;;
            8) do_lockdown ;;
            *) note "Type one of the numbers." ;;
        esac
    done
}

# Take a person out of every list and remove all their logins.
remove_person() {   # take NAME out of every list, then remove their logins
    local name=$1 email users allowed
    email=$(env_get GARDEN_USERS | tr ',' '\n' | grep -i "^$name=" | cut -d= -f2)
    users=$(env_get GARDEN_USERS | tr ',' '\n' | grep -vi "^$name=" | paste -sd, -)
    env_set GARDEN_USERS "$users"
    env_set GARDEN_UPLOADER_NAMES "$(env_get GARDEN_UPLOADER_NAMES | tr ',' '\n' | grep -vix "$name" | paste -sd, -)"
    if [ -n "$email" ]; then
        allowed=$(env_get BEDS_ALLOWED_EMAILS | tr ',' '\n' | grep -vix "$email" | paste -sd, -)
        env_set BEDS_ALLOWED_EMAILS "$allowed"
    fi
    tools tools remove-user "$name"
    reload_signin
    run dc up -d >/dev/null
}

# List the SSH keys that can open this computer's terminal, and remove one.
do_ssh_keys() {
    local f="$HOME/.ssh/authorized_keys" i line c
    say "Computers that can open this computer's terminal with a key"
    note "Each line is one other computer's 'public key'. The last word is its label,"
    note "usually user@computer. Remove any you do not recognise."
    note "(Your own password still works for SSH from your home network.)"
    [ -s "$f" ] || { note "None. Only a password can open SSH here."; return 0; }
    mapfile -t KEYS < "$f"
    for i in "${!KEYS[@]}"; do
        line=${KEYS[$i]}
        printf '   %d) %s ... %s\n' $((i + 1)) "$(echo "$line" | cut -d' ' -f1)" "$(echo "$line" | awk '{print $NF}')"
    done
    c=$(ask "Number to remove (0 = keep them all)" 0)
    case "$c" in 0|"") return 0 ;; *[!0-9]*) note "Type a number."; return 0 ;; esac
    [ -n "${KEYS[$((c - 1))]:-}" ] || { note "No such number."; return 0; }
    cp "$f" "$f.bak"
    grep -vxF -- "${KEYS[$((c - 1))]}" "$f.bak" > "$f"
    chmod 600 "$f"
    note "Removed. (A copy of the old list is in $f.bak)"
}

# Users 8: only you remain; new passwords and secrets; everyone signed out.
do_lockdown() {
    say "LOCK DOWN: you, and only you"
    Lblock <<EOF
   This will:
   1. remove every website and sign-in page login except yours ($(me))
   2. let only your email into the garden tools
   3. make new database passwords and new sign-in secrets
      (everyone, including you, is signed out)
   4. ask you for a NEW password
   5. show the SSH keys so you can remove any you do not recognise
   Afterwards, also do these by hand (only you can):
   - change your Ubuntu password:          passwd
   - change the router's admin password    (on the router's page)
   - Gmail: remove old app passwords at    https://myaccount.google.com/apppasswords
   - No-IP: change your No-IP password at  https://my.noip.com
EOF
    confirm "Lock down now?" n || return 0
    local mine pair email
    mine=$(me)
    email=$(env_get GARDEN_USERS | cut -d, -f1 | cut -d= -f2)
    for pair in $(env_get GARDEN_USERS | tr ',' ' '); do
        [ "${pair%%=*}" = "$mine" ] || remove_person "${pair%%=*}"
    done
    env_set GARDEN_USERS "$mine=$email"; env_set GARDEN_UPLOADER_NAMES "$mine"; env_set BEDS_ALLOWED_EMAILS "$email"
    # Guests and anyone else on the sign-in page.
    if [ -f "$(env_get AUTH_DIR)/users.yml" ]; then
        local u
        # shellcheck disable=SC2013  # user names are single words
        for u in $(awk '/^  [^ ]/{u=$1; sub(":","",u); print u}' "$(env_get AUTH_DIR)/users.yml"); do
            [ "$u" = "$(echo "$mine" | tr '[:upper:]' '[:lower:]')" ] || tools tools remove-user "$u" 2>/dev/null || true
        done
    fi
    tools tools logout-all
    env_set AUTHELIA_SESSION_SECRET "$(openssl rand -hex 32)"
    env_set AUTHELIA_JWT_SECRET "$(openssl rand -hex 32)"
    do_renew_db_passwords
    do_password "$mine"
    do_ssh_keys
    say "Locked down. Only $mine can sign in, with the new password."
}

# A look at exactly what came in the zip from the author, folder by folder, and a
# check that no file has been changed since (fingerprints).
do_whats_from_author() {
    say "What came in the zip from the author"
    Lblock <<'EOF'
   Folder / file            What it is
   site/                    the website's style, scripts, page cards, the phone app download
   rust/                    the website's code (Rust), with comments on almost every line
   beds/                    the garden tools' code (Python)
   docker/, docker-compose.yml   how each program is built and started
   garden.sh, Start-Here-*  this setup program and its launchers
   windows/                 the Windows menu
   START-HERE/, lab/, docs/ guides, practice scripts, notes on how the site was built
   db/schema.sql            the shape of the database (no data)
   backups/                 YOUR garden: plants, photos, comments, wiki pages
   Everything else here (programs, databases, certificates) is downloaded
   from the official places listed in WHAT-GETS-INSTALLED.md, or made on
   this computer during the install.
EOF
    local d
    for d in site rust beds docker START-HERE lab docs windows backups; do
        [ -d "$d" ] && printf '   %-11s %6s files  %s\n' "$d/" "$(find "$d" -type f | wc -l)" "$(du -sh "$d" | cut -f1)"
    done
    if [ -f FILES-FROM-AUTHOR.txt ] && confirm "Check every file's fingerprint against the list made when the zip was built?" y; then
        local changed
        changed=$(sha256sum -c --quiet FILES-FROM-AUTHOR.txt 2>/dev/null | grep -v ': OK$' || true)
        if [ -z "$changed" ]; then note "Every file is exactly as it was in the zip."
        else note "These files are different from the zip (changed by you, or by the setup):"; while IFS= read -r d; do note "   $d"; done <<<"$changed"; fi
    fi
    if confirm "Open this folder in the Files app to look around?" n; then xdg-open "$HERE" >/dev/null 2>&1 & fi
}

# More 12: remove what the garden installed (two levels: programs only, or everything).
do_uninstall() {
    say "Uninstall"
    Lblock <<'EOF'
   1) Remove the PROGRAMS, keep my garden
      Stops and removes the garden's containers and images, the nightly
      backup, the NAS connection, the desktop icon, the front door (Caddy and
      the No-IP updater) and the firewall's web doors (80 and 443).
      Keeps: this folder, your photos and database (local/), your backups.
      Install again any time with menu 1.

   2) Remove EVERYTHING (start completely fresh)
      All of 1, PLUS your photos, database, wiki, settings (.env), logs and
      backups on THIS computer. A final backup is made first and you choose
      where to keep it. You must type DELETE to confirm.
      Docker itself stays (other programs may use it); the command to remove
      it is shown at the end.

   0) Back
EOF
    local c; c=$(ask "Type 1, 2 or 0" 0)
    case "$c" in 1) uninstall_programs ;; 2) uninstall_everything ;; *) return 0 ;; esac
}

# Level 1: everything the setup added to Ubuntu, but not your garden's data.
uninstall_programs() {   # uninstall_programs [noask]
    [ "${1:-}" = noask ] || confirm "Remove the garden's programs (your garden's data stays)?" n || return 0
    say "Stopping and removing the garden's containers and images"
    run dc --profile tools --profile dbweb --profile wiki-setup --profile beds --profile wiki \
        --profile n8n --profile gate --profile llm down --rmi local --remove-orphans
    say "Removing the schedule, the icon and the NAS connection"
    ( crontab -l 2>/dev/null | grep -v -e 'garden.sh backup' -e 'docker/backup.sh' -e 'garden.sh checkup' ) | crontab -
    rm -f "$HOME/.local/share/applications/junsgarden.desktop" "$(xdg-user-dir DESKTOP 2>/dev/null || echo "$HOME/Desktop")/junsgarden.desktop"
    local n
    for n in $NAS_SLOTS; do
        if grep -q " $(nas_mount_of "$n") cifs " /etc/fstab 2>/dev/null; then
            nas_disconnect "$n"
            note "Network place $n disconnected (the files on it stay)."
        fi
    done
    if command -v caddy >/dev/null 2>&1; then
        say "Removing the front door (Caddy)"
        run sudo systemctl disable --now caddy
        run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 remove -y -qq caddy >/dev/null
        run sudo rm -f /etc/apt/sources.list.d/caddy-stable.list
    fi
    if systemctl list-unit-files 2>/dev/null | has '^noip-duc.service'; then
        say "Removing the No-IP updater and its key"
        run sudo systemctl disable --now noip-duc
        run sudo DEBIAN_FRONTEND=noninteractive apt-get -o DPkg::Lock::Timeout=900 remove -y -qq noip-duc >/dev/null
        run sudo rm -f /etc/default/noip-duc
    fi
    if crontab -l 2>/dev/null | has duckdns; then
        ( crontab -l 2>/dev/null | grep -v duckdns ) | crontab -
        rm -rf "$HOME/.config/garden-duckdns"
        note "DuckDNS updater removed."
    fi
    say "Closing the firewall's web doors (80 and 443)"
    sudo ufw delete allow 80/tcp >/dev/null 2>&1; sudo ufw delete allow 443/tcp >/dev/null 2>&1
    sudo sed -i '/ garden\.test$/d' /etc/hosts 2>/dev/null
    rm -f .garden-built
    say "Done. Your garden's data and backups are still in $HERE"
    note "Not removed (you decide): Docker, the SSH server, git, the 4 GB swap file"
    note "  Docker:     https://docs.docker.com/engine/install/ubuntu/#uninstall-docker-engine"
    note "  Swap file:  sudo swapoff /swapfile-garden && sudo rm /swapfile-garden  (and remove its line in /etc/fstab)"
}

# Level 2: everything, including the garden's data on this computer.
uninstall_everything() {
    say "Remove EVERYTHING"
    note "First, a final backup, so nothing is lost by accident."
    # </dev/null: the backup must not swallow the answers typed for the questions below.
    do_backup final </dev/null || { fail "the final backup did not work, so nothing was removed"; return 1; }
    local f; f=$(newest_backup_path)
    note "Final backup: $f"
    note "Copy it somewhere safe NOW (USB stick, NAS, Google Drive). This folder is about to be emptied."
    if [ -n "$(env_get BACKUP_NAS_DIR)" ] && [ -d "$(env_get BACKUP_NAS_DIR)" ]; then
        cp "$f" "$(env_get BACKUP_NAS_DIR)/" && note "A copy is also on the NAS: $(env_get BACKUP_NAS_DIR)"
    fi
    local keep; keep=$(ask "Folder to move the final backup to before deleting (empty = $HOME)" "$HOME")
    mkdir -p "$keep" && mv "$f" "$keep/" && note "Kept: $keep/$(basename "$f")"
    local word; read -r -p "$(Lt "Type DELETE (in capitals) to remove everything, or anything else to stop"): " word
    [ "$word" = DELETE ] || { note "Stopped. Nothing was deleted."; return 0; }
    uninstall_programs_noask
    say "Deleting the garden's data on this computer"
    # The database and sign-in folders belong to the containers' users, so sudo.
    run sudo rm -rf ./local ./backups ./logs ./help .env .garden-mode .garden-lang .garden-built \
        CHECKLIST.txt checklist-history.log .garden-remote
    rclone config delete garden-gdrive >/dev/null 2>&1 || true
    # Only the garden's own leftover volumes (project "junsgarden"), never other programs' ones.
    sudo docker volume ls -q --filter label=com.docker.compose.project=junsgarden | xargs -r sudo docker volume rm >/dev/null 2>&1 || true
    say "Everything the garden installed is gone. Left: the program files in $HERE"
    note "and your final backup in $keep. To remove the program files too:"
    note "  cd ~ && rm -rf \"$HERE\""
}

# The same as level 1, without asking again (used by level 2).
# (Not ASSUME_YES: that answers every question with its default, and the
# default for "remove the programs?" is no.)
uninstall_programs_noask() { uninstall_programs noask; }

# ---------- the checklist -------------------------------------------------------------
do_check() {
    local out=CHECKLIST.txt P=0 W=0 F=0
    ok()   { printf 'PASS  %s\n' "$1"; P=$((P + 1)); }
    wn()   { printf 'WARN  %s\n        -> %s\n' "$1" "$2"; W=$((W + 1)); }
    bad()  { printf 'FAIL  %s\n        -> %s\n' "$1" "$2"; F=$((F + 1)); }
    info() { printf 'INFO  %s\n' "$1"; }
    listening() { ss -ltnH 2>/dev/null | awk '{print $4}' | grep -E "[:.]$1\$"; }
    cstate() { dk inspect -f '{{.State.Status}}{{if .State.Health}} ({{.State.Health.Status}}){{end}}' "$1" 2>/dev/null; }
    container() {   # container NAME what
        local s; s=$(cstate "$1")
        case "$s" in running*) ok "$1 ($2): $s" ;; "") bad "$1 ($2) does not exist" "bash garden.sh start" ;; *) bad "$1 ($2): $s" "bash garden.sh logs" ;; esac
    }
    {
    echo "JUN'S GARDEN CHECKLIST"
    echo "Last run: $(date '+%A %Y-%m-%d %H:%M') by $USER on $(hostname)"
    echo "Folder:   $HERE"
    echo "Parts on: core$( [ -n "$(env_get COMPOSE_PROFILES)" ] && echo ",$(env_get COMPOSE_PROFILES)")"
    echo "Run it again any time:  bash garden.sh check"
    echo

    echo "== 1. Programs that must be installed"
    local p
    for p in docker git curl openssl ssh; do
        if command -v "$p" >/dev/null 2>&1; then ok "$p is installed"; else bad "$p is missing" "bash garden.sh install"; fi
    done
    if dk compose version >/dev/null 2>&1; then ok "docker compose is installed"; else bad "docker compose is missing" "bash garden.sh install"; fi
    if [ "$(docker_mode)" = engine ] && docker_desktop_shared; then wn "Docker Desktop (Windows) shares its Docker with this Ubuntu" "Docker Desktop > Settings > Resources > WSL integration: switch this Ubuntu off (guide 25)"; fi
    if [ -x /usr/sbin/sshd ]; then ok "SSH server is installed"; else wn "SSH server is not installed" "sudo apt install -y openssh-server"; fi

    echo; echo "== 2. Services that must be running"
    if [ "$(docker_mode)" = desktop ]; then
        if dk info 2>/dev/null | has 'Docker Desktop'; then ok "Docker Desktop is running and connected"; else bad "Docker Desktop does not answer" "start Docker Desktop on Windows; Settings > Resources > WSL integration: this Ubuntu on"; fi
    elif systemctl is-active --quiet docker 2>/dev/null; then ok "Docker service is running"; else bad "Docker is not running" "sudo systemctl start docker"; fi
    if systemctl is-active --quiet ssh 2>/dev/null; then ok "SSH service is running"; else wn "SSH service is not running" "sudo systemctl enable --now ssh"; fi
    container garden-db "database"
    container garden-site "website"
    has_part beds && container junsgarden-ingest "garden tools"
    has_part wiki && container garden-wiki "wiki"
    has_part n8n  && container garden-n8n "automations"
    has_part gate && container garden-auth "sign-in page"
    local h; h=$(curl -fsS --max-time 5 "$(site_url)/healthz" 2>/dev/null)
    if echo "$h" | has '"status":"ok"'; then ok "Website answers: $h"; else bad "Website does not answer on $(site_url)" "bash garden.sh logs"; fi
    if has_part beds; then
        if curl -fsS --max-time 5 http://127.0.0.1:8025/healthz >/dev/null 2>&1; then ok "Garden tools answer on 127.0.0.1:8025"; else bad "Garden tools do not answer" "docker logs junsgarden-ingest"; fi
    fi
    if has_part wiki; then
        if curl -fsS --max-time 5 -o /dev/null http://127.0.0.1:3000/; then ok "Wiki answers on 127.0.0.1:3000"; else bad "Wiki does not answer" "docker logs garden-wiki"; fi
    fi

    echo; echo "== 3. Ports (the doors into this computer)"
    local sp name; sp=$(site_port); name=$(env_get PUBLIC_NAME)
    if listening 22 >/dev/null; then ok "22 SSH: listening (firewall allows it from home only)"; else wn "22 SSH: not listening" "sudo systemctl enable --now ssh"; fi
    if listening "$sp" | has '^127\.0\.0\.1'; then ok "$sp website: private to this computer (127.0.0.1)"
    elif listening "$sp" >/dev/null; then wn "$sp website: open to the network" "keep 127.0.0.1: in docker-compose.yml; Caddy is the front door"
    else bad "$sp website: nothing is listening" "bash garden.sh start"; fi
    if listening 3306 >/dev/null; then wn "3306 database: open on this computer" "it should stay inside Docker only"; else ok "3306 database: closed (only the programs inside Docker reach it)"; fi
    local port
    for port in 8025 3000 9091 5678 8082; do
        if listening "$port" >/dev/null && ! listening "$port" | has -E '^(127\.0\.0\.1|\[::1\])'; then wn "$port is open to the network" "it should be 127.0.0.1 only"; fi
    done
    if [ -n "$name" ]; then
        if systemctl is-active --quiet caddy 2>/dev/null && listening 443 >/dev/null; then ok "80/443 Caddy is answering for $name"; else bad "Caddy is not answering on 443" "sudo systemctl status caddy"; fi
        info "Router: forward ONLY TCP 80 and TCP 443 to ${name:+this laptop }$(env_get LAPTOP_IP || true). Nothing else."
    else info "80/443: not used yet (the front door step: bash garden.sh public)"; fi
    local other
    other=$(ss -ltnH 2>/dev/null | awk '{print $4}' | grep -vE '^(127\.|\[::1\]|::1)' | grep -vE "[:.](22|80|443)\$" | sort -u | tr '\n' ' ')
    [ -z "$other" ] || info "Other open doors (check you know what they are): $other"
    local fw; fw=$(sudo -n ufw status 2>/dev/null)
    if echo "$fw" | has 'Status: active'; then
        ok "Firewall (ufw) is on"
        info "Firewall allows: $(echo "$fw" | awk 'NR>4 && $1!="" {print $1}' | sort -u | tr '\n' ' ')"
    elif command -v ufw >/dev/null 2>&1; then wn "Cannot confirm the firewall is on" "sudo ufw status   (should say: active)"
    else wn "No firewall program" "bash garden.sh install"; fi

    echo; echo "== 4. Folders (from .env)"
    if [ -f .env ]; then
        if [ "$(stat -c %a .env)" = 600 ]; then ok ".env is private (mode 600)"; else wn ".env can be read by others" "chmod 600 .env"; fi
        local k d
        for k in SITE_DIR IMAGES_DIR DATA_DIR CACHE_DIR DB_DIR BACKUP_DIR; do
            d=$(env_get "$k")
            if [ -d "$d" ]; then ok "$k=$d ($(find "$d" -type f 2>/dev/null | wc -l) files)"; else bad "$k=$d does not exist" "rename the folder back, or fix the line in .env (guide 13)"; fi
        done
        [ -s "$RENAME_LOG" ] && info "Renamed or moved: $(($(wc -l < "$RENAME_LOG") - 1)) change(s), listed in $RENAME_LOG. The guides show the first names."
        garden_is_off && info "The garden is switched OFF ($(head -1 "$OFF_MARK")). On again: bash garden.sh on"
    else bad "No .env settings file" "bash garden.sh install"; fi

    echo; echo "== 5. Backups"
    local nb age
    nb=$(newest_backup)
    if [ -n "$nb" ]; then
        age=$(( ( $(date +%s) - $(stat -c %Y "$(env_get BACKUP_DIR)/$nb") ) / 86400 ))
        if [ "$age" -le 2 ]; then ok "Newest backup: $nb ($age days old)"; else wn "Newest backup is $age days old: $nb" "bash garden.sh backup"; fi
    else wn "No backups yet" "bash garden.sh backup"; fi
    if crontab -l 2>/dev/null | has 'garden.sh backup'; then ok "Nightly backup is scheduled (2:30 am)"; else wn "No nightly backup" "bash garden.sh install"; fi

    echo; echo "== 6. Network and web name"
    local dev ip gw mac saved
    dev=$(ip route show default 2>/dev/null | awk '{print $5; exit}')
    gw=$(ip route show default 2>/dev/null | awk '{print $3; exit}')
    ip=$(ip -4 -o addr show "$dev" 2>/dev/null | awk '{print $4}' | cut -d/ -f1 | head -1)
    mac=$(cat "/sys/class/net/$dev/address" 2>/dev/null)
    saved=$(env_get LAPTOP_IP || true)
    info "IP address ${ip:-?}  MAC ${mac:-?}  router ${gw:-?}"
    if [ -z "$saved" ]; then wn "Reserved address not confirmed yet" "bash garden.sh network"
    elif [ "$saved" = "$ip" ]; then ok "Address $ip matches the one reserved in the router"
    else bad "Address changed: now $ip, reserved $saved" "fix the reservation in the router (guide 14), or port forwarding breaks"; fi
    if [ -n "$name" ] && [ "${name##*.}" != test ]; then
        if systemctl is-active --quiet noip-duc 2>/dev/null; then ok "No-IP updater is running"; else bad "No-IP updater is not running" "sudo systemctl status noip-duc"; fi
        local dns pub
        dns=$(getent hosts "$name" | awk '{print $1; exit}'); pub=$(curl -fsS --max-time 5 https://api.ipify.org 2>/dev/null)
        if [ -n "$dns" ] && [ "$dns" = "$pub" ]; then ok "$name points at this home ($pub)"; else wn "$name points at ${dns:-nothing}, this home is ${pub:-?}" "wait 5 minutes, then: sudo systemctl status noip-duc"; fi
    fi
    local pct; pct=$(df --output=pcent / | tail -1 | tr -dc '0-9')
    if [ "${pct:-100}" -lt 85 ]; then ok "Disk is ${pct}% full"; else bad "Disk is ${pct}% full" "copy old backups to a USB stick and delete them, then: bash garden.sh cleanup"; fi

    echo
    echo "RESULT: $P pass, $W warn, $F fail   (PASS good, WARN fix soon, FAIL fix first)"
    } | tee "$out"
    echo "$(date '+%Y-%m-%d %H:%M') $(tail -1 "$out")" >> checklist-history.log
    note "Saved to $HERE/$out"
    grep -q '^FAIL' "$out" && return 1 || return 0
}

# ---------- the menu -------------------------------------------------------------------
banner() {
    clear 2>/dev/null || true
    printf '\033[1;32m  ==============================================\n     Jun'"'"'s Garden%s\n  ==============================================\033[0m\n' "${1:+  -  $1}"
    echo "  $(Lt "This computer:") $(hostname)   $(hostname -I 2>/dev/null | awk '{print $1}')"
    if [ -f .env ]; then
        echo "  $(Lt "Parts on:") core$( [ -n "$(env_get COMPOSE_PROFILES)" ] && echo ",$(env_get COMPOSE_PROFILES)")   $(Lt "Website:") $(site_url)"
    else
        L "  Not installed yet: choose 1."
    fi
}

# The main menu (shown when you run garden.sh without a job name).
menu() {
    while true; do
        banner
        Lblock <<'EOF'

   1) INSTALL  (first time, or carry on, or repair)

   2) Start the garden          3) Stop the garden
   4) Open the website          5) Checklist: is everything OK?
   6) Backups: back up, NAS, Google Drive   7) Restore a backup
   8) Settings: names, emails, passwords, keys, folders
   9) Parts: garden tools, wiki, n8n (on / off)
   U) Users and access: add, remove, guests, lock down
   K) API keys: paste, test or replace (Gemini, Gmail, ...)
   A) My addresses: every web address, and which need signing in

   H) Help file for someone helping you (no passwords in it)
   M) More (advanced)        L) Language: English / 한국어 / both
   0) Quit
EOF
        local c; echo; read -r -p "  $(Lt "Type a number or letter, then press Enter"): " c || { echo; exit 0; }   # no more input (window closed): stop, do not loop
        case "$c" in
            1) do_install ;;
            2) if garden_is_off; then power_on; else run dc up -d && wait_site; fi ;;
            3) run dc stop && note "Stopped. Nothing was deleted." ;;
            4) xdg-open "$(site_url)" >/dev/null 2>&1 & note "Opening $(site_url)" ;;
            5) do_check ;;
            6) do_backups_menu ;;        7) do_restore ;;
            8) do_settings ;;            9) do_parts ;;
            u|U) do_users ;;
            k|K) do_keys ;;
            a|A) do_addresses ;;
            l|L) choose_language; continue ;;
            h|H) do_helpme ;;
            m|M) menu_more; continue ;;
            0|q|Q) exit 0 ;;
            *) note "Please type one of the numbers or letters shown." ;;
        esac
        pause
    done
}

# The 'More' menu with the advanced jobs.
# ---------- change the web name in the files (menu M, 16) ------------------------------
# The files came with junsgarden.zapto.org written in a few places: the share
# buttons, the privacy page, the phone app's update check and the Android
# app's source. This swaps that name for the one you choose everywhere it
# matters, and keeps a copy of every file first. History notes (docs/,
# CHANGES.md) and the guides' examples stay as they were.
WEBNAME_FILES=(
    site/privacy.html
    site/apps/version.json
    rust/junsgarden/templates/index.html
    rust/junsgarden/src/main.rs
    rust/junsgarden/src/php.rs
    rust/junsgarden/src/pages/index.rs
    rust/junsgarden/src/pages/blog.rs
    rust/junsgarden/src/orchestrator/ai.rs
    beds/freeai.py
    docker-compose.yml
    android-app/app/build.gradle.kts
    android-app/app/src/main/kotlin/org/jasonagibson/junsgarden/app/Updater.kt
)

# Tidy what was typed: lowercase; no http://, https://, www., port or path.
clean_webname() {
    local n
    n=$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')
    n=${n#http://}; n=${n#https://}; n=${n#www.}; n=${n%%/*}; n=${n%%:*}; n=${n%.}
    printf '%s' "$n"
}
# A web name: parts of letters, numbers and - joined by dots, ending in letters.
valid_webname() { [[ "$1" =~ ^([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$ ]]; }
# The name the files use now, read from the phone app's update file.
current_webname() { grep -o 'https://[^/"]*' site/apps/version.json 2>/dev/null | head -1 | sed 's#https://##'; }

# Before a fresh install: keep junsgarden.zapto.org, or use a new name now?
# (Windows asks this itself and passes the answer as GARDEN_WEBNAME.)
ask_webname_first() {
    if [ -n "${GARDEN_WEBNAME:-}" ]; then do_webname "$GARDEN_WEBNAME"; return 0; fi
    [ "${ASSUME_YES:-0}" = 1 ] || [ -n "${GARDEN_REMOTE_INSTALL:-}" ] && return 0
    say "Your web name"
    Lblock <<'EOF'
   Your garden's address on the internet is a web name. It came as
   junsgarden.zapto.org.
     1) Keep the name it has now (recommended). Before going online, add it
        to your own free No-IP account (guide 04, Part A).
     2) Choose a new name now (Website name swap).
   You can change it later too: menu M, then 16.
EOF
    [ "$(ask "Type 1 or 2" 1)" = 2 ] && do_webname
    return 0
}

# How to get a web name (shown by the Website name swap).
webname_how_to() {
    Lblock <<'EOF'
   Getting a web name (free, about 5 minutes):
   1. Make a free account at https://www.noip.com (your own email).
   2. Dynamic DNS, No-IP Hostnames, Create Hostname: type a name and pick one of
      the free endings (for example zapto.org). If it is taken, try another.
   3. A free name must be confirmed every 30 days: No-IP emails you a link.
      (One free name per account; their paid plan has 3.)
   You can type the name here first and add it at No-IP before going online.
   More names for other things (Home Assistant, automations): free DuckDNS
   names, menu M, then 13 (guide 04, Advanced).
EOF
}

do_webname() {   # do_webname [NEW-NAME]   (with a name: no questions)
    local old new raw f keep changed=() given=${1:-}
    old=$(current_webname)
    [ -n "$old" ] || { fail "could not find the current web name in site/apps/version.json"; return 1; }
    explain "Website name swap" \
        "The files use $old in a few places: the website's share buttons, the privacy page, the phone app's update check (site/apps/version.json) and the Android app's source. This changes them to the name you choose. A copy of every file is kept first." \
        "To run the garden under another name (for example one for your mom). The new name must be in YOUR No-IP account too (guide 04, Part A); the No-IP updater (DUC) updates every name on your DDNS key." \
        "START-HERE/04-go-public-no-ip-caddy.md"
    [ -n "$given" ] || webname_how_to
    raw=${given:-$(ask "New web name (for example momsgarden.zapto.org; empty = keep $old)" "")}
    new=$(clean_webname "$raw")
    [ -n "$new" ] || { note "Nothing changed."; return 0; }
    [ "$new" = "$raw" ] || note "Using: $new   (http://, https://, www. and anything after / are left out)"
    valid_webname "$new" || { fail "\"$new\" does not look like a web name (like name.zapto.org)"; return 1; }
    [ "$new" != "$old" ] || { note "That is already the name. Nothing changed."; return 0; }
    for f in "${WEBNAME_FILES[@]}"; do [ -f "$f" ] && grep -qF "$old" "$f" && changed+=("$f"); done
    note "These files say $old and will say $new:"
    printf '      %s\n' "${changed[@]}"
    [ -n "$given" ] || confirm "Change them now?" y || { note "Nothing changed."; return 0; }

    keep="logs/web-name-change_$(date +%Y-%m-%d_%H%M)"
    for f in "${changed[@]}"; do
        if ! { mkdir -p "$keep/$(dirname "$f")" && cp -p "$f" "$keep/$f"; }; then
            fail "could not keep a copy of $f, so nothing more was changed"; return 1
        fi
        # A plain text swap: the dots in the name are matched as dots, not as patterns.
        OLD="$old" NEW="$new" perl -pi -e 's/\Q$ENV{OLD}\E/$ENV{NEW}/g' "$f"
    done
    note "Files changed. The old copies are in $keep (copy them back to undo)."

    # Already installed? Then the settings and the running programs follow.
    if [ -f .env ]; then
        if [ "$(env_get PUBLIC_NAME)" = "$old" ]; then
            env_set PUBLIC_NAME "$new"; env_set PUBLIC_URL "https://$new"
            note "Settings (.env): the web name is now $new."
            if systemctl is-active --quiet caddy 2>/dev/null; then apply_subdomains; fi
        fi
        if confirm "Rebuild the website now so it uses $new (a few minutes)?" y; then
            # shellcheck disable=SC2046  # the profile list is meant to split into words
            run dc $(profile_args) build && run dc up -d && wait_site
        else
            note "Later: menu M, then 7 (rebuild)."
        fi
    fi
    if [ -d .git ]; then
        git add -- "${changed[@]}" && git commit -q -m "Changed the web name from $old to $new (menu M, 16)" -- "${changed[@]}" \
            && note "Saved in your git history (git log shows it; git show shows every changed line)."
    fi
    note "Phones that already have the app still look at $old for updates. Build the app"
    note "again (android-app/README.md) and install it once on each phone; after that its"
    note "updates come from $new."
    note "The file check (menu M, 11) will list these files as changed: that is expected."
}

# More 17: reach the garden computer from anywhere, privately, with Tailscale.
# Tailscale links only YOUR devices into a small private network: no port is
# opened in the router, and nobody else can even see the computer. On: install
# it (Tailscale's own installer), sign in with the link it prints, and let SSH
# and Remote Desktop in through it (one firewall line). Off: the reverse.
do_tailscale() {
    explain "Reach the garden from anywhere, privately (Tailscale)" \
        "Tailscale links your own devices (this computer, your PC, your phone) into a small private network that only you can join. Each device gets a fixed name, such as $(hostname), that works at home and away. Free for personal use (up to 6 users, any number of devices)." \
        "So you can use SSH, WinSCP, Termius or Remote Desktop from anywhere, while ports 22 and 3389 stay closed in the router: only devices signed in to YOUR Tailscale account can connect." \
        "https://tailscale.com/kb/1017/install  and  https://tailscale.com/kb/1077/secure-server-ubuntu"
    if command -v tailscale >/dev/null 2>&1 && tailscale status >/dev/null 2>&1; then
        note "Tailscale is ON. This computer's name: $(hostname)   its private address: $(tailscale ip -4 2>/dev/null | head -1)"
        confirm "Switch Tailscale off (and close its firewall line)?" n || return 0
        run sudo tailscale down
        run sudo ufw delete allow in on tailscale0 2>/dev/null || true
        note "Off. To remove the program too:  sudo apt remove tailscale"
        return 0
    fi
    Lblock <<'EOF'
   Before you start (5 minutes):
   1. Make your free Tailscale account: https://tailscale.com/download
      (sign in with an account you already have, for example Google).
   2. Install Tailscale on your phone and your PC from the same page, and sign
      in there with the SAME account.
   Then come back here: this computer joins your private network.
EOF
    confirm "Install Tailscale on this computer now (Tailscale's own installer)?" y || return 0
    if ! command -v tailscale >/dev/null 2>&1; then
        note "Tailscale's official install script (it adds Tailscale's own Ubuntu package source):"
        run bash -c "curl -fsSL https://tailscale.com/install.sh | sh" || { fail "Tailscale did not install. Is the internet on?"; return 1; }
    fi
    note "Now sign in: a link appears below. Open it in a browser (on any device), sign in"
    note "with your Tailscale account, and this computer joins. This waits until you have."
    run sudo tailscale up || { fail "not signed in. Choose this again to retry."; return 1; }
    note "Letting SSH and Remote Desktop in through Tailscale only (one firewall line):"
    run sudo ufw allow in on tailscale0 comment 'Tailscale: only your own devices'
    note "Done. In Termius, WinSCP or Remote Desktop, use this as the Host name, at home and away:"
    note "   $(hostname)    (or its private address: $(tailscale ip -4 2>/dev/null | head -1))"
    note "Your devices: https://login.tailscale.com/admin/machines . Off again: this same menu item."
}

# ---------- the on / off switch and Docker's check-up (More 14; garden.sh off | on | checkup | cleanup) ----
# OFF stops every garden program and the front door. It is "docker compose down"
# WITHOUT -v: the containers are removed, but every file stays. Your plants (the
# database folder), photos, data, cache, backups and settings live in folders on
# this computer, not inside Docker. ON starts it all again, even months later,
# and checks it. The check-up starts a stuck program again. The clean-up removes
# only Docker's leftovers (old image versions, images nothing uses, old build
# notes): never an image the garden uses, never a volume, never one of your folders.
OFF_MARK=.garden-off
garden_is_off() { [ -f "$OFF_MARK" ]; }

do_power() {
    [ -f .env ] || { fail "install first (menu 1)"; return 1; }
    say "On / off switch and Docker check-up"
    if garden_is_off; then note "Now: OFF ($(head -1 "$OFF_MARK"))."
    else note "Now: ON ($(dc ps -q 2>/dev/null | wc -l) programs running)."; fi
    Lblock <<'EOF'
   Nothing here deletes your plants, photos, data, cache, backups or settings.

   1) Everything OFF: stops all the garden's programs and the front door.
      For when you do not want to host the site for a while, or the internet
      traffic is too much. Turn it on again any time, even months later.
   2) Everything ON again: starts it all and checks the website
   3) Off the internet only: the garden keeps running at home (More 1 puts it back)
   4) Check-up: start again any program that is stuck
   5) Clean-up: remove Docker's leftovers to free disk space (safe)
   6) Automatic check-up every 6 hours (on / off)
   0) Back
EOF
    case "$(ask "Number" 0)" in
        1) power_off ;;        2) power_on ;;
        3) do_public_off ;;    4) docker_checkup ;;
        5) docker_cleanup ;;   6) checkup_schedule ;;
        *) return 0 ;;
    esac
}

# Switch 1 (garden.sh off): everything off; every file stays.
power_off() {
    garden_is_off && { note "It is already off. Choice 2 turns it on."; return 0; }
    explain "Everything off" \
        "Stops and removes the garden's programs (docker compose down, WITHOUT -v) and stops the front door (Caddy). Your plants, photos, data, cache, backups and settings stay in their folders, untouched. The nightly backup pauses: nothing changes while it is off." \
        "When you do not want to host the site for a while, or the traffic is too much. Visitors then get no answer. Turning it on again brings back exactly what was there." \
        "https://docs.docker.com/reference/cli/docker/compose/down/"
    confirm "Turn the garden off now?" n || return 0
    local caddy=0
    if systemctl is-enabled --quiet caddy 2>/dev/null; then caddy=1; run sudo systemctl disable --now caddy; fi
    # shellcheck disable=SC2046  # the profile list is meant to split into words
    run dc $(profile_args) --profile dbweb down || { fail "Docker did not stop everything; look at: docker ps"; return 1; }
    printf 'since %s\ncaddy=%s\n' "$(date '+%Y-%m-%d %H:%M')" "$caddy" > "$OFF_MARK"
    note "Off. Nothing is running, and nothing was deleted."
    Lblock <<'EOF'
   While it is off:
   - A free No-IP web name must still be confirmed every 30 days (No-IP emails
     you a link). Keep clicking it, or the name is lost.
   - The padlock (HTTPS) certificate renews by itself when the garden is on again.
   - On again: menu M, 14, then 2 (or menu 2, or: bash garden.sh on).
EOF
}

# Switch 2 (garden.sh on; menu 2 too when it was switched off): start it all and check.
power_on() {
    local since='' caddy=0 days plants
    if garden_is_off; then
        since=$(sed -n 's/^since //p' "$OFF_MARK"); caddy=$(sed -n 's/^caddy=//p' "$OFF_MARK")
    fi
    say "Turning the garden on"
    # If Docker lost a program's image meanwhile, "up" builds it again (that takes longer).
    run dc up -d || { fail "the programs did not start. Look at:  bash garden.sh logs"; return 1; }
    rm -f "$OFF_MARK"
    [ "$caddy" = 1 ] && run sudo systemctl enable --now caddy
    wait_site || return 1
    wait_db; plants=$(echo "SELECT COUNT(*) FROM juns_garden_db.data_source;" | db_root 2>/dev/null)
    note "The website answers at $(site_url). Plants in the database: ${plants:-?}"
    if [ -n "$since" ]; then
        days=$(( ($(date +%s) - $(date -d "$since" +%s 2>/dev/null || date +%s)) / 86400 ))
        note "It was off for $days day(s), since $since."
        [ "$days" -ge 1 ] && confirm "Make a backup now (the nightly one was paused)?" y && do_backup
    fi
    [ "$caddy" = 1 ] && note "The front door is on again. An old padlock certificate renews by itself within minutes."
    return 0
}

# Switch 4 (garden.sh checkup; also every 6 hours if switched on): every garden
# program that should run is running and healthy. A stuck one (stopped with an
# error, "unhealthy", or restarting over and over) is made again and started.
# It never deletes a file. Quiet (from the schedule): prints only what it fixed.
docker_checkup() {   # docker_checkup [quiet]
    local quiet=${1:-} svc state health code fixed=0 n=0
    if garden_is_off; then
        [ -n "$quiet" ] || note "The garden is switched off, so there is nothing to check. Choice 2 turns it on."
        return 0
    fi
    # "|" between the fields, not a tab: bash's read joins tabs, so an empty
    # health field would shift the exit code into its place.
    while IFS='|' read -r svc state health code; do
        n=$((n + 1))
        case "$state" in
            running) [ "$health" = unhealthy ] || continue ;;
            exited)  case "$svc" in *-init|*-setup) [ "$code" = 0 ] && continue ;; esac ;;
        esac
        note "$(date '+%Y-%m-%d %H:%M') $svc was $state${health:+ ($health)}: starting it again."
        dc up -d --force-recreate "$svc" >/dev/null 2>&1 && fixed=$((fixed + 1))
    done < <(dc ps -a --format '{{.Service}}|{{.State}}|{{.Health}}|{{.ExitCode}}' 2>/dev/null)
    # Anything missing altogether is made again too.
    dc up -d >/dev/null 2>&1
    if ! curl -fsS -o /dev/null --max-time 10 "$(site_url)/healthz"; then
        note "$(date '+%Y-%m-%d %H:%M') the website did not answer; restarting it."
        dc restart site >/dev/null 2>&1 && fixed=$((fixed + 1))
    fi
    if [ -z "$quiet" ] && systemctl is-enabled --quiet caddy 2>/dev/null && ! systemctl is-active --quiet caddy; then
        note "The front door (Caddy) had stopped: starting it again."; run sudo systemctl restart caddy
    fi
    [ -n "$quiet" ] && return 0
    if [ "$fixed" -eq 0 ]; then note "All $n programs are fine. Nothing needed doing."
    else note "Started again: $fixed. If one keeps getting stuck, its messages say why: bash garden.sh logs"; fi
}

# Switch 5 (garden.sh cleanup): remove Docker's leftovers, safely.
docker_cleanup() {
    explain "Clean-up: Docker's leftovers" \
        "Removes old versions of the garden's programs (left behind each time one is rebuilt or updated) that are older than 7 days, and Docker's build notes older than 30 days. It keeps every image the garden uses, also those of parts that are off right now, and anything newer than 7 days (after you edited files and rebuilt, the old version stays in case you need to go back). Images of your other Docker projects are left alone. Use it with care: it shows the list and asks first, and Enter means no." \
        "Docker keeps old copies forever; after some updates they can fill gigabytes. Your plants, photos, data, cache, backups and settings are not Docker's: they are never touched. Docker volumes are never touched either." \
        "https://docs.docker.com/engine/manage-resources/pruning/"
    garden_is_off && { fail "the garden is switched off. Turn it on first (choice 2): the clean-up runs only while the garden runs."; return 1; }
    dc ps --status running -q 2>/dev/null | has . || { fail "the garden's programs are not running. Start them first (menu 2)."; return 1; }
    local keep keep_names id repo p made young=0
    local -a all_profiles=() remove=()
    # Every image the garden uses, with every part switched on (so nothing of a part that is off goes).
    for p in $(grep -o 'profiles: \["[a-z-]*"\]' docker-compose.yml | grep -o '"[a-z-]*"' | tr -d '"' | sort -u); do all_profiles+=(--profile "$p"); done
    keep=$(dc "${all_profiles[@]}" config --images 2>/dev/null | sed -E '/:[^/]*$/!s/$/:latest/')
    [ -n "$keep" ] || { fail "could not read the garden's list of images, so nothing was removed"; return 1; }
    keep_names=$(printf '%s\n' "$keep" | sed -E 's/:[^/:]*$//' | sort -u)
    run dk system df
    # A leftover: an image that no container uses (running or stopped), and that is
    # either nameless ("<none>": an old version a rebuild left behind) or an older
    # version of one of the garden's own images (same name, another version).
    # Images of other Docker projects (Docker Desktop may run many) never qualify.
    while IFS=$'\t' read -r id repo; do
        printf '%s\n' "$keep" | has -Fx "$repo" && continue
        if [ "$repo" != "<none>:<none>" ]; then printf '%s\n' "$keep_names" | has -Fx "${repo%:*}" || continue; fi
        [ -n "$(dk ps -a -q --filter ancestor="$id")" ] && continue
        # Newer than 7 days stays: a recent rebuild's old version may still be wanted.
        made=$(date -d "$(dk image inspect -f '{{.Created}}' "$id" 2>/dev/null)" +%s 2>/dev/null || date +%s)
        if [ $(( $(date +%s) - made )) -lt $((7 * 86400)) ]; then young=$((young + 1)); continue; fi
        remove+=("$( [ "$repo" = "<none>:<none>" ] && echo "$id" || echo "$repo" )")
    done < <(dk images --format '{{.ID}}\t{{.Repository}}:{{.Tag}}')
    if [ "${#remove[@]}" -gt 0 ]; then
        note "Leftovers that nothing uses:"
        printf '      %s\n' "${remove[@]}"
        if confirm "Remove these?" n; then
            # No -f: Docker itself refuses to remove an image that something still uses.
            for id in "${remove[@]}"; do dk rmi "$id" >/dev/null 2>&1 && note "removed $id"; done
        fi
    else
        note "No unused images older than 7 days."
    fi
    [ "$young" -gt 0 ] && note "Kept $young newer than 7 days (they go in a later clean-up)."
    note "Old build notes (older than 30 days):"
    run dk builder prune -f --filter until=720h
    run dk system df
    note "Done. Never used here: docker volume prune, docker system prune --volumes, or anything with -v."
}

# Switch 6: the automatic check-up every 6 hours (a cron line), on or off.
checkup_schedule() {
    if crontab -l 2>/dev/null | has -F 'garden.sh checkup'; then
        confirm "The automatic check-up is on. Switch it off?" n || return 0
        ( crontab -l 2>/dev/null | grep -v -F 'garden.sh checkup' ) | crontab -
        note "Off."
        return 0
    fi
    explain "Automatic check-up" \
        "Every 6 hours, cron runs 'garden.sh checkup': a program that is stuck is started again, and a line goes into logs/checkup.log. It never deletes anything, and does nothing while the garden is switched off." \
        "So the garden mends small hiccups by itself, also while you are away. See the line: crontab -l" ""
    confirm "Switch it on?" y || return 0
    local line="15 */6 * * * cd \"$HERE\" && bash garden.sh checkup >> logs/checkup.log 2>&1"
    mkdir -p logs
    ( crontab -l 2>/dev/null | grep -v -F 'garden.sh checkup'; echo "$line" ) | crontab -
    note "On: $line"
}

menu_more() {
    while true; do
        banner "More"
        Lblock <<'EOF'

   1) The front door: HTTPS, your sign-in page, your web name
   2) Network: my IP address and reserving it in the router
   3) Email (Gmail) for password links, with a test email
   4) Set my password (website + sign-in page)
   5) Install on ANOTHER computer (over SSH)
   6) Show the programs' recent messages (logs)
   7) Rebuild after editing Rust files
   8) Look inside the database (Adminer web page)
   9) Connect a local AI computer
  10) Desktop icon
  11) Look at the files that came from the author (and check them)
  12) Uninstall (programs only, or everything)
  13) Subdomains with DuckDNS (wiki., n8n., ...): add, remove
  14) On / off: the whole garden, or only off the internet; check-up, clean-up
  15) AI cost control: LiteLLM gateway with a monthly limit (on / off)
  16) Website name swap: use another web name than junsgarden.zapto.org
  17) Reach the garden from anywhere, privately (Tailscale): on / off
  18) Rename or move a file or folder (every place that uses it follows)

   0) Back
EOF
        local c; echo; read -r -p "  $(Lt "Type a number, then press Enter"): " c || { echo; exit 0; }   # no more input: stop
        case "$c" in
            1) do_public ;;              2) do_network ;;
            3) do_email ;;               4) do_password ;;
            5) do_remote ;;              6) dc logs --tail 50 ;;
            7) do_rebuild ;;             8) do_dbweb ;;
            9) do_ai ;;                  10) make_icon ;;
            11) do_whats_from_author ;;  12) do_uninstall ;;
            13) do_subdomains ;;        14) do_power ;;
            15) do_llm ;;               16) do_webname ;;
            17) do_tailscale ;;         18) do_rename ;;
            0|b|B) return 0 ;;
            *) note "Please type one of the numbers shown." ;;
        esac
        pause
    done
}

case "${1:-menu}" in
    menu)       menu ;;
    install)    do_install ;;
    public)     do_public ;;
    remote)     do_remote ;;
    start)      if garden_is_off; then power_on; else dc up -d && wait_site; fi ;;
    stop)       dc stop ;;
    open)       xdg-open "$(site_url)" >/dev/null 2>&1 & ;;
    logs)       dc logs --tail 50 ;;
    backup)     do_backup "${2:-}" ;;
    backups)    do_backups_menu ;;
    nas)        do_nas ;;
    cloud)      do_cloud ;;
    restore)    do_restore "${2:-}" ;;
    verify)     tools tools verify "${2:-latest}" ;;
    check)      do_check ;;
    password)   do_password "${2:-}" ;;
    email)      do_email ;;
    network)    do_network ;;
    parts)      do_parts ;;
    ai)         do_ai ;;
    wiki)       wiki_setup ;;
    rebuild)    do_rebuild ;;
    dbweb)      do_dbweb ;;
    dbweb-stop) dc --profile dbweb stop adminer && dc --profile dbweb rm -f adminer ;;
    db)         do_db ;;
    icon)       make_icon ;;
    uninstall)  do_uninstall ;;
    helpme)     do_helpme ;;
    files)      do_whats_from_author ;;
    settings)   do_settings ;;
    keys)       do_keys ;;
    addresses)  do_addresses ;;
    subdomains) do_subdomains ;;
    offline)    do_public_off ;;
    llm)        do_llm ;;
    language)   choose_language ;;
    users)      do_users ;;
    lockdown)   do_lockdown ;;
    webname)    do_webname ;;
    tailscale)  do_tailscale ;;
    rename)     do_rename ;;
    renames)    show_renames ;;
    redo-renames) redo_renames ;;
    power)      do_power ;;
    off)        power_off ;;
    on)         power_on ;;
    checkup)    docker_checkup "${2:-}" ;;
    cleanup)    docker_cleanup ;;
    photos)     do_photos ;;
    test)       dk run --rm -v "$HERE/rust":/build -w /build rust:1.95-bookworm cargo test --locked -p junsgarden ;;
    *)          sed -n '2,34p' "$0" ;;
esac
