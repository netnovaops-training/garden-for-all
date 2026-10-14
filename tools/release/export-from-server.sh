#!/bin/bash
# =============================================================================
#  export-from-server.sh: how the garden's data was taken from the old live
#  server for its new owner. Kept here so anyone can see exactly what went into
#  the data download, and what was left out on purpose.
#
#  It READS the live garden (nothing there is changed) and writes one backup
#  file in the garden's own backup format (the same layout docker/tools/
#  garden-tools.sh makes), so `bash garden.sh restore` puts it back:
#     database.sql.gz  the plants database
#     images.tar       the photos
#     data.tar         comments, guestbook, lists
#     cashe.tar        the AI cache
#     MANIFEST.txt     what is inside, and what was left out
#     SHA256SUMS       a fingerprint of each piece, checked on restore
#
#  LEFT OUT ON PURPOSE:
#     - the logins: garden_users (password hashes), garden_sessions and
#       login_attempts are exported EMPTY. The new owner sets his own password.
#     - four old safety-copy tables nobody needs.
#  Nothing here prints a password: the database password is read inside the
#  database container (MYSQL_PWD), never on this command line.
#
#  Run on the old server, with its own folder and container names:
#     GARDEN_DATA=/path/to/garden-data GARDEN_DB=the-db-container bash export-from-server.sh
#  GARDEN_DATA holds images/, www/data and www/cashe (the live site's layout).
# =============================================================================
set -euo pipefail
SRC=${GARDEN_DATA:?set GARDEN_DATA to the live garden data folder}
DB=${GARDEN_DB:?set GARDEN_DB to the database container name}
stamp=$(date +%Y-%m-%d_%H%M)
name="junsgarden-backup_${stamp}_from-the-author"
out=$HOME/jun-export
work=$out/$name
rm -rf "$out"; mkdir -p "$work"

# The database, without the logins (their tables are kept, empty) and the old safety copies.
# shellcheck disable=SC2016
docker exec "$DB" sh -c 'MYSQL_PWD="${MARIADB_ROOT_PASSWORD:-$MYSQL_ROOT_PASSWORD}" exec mariadb-dump -uroot \
    --single-transaction --routines --triggers --add-drop-table \
    --ignore-table-data=juns_garden_db.garden_users \
    --ignore-table-data=juns_garden_db.garden_sessions \
    --ignore-table-data=juns_garden_db.login_attempts \
    --ignore-table=juns_garden_db.data_source_backup_20260929_063426 \
    --ignore-table=juns_garden_db.data_source_srcbak_20260930_030827 \
    --ignore-table=juns_garden_db.future_plants_srcbak_20260930_030826 \
    --ignore-table=juns_garden_db.soil_type_backup_20260928_043124 \
    juns_garden_db' | gzip > "$work/database.sql.gz"

# shellcheck disable=SC2016
plants=$(docker exec "$DB" sh -c 'MYSQL_PWD="${MARIADB_ROOT_PASSWORD:-$MYSQL_ROOT_PASSWORD}" mariadb -uroot -N -e "SELECT COUNT(*) FROM juns_garden_db.data_source"')

{
    echo "Jun's Garden backup"
    echo "Name:        $name"
    echo "Made:        $(date '+%Y-%m-%d %H:%M %Z')"
    echo "From:        The author's server (the live junsgarden.zapto.org), for Jun's own copy"
    echo "Plants:      $plants"
    echo "Left out on purpose: login passwords and sessions (set your own with"
    echo "garden.sh password) and four old safety-copy tables."
    echo
    echo "Pieces (each one is checked by SHA256SUMS when you restore):"
    echo "  database.sql.gz   the whole database (juns_garden_db)"
} > "$work/MANIFEST.txt"

# The folders, each as one piece.
for part in images data cashe; do
    case $part in images) dir=$SRC/images ;; data) dir=$SRC/www/data ;; cashe) dir=$SRC/www/cashe ;; esac
    tar -cf "$work/$part.tar" -C "$dir" .
    printf '  %-16s  %s files, %s\n' "$part.tar" "$(find "$dir" -type f | wc -l)" "$(du -sh "$dir" | cut -f1)" >> "$work/MANIFEST.txt"
done
{
    echo
    echo "Restore:  Ubuntu: bash garden.sh restore $name.tar"
    echo "          Windows: Garden-Menu, 'This PC', 'Restore a backup'"
} >> "$work/MANIFEST.txt"

( cd "$work" && sha256sum database.sql.gz ./*.tar MANIFEST.txt > SHA256SUMS )
tar -cf "$out/$name.tar" -C "$out" "$name"
rm -rf "$work"
ls -l "$out"
echo "plants=$plants"
