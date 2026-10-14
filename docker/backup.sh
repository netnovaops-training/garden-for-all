#!/bin/sh
# =============================================================================
#  backup.sh: kept so older guides still work. The backup itself now lives in
#  garden.sh (and the toolbox container), so Ubuntu and Windows make the very
#  same kind of backup.
#
#  Run:   sh docker/backup.sh          (the same as:  bash garden.sh backup)
#
#  What you get: ONE file in the backups folder, named by date and time:
#      backups/junsgarden-backup_2026-10-13_0230.tar
#  It holds the database, every photo, the data files and the AI cache, plus
#  a MANIFEST.txt (what is inside) and SHA256SUMS (fingerprints that prove
#  nothing is missing or damaged). START-HERE/13-backups-and-folders.md
#  explains the names and how to restore.
# =============================================================================
cd "$(dirname "$0")/.." || exit 1
exec bash garden.sh backup "$@"
