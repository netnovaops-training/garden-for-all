#!/bin/sh
# Copy the working files (committed or not) of this checkout to the lab machine's
# ~/garden-rust-pages/src-cand, plus the lab scripts. The lab machine is any Linux box with
# Docker, reached over SSH:   GARDEN_LAB=user@host sh docs/rust-pages/sync.sh
set -eu
: "${GARDEN_LAB:?set GARDEN_LAB=user@host (the lab machine)}"
cd "$(dirname "$0")/../.."
git ls-files -co --exclude-standard -z | tar --null -T - -cf - |
    ssh -o BatchMode=yes "$GARDEN_LAB" 'd=~/garden-rust-pages/src-cand; mkdir -p $d && find $d -mindepth 1 -delete && tar -xf - -C $d'
# (The folder is emptied, not replaced: a running gp-cand has src-cand/START-HERE mounted as
# /guides, and a replaced folder would leave it looking at the deleted one.)
scp -q docs/rust-pages/lab.sh docs/rust-pages/sbs.py docs/rust-pages/shots.py docs/rust-pages/urls-*.txt \
    docs/rust-pages/pages-visual.txt "$GARDEN_LAB:garden-rust-pages/"
echo "synced"
