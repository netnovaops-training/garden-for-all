#!/bin/sh
# =============================================================================
#  setup.sh: kept so older guides still work. It now hands over to garden.sh,
#  which does the whole setup (Docker, firewall, settings, build, your plants
#  and photos, nightly backup, desktop icon, checklist).
#
#  Run:   sh setup.sh        (the same as:  bash garden.sh install)
# =============================================================================
cd "$(dirname "$0")" || exit 1
exec bash garden.sh install "$@"
