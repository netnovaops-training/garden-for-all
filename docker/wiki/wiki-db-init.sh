#!/bin/sh
# =============================================================================
#  wiki-db-init.sh: make sure the wiki has its own database and login.
#
#  You do not run this yourself. Docker runs it (service "wiki-db-init")
#  every time the garden starts, just before the wiki starts.
#
#  Why every time and not only once?
#    MariaDB runs its "first start" scripts only when its data folder is
#    EMPTY. Your garden database may already exist, so that would never
#    happen. This script is safe to run any number of times:
#      - "IF NOT EXISTS" means: make it only if it is not there yet.
#      - ALTER USER sets the wiki password again from your .env file, so the
#        database and the wiki always agree on it.
#    It never deletes anything and never touches the plant database.
#
#  What it makes:
#    database  junsgarden_wiki   (the wiki's pages, users and settings)
#    user      wikijs            (may only use junsgarden_wiki, nothing else)
#
#  Passwords come from .env through Docker (DB_ROOT_PASSWORD and
#  WIKI_DB_PASSWORD). They are never printed and never on a command line.
# =============================================================================
set -eu

DB_HOST=${DB_HOST:-db}
WIKI_DB_NAME=${WIKI_DB_NAME:-junsgarden_wiki}
WIKI_DB_USER=${WIKI_DB_USER:-wikijs}

[ -n "${DB_ROOT_PASSWORD:-}" ]  || { echo "ERROR: DB_ROOT_PASSWORD is not set (is the .env file there?)" >&2; exit 1; }
[ -n "${WIKI_DB_PASSWORD:-}" ]  || { echo "ERROR: WIKI_DB_PASSWORD is not set in .env" >&2; exit 1; }

# MYSQL_PWD is the variable the mariadb program reads the password from,
# so the password never shows up in a list of running programs.
export MYSQL_PWD="$DB_ROOT_PASSWORD"

# Wait until the database answers (2 minutes at most).
tries=0
until mariadb -h "$DB_HOST" -uroot -e 'SELECT 1' >/dev/null 2>&1; do
    tries=$((tries + 1))
    if [ "$tries" -ge 60 ]; then
        echo "ERROR: the database (garden-db) did not answer for 2 minutes." >&2
        exit 1
    fi
    sleep 2
done

# Inside SQL text a ' must be written as '' and a \ as \\.
# This makes any password safe to put between the quotes below.
pw=$(printf '%s' "$WIKI_DB_PASSWORD" | sed -e 's/\\/\\\\/g' -e "s/'/''/g")

# The SQL goes in through standard input (the "<<SQL" part), not the
# command line, so the password stays private.
mariadb -h "$DB_HOST" -uroot <<SQL
CREATE DATABASE IF NOT EXISTS \`$WIKI_DB_NAME\`
    CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER IF NOT EXISTS '$WIKI_DB_USER'@'%' IDENTIFIED BY '$pw';
ALTER USER '$WIKI_DB_USER'@'%' IDENTIFIED BY '$pw';
GRANT ALL PRIVILEGES ON \`$WIKI_DB_NAME\`.* TO '$WIKI_DB_USER'@'%';
FLUSH PRIVILEGES;
SQL

echo "wiki database ready: $WIKI_DB_NAME (user $WIKI_DB_USER)"
