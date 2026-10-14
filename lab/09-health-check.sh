#!/bin/bash
# =============================================================================
#  09-health-check.sh: "is my lab working?" in one command.
#
#  Run:   bash lab/09-health-check.sh
#
#  It runs the garden's checklist (the same as menu 5, or: bash garden.sh check).
#  It only LOOKS. It changes nothing. Each line says PASS, WARN, FAIL or INFO,
#  and every FAIL and WARN line says what to do. The last line is the score.
#  A copy is saved in CHECKLIST.txt, with the date and time it ran.
# =============================================================================
cd "$(dirname "$0")/.." || exit 1
exec bash garden.sh check
