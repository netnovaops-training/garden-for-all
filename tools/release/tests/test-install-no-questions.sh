#!/bin/bash
# shellcheck disable=SC2016,SC2034  # the checks are strings, read by eval
# =============================================================================
#  Test-only: two install steps that must finish without
#  anybody at the keyboard. Found in a real install through the setup wizard
#  (2026-10-14), where the black window then sat waiting or stopped:
#   1. Step 8 (settings file): with the wizard, the database password is made
#      for you (the wizard's summary says so); it must not be asked.
#   2. Step 4 (SSH server): when port 22 is already taken (on a Windows PC often
#      another Ubuntu in WSL, which shares the network), the SSH server is
#      installed WITHOUT starting it, instead of stopping the whole install.
#  The two functions are taken from garden.sh as they are; everything around
#  them (apt, sudo, systemctl, the questions) is a stand-in that only records.
#     bash tools/release/tests/test-install-no-questions.sh
# =============================================================================
HERE=$(cd "$(dirname "$0")/../../.." && pwd)
pass=0; failn=0
chk() { if eval "$2"; then echo "PASS  $1"; pass=$((pass + 1)); else echo "FAIL  $1"; failn=$((failn + 1)); fi; }
fn() { awk -v n="$1()" 'index($0, n " {") == 1 {p = 1} p {print} p && /^}/ {exit}' "$HERE/garden.sh"; }
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
cd "$T" || exit 1

# ---- stand-ins -----------------------------------------------------------------
note() { printf 'NOTE %s\n' "$*" >> calls.log; }
fail() { printf 'FAIL %s\n' "$*" >> calls.log; }
explain() { :; }
ask() { printf 'ASK %s\n' "$1" >> calls.log; echo ""; }
ask_secret() { printf 'ASKSECRET %s\n' "$1" >> calls.log; echo ""; }
owner_get() { echo ""; }
owner_env() { :; }
wizard_secret() { echo ""; }
run() { printf 'RUN %s\n' "$*" >> calls.log; "$@"; }
sudo() { printf 'SUDO %s\n' "$*" >> calls.log
         case "$1" in tee) cat > "$T/policy-rc.d" ;; rm) rm -f "$T/policy-rc.d" ;; esac; return 0; }
systemctl() { printf 'SYSTEMCTL %s\n' "$*" >> calls.log; [ "$1" = is-active ] && echo inactive; return 0; }

# ---- 1. the settings file, from the wizard, nobody at the keyboard -----------
eval "$(fn make_env | sed 's#/usr/sbin/policy-rc.d#'"$T"'/policy-rc.d#g')"
: > calls.log
GARDEN_WIZARD=1 GARDEN_NAME=Jun GARDEN_EMAIL=jun@example.com make_env </dev/null >/dev/null 2>&1
chk "wizard: the database password is not asked" '! grep -q "^ASKSECRET" calls.log'
chk "wizard: it says one is made for you" 'grep -q "^NOTE The database password: a strong one is made for you" calls.log'
chk "wizard: .env has a long random database password" 'grep -Eq "^DB_PASSWORD=[0-9a-f]{48}$" .env'
chk "wizard: .env has Jun as the login" 'grep -qx "GARDEN_USERS=Jun=jun@example.com" .env'
rm -f .env; : > calls.log
# Without the wizard and with a keyboard (a terminal), it is still asked: run under `script`.
if command -v script >/dev/null; then
    declare -f note fail explain ask ask_secret owner_get owner_env wizard_secret make_env > funcs.sh
    script -qc "bash -c 'source ./funcs.sh; GARDEN_NAME=Jun GARDEN_EMAIL=jun@example.com make_env'" /dev/null </dev/null >/dev/null 2>&1
    chk "no wizard, a person at the keyboard: the question is still asked" 'grep -q "^ASKSECRET Database password" calls.log'
    rm -f .env; : > calls.log
    script -qc "bash -c 'source ./funcs.sh; GARDEN_WIZARD=1 GARDEN_NAME=Jun GARDEN_EMAIL=jun@example.com make_env'" /dev/null </dev/null >/dev/null 2>&1
    chk "wizard, in the black window (a terminal): not asked either" '! grep -q "^ASKSECRET" calls.log && grep -Eq "^DB_PASSWORD=[0-9a-f]{48}$" .env'
fi

# ---- 2. the SSH server when port 22 is taken ---------------------------------------
eval "$(fn install_ssh | sed 's#/usr/sbin/policy-rc.d#'"$T"'/policy-rc.d#g')"
ssh_port_busy() { return 0; }
: > calls.log
install_ssh >/dev/null 2>&1; rc=$?
chk "port 22 taken: the step finishes" '[ $rc = 0 ]'
chk "port 22 taken: it says why the server stays off" 'grep -q "^NOTE Port 22 (SSH) is already in use" calls.log'
chk "port 22 taken: services are held while it installs (policy-rc.d, exit 101)" 'grep -q "^SUDO tee .*policy-rc.d" calls.log'
chk "port 22 taken: the hold is removed afterwards" '[ ! -e "$T/policy-rc.d" ] && grep -q "^SUDO rm -f .*policy-rc.d" calls.log'
chk "port 22 taken: the server is not started" '! grep -q "enable --now ssh" calls.log'
ssh_port_busy() { return 1; }
: > calls.log
install_ssh >/dev/null 2>&1; rc=$?
chk "port 22 free: installed and started as before" '[ $rc = 0 ] && grep -q "enable --now ssh" calls.log && ! grep -q "policy-rc.d" calls.log'
echo "RESULT: $pass passed, $failn failed"
[ "$failn" = 0 ]
