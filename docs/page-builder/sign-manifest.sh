#!/bin/sh
# Sign the page builder's list of sections (rust/junsgarden/sections-manifest.json), so
# gardens can trust the "new sections" badge. For the author only. Needs OpenSSL 3.
#
#   sh docs/page-builder/sign-manifest.sh --new-key /outside/the/repo/sections-key.pem
#       once: makes the Ed25519 key and prints its public half (64 hex characters) to put in
#       AUTHOR_SECTIONS_KEY in rust/junsgarden/src/pagebuilder/updates.rs.
#   sh docs/page-builder/sign-manifest.sh /outside/the/repo/sections-key.pem
#       after every change to the list: writes sections-manifest.json.sig next to it.
#
# The signed text is the line "junsgarden-sections-v1" and then the list's exact bytes.
# The key must stay OUTSIDE the repository (it is refused inside it). Never commit it.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=$(git -C "$here" rev-parse --show-toplevel)

new=0
if [ "${1:-}" = "--new-key" ]; then new=1; shift; fi
key=${1:?"usage: sign-manifest.sh [--new-key] /path/outside/the/repo/key.pem"}
keydir=$(cd "$(dirname "$key")" && pwd)
case "$keydir/" in
    "$repo"/*) echo "The key must be outside the repository: $key"; exit 1 ;;
esac

if [ "$new" = 1 ]; then
    [ ! -e "$key" ] || { echo "$key already exists; not replaced."; exit 1; }
    (umask 077 && openssl genpkey -algorithm ed25519 -out "$key")
    printf 'Public key for AUTHOR_SECTIONS_KEY: '
    openssl pkey -in "$key" -pubout -outform DER | tail -c 32 | od -An -tx1 | tr -d ' \n'
    echo
    exit 0
fi

list="$repo/rust/junsgarden/sections-manifest.json"
msg=$(mktemp)
trap 'rm -f "$msg"' EXIT
{ printf 'junsgarden-sections-v1\n'; cat "$list"; } > "$msg"
openssl pkeyutl -sign -inkey "$key" -rawin -in "$msg" | od -An -tx1 | tr -d ' \n' > "$list.sig"
echo "Signed: $list.sig (commit both files together)"
