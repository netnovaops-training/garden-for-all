#!/bin/bash
# =============================================================================
#  03-start-garden.sh: start your garden website (a friendly wrapper).
#
#  Run:   bash lab/03-start-garden.sh
#
#  It runs  bash garden.sh install , which:
#    1. checks Docker (installs it if missing)
#    2. asks your name and email and writes the private .env file
#    3. builds the site, loads your plants and photos from the backup that
#       came in the zip, and starts it
#    4. adds the nightly backup and a desktop icon
#    5. writes the checklist (CHECKLIST.txt)
#
#  The first build takes 5 to 15 minutes (30 to 60 on an old laptop).
# =============================================================================
set -eu
cd "$(dirname "$0")/.."
bash garden.sh install
echo
echo "Next: set your password:  bash garden.sh password"
