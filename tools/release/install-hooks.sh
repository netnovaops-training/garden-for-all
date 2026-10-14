#!/bin/sh
# Installs the repository's git hooks (tools/release/hooks/*) into this clone:
#     sh tools/release/install-hooks.sh
# Right now there is one: pre-commit, which keeps CODE-FINGERPRINTS.txt up to date.
# An existing hook of the same name that is different is not overwritten.
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
hooks=$(cd "$root" && git rev-parse --git-path hooks) || exit 1
case "$hooks" in /*) ;; *) hooks="$root/$hooks" ;; esac
mkdir -p "$hooks"
for f in "$here"/hooks/*; do
    name=$(basename "$f")
    if [ -e "$hooks/$name" ] && ! cmp -s "$f" "$hooks/$name"; then
        echo "A different $name hook is already in $hooks. Not replaced; look at it, then copy $f over it yourself." >&2
        exit 1
    fi
    cp "$f" "$hooks/$name" && chmod +x "$hooks/$name" && echo "installed: $hooks/$name"
done
