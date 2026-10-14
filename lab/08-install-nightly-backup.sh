#!/bin/bash
# =============================================================================
#  08-install-nightly-backup.sh: make the garden back itself up every night.
#
#  Run:   bash lab/08-install-nightly-backup.sh
#  Safe to run twice: it keeps exactly one backup line.
#
#  What it does: adds one line to your personal cron (the laptop's alarm clock)
#  so that  garden.sh backup nightly  runs at 2:30 every night. Each night
#  makes backups/junsgarden-backup_DATE_0230_nightly.tar. Nightly backups
#  older than 30 days are deleted; the ones you make by hand are kept.
#  The output is written to backups/backup.log.
#
#  If you MOVE or RENAME the garden folder, run this again: the line holds the
#  folder's full path.
#
#  Remember: a backup on the same laptop is only half a backup. Copy the
#  backups folder to a USB stick every week (Project 2).
#
#  Undo:  crontab -e    and delete the line that mentions garden.sh backup
# =============================================================================
set -eu
cd "$(dirname "$0")/.."
HERE=$(pwd)

mkdir -p backups
LINE="30 2 * * * cd \"$HERE\" && bash garden.sh backup nightly >> backups/backup.log 2>&1"
( crontab -l 2>/dev/null | grep -v -e 'garden.sh backup' -e 'docker/backup.sh'; echo "$LINE" ) | crontab -
echo "Scheduled:"
crontab -l | grep -F "garden.sh backup"
