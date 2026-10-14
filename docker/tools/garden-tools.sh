#!/bin/bash
# =============================================================================
#  garden-tools: the jobs that run INSIDE the toolbox container.
#
#  You do not run this file yourself. garden.sh (Ubuntu) and Garden-Menu.ps1
#  (Windows) call it through Docker, for example:
#      docker compose run --rm tools backup
#
#  Inside the container your folders always have the same names, whatever
#  you called them on the computer (the .env file decides that):
#      /garden/images   your photos                    (IMAGES_DIR in .env)
#      /garden/data     comments, csv files            (DATA_DIR)
#      /garden/cashe    the AI cache                   (CACHE_DIR)
#      /garden/n8n      n8n workflows and logins       (N8N_DIR)
#      /garden/auth     the sign-in page's settings    (AUTH_DIR)
#      /backups         where backups go               (BACKUP_DIR)
#  That is why a backup made on one computer restores on another, even if
#  the folders there have different names.
#
#  Jobs:
#      backup [label]          save everything into one .tar file
#      restore FILE|latest     put a backup back (makes a safety backup first)
#      verify FILE|latest      check a backup is complete, change nothing
#      list                    list the backups
#      set-password NAME       set the password for the website login AND
#                              the sign-in page (one password for both)
#      add-guest NAME EMAIL    a guest account on the sign-in page only
#      remove-user NAME        take a person's logins away (website + sign-in page)
#      logout-all              sign everybody out of the website
#      list-users              who can sign in
#      test-email ADDRESS      send a test email with the Gmail settings
#      wait-db                 wait until the database answers
#      plant-count             print how many plants the database has
# =============================================================================
set -euo pipefail

DB_HOST=${DB_HOST:-db}
DB_NAME=${DB_NAME:-juns_garden_db}        # the garden database
WIKI_DB=${WIKI_DB_NAME:-junsgarden_wiki}  # the wiki's database (if the wiki is installed)
G=/garden
B=/backups
PARTS="images data cashe n8n auth"        # folders a backup holds (when they exist)

say()  { printf '== %s\n' "$*"; }
die()  { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

# The database root password comes from .env through Docker. MYSQL_PWD is
# the variable the mariadb programs read, so it never shows in a command line.
[ -n "${DB_ROOT_PASSWORD:-}" ] || die "DB_ROOT_PASSWORD is not set (is the .env file there?)"
export MYSQL_PWD="$DB_ROOT_PASSWORD"
sql() { mariadb -h "$DB_HOST" -uroot "$@"; }

# Give files back to the person who owns the project folder (Ubuntu only).
# Without this, files made by the container would belong to "root".
give_back() {
    if [ -n "${HOST_UID:-}" ] && [ "${HOST_UID}" != "0" ]; then
        chown "${HOST_UID}:${HOST_GID:-$HOST_UID}" "$@" 2>/dev/null || true
    fi
}

wait_db() {
    for _ in $(seq 1 60); do
        sql -e 'SELECT 1' >/dev/null 2>&1 && return 0
        sleep 2
    done
    die "the database did not answer for 2 minutes. Is garden-db running?"
}

plant_count() {
    sql -N -e "SELECT COUNT(*) FROM \`$DB_NAME\`.data_source" 2>/dev/null || echo 0
}
db_exists() {
    [ "$(sql -N -e "SELECT COUNT(*) FROM information_schema.schemata WHERE schema_name='$1'")" = 1 ]
}

# Turn "latest" or a file name into the full path of a backup file.
pick() {
    local want=${1:-}
    [ -n "$want" ] || die "say which backup: a file name, or 'latest'"
    if [ "$want" = latest ]; then
        want=$(find "$B" -maxdepth 1 -name 'junsgarden-backup_*.tar' -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)
        [ -n "$want" ] || die "there are no backups in the backups folder yet"
    fi
    case "$want" in /*) ;; *) want="$B/$(basename "$want")" ;; esac
    [ -f "$want" ] || die "no such backup: $(basename "$want")"
    printf '%s' "$want"
}

# Unpack a backup into a temporary folder and check every piece against
# its fingerprint (SHA256SUMS). Prints the folder it unpacked into.
unpack_and_check() {
    local file=$1 tmp dir
    tmp=$(mktemp -d /tmp/restore.XXXXXX)
    tar -xf "$file" -C "$tmp" || die "could not open $(basename "$file") (damaged or not a garden backup)"
    dir=$(find "$tmp" -mindepth 1 -maxdepth 1 -type d | head -1)
    [ -f "$dir/SHA256SUMS" ] || die "$(basename "$file") has no SHA256SUMS: not a garden backup"
    ( cd "$dir" && sha256sum -c --quiet SHA256SUMS ) >&2 || die "a piece of $(basename "$file") is damaged. Use an older backup."
    printf '%s' "$dir"
}

dump() {   # dump one database, compressed
    mariadb-dump -h "$DB_HOST" -uroot --single-transaction --routines --triggers --add-drop-table "$1" | gzip
}

do_backup() {
    local label=${1:-} stamp name work count bytes part
    stamp=$(date +%Y-%m-%d_%H%M)
    name="junsgarden-backup_${stamp}${label:+_$label}"
    work=$(mktemp -d /tmp/backup.XXXXXX)/$name
    mkdir -p "$work" "$B"

    wait_db
    say "Saving the garden database"
    dump "$DB_NAME" > "$work/database.sql.gz"
    {
        echo "Jun's Garden backup"
        echo "Name:        $name"
        echo "Made:        $(date '+%Y-%m-%d %H:%M %Z')"
        echo "From:        ${FROM_COMPUTER:-unknown computer}"
        echo "Plants:      $(plant_count)"
        echo
        echo "Pieces (each one is checked by SHA256SUMS when you restore):"
        echo "  database.sql.gz   the garden database ($DB_NAME)"
    } > "$work/MANIFEST.txt"
    if db_exists "$WIKI_DB"; then
        say "Saving the wiki database"
        dump "$WIKI_DB" > "$work/wiki.sql.gz"
        echo "  wiki.sql.gz       the wiki's pages ($WIKI_DB)" >> "$work/MANIFEST.txt"
    fi

    for part in $PARTS; do
        [ -d "$G/$part" ] || continue          # that part is not installed here
        say "Saving $part"
        tar -cf "$work/$part.tar" -C "$G/$part" .
        count=$(find "$G/$part" -type f | wc -l)
        bytes=$(du -sh "$G/$part" | cut -f1)
        printf '  %-16s  %s files, %s\n' "$part.tar" "$count" "$bytes" >> "$work/MANIFEST.txt"
    done
    {
        echo
        echo "Folder names on the computer that made it (from .env):"
        echo "  IMAGES_DIR=${IMAGES_DIR:-?}  DATA_DIR=${DATA_DIR:-?}  CACHE_DIR=${CACHE_DIR:-?}"
        echo "These do not need to match when you restore: the backup stores the"
        echo "pieces by their job (images, data, ...), not by folder name."
        echo
        echo "Restore:  Ubuntu: bash garden.sh restore $name.tar"
        echo "          Windows: the Garden menu, 7 (Restore a backup)"
    } >> "$work/MANIFEST.txt"

    # nullglob: a pattern that matches nothing becomes nothing (not an error).
    ( cd "$work" && shopt -s nullglob && sha256sum ./*.gz ./*.tar MANIFEST.txt > SHA256SUMS )
    tar -cf "$B/$name.tar" -C "$(dirname "$work")" "$name"
    rm -rf "$(dirname "$work")"
    give_back "$B/$name.tar"

    # Only the automatic nightly backups are cleaned up (older than 7 days: each
    # backup holds every photo, so 30 of them would fill a small disk).
    # Backups you make yourself are never deleted for you.
    find "$B" -maxdepth 1 -name 'junsgarden-backup_*_nightly.tar' -mtime +7 -delete

    say "Saved: $name.tar ($(du -h "$B/$name.tar" | cut -f1))"
    cat "$B/$name.tar" >/dev/null   # read it back once: a bad disk shows up now, not on restore day
}

do_verify() {
    local file dir
    file=$(pick "${1:-}")
    dir=$(unpack_and_check "$file")
    cat "$dir/MANIFEST.txt"
    echo
    say "OK: every piece of $(basename "$file") matches its fingerprint."
    rm -rf "$(dirname "$dir")"
}

load_db() {   # load_db NAME FILE.sql.gz : replace a database with a dump
    sql -e "DROP DATABASE IF EXISTS \`$1\`; CREATE DATABASE \`$1\` CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;"
    # Newer MariaDB versions start a dump with a "sandbox" line that older
    # ones do not understand. Dropping it is safe: it is only a guard.
    gunzip -c "$2" | sed '1{/^\/\*M!999999/d;}' | sql "$1"
}

do_restore() {
    local file dir part
    file=$(pick "${1:-}")
    say "Checking $(basename "$file") before touching anything"
    dir=$(unpack_and_check "$file")
    wait_db

    if [ "${NO_SAFETY:-0}" != 1 ] && [ "$(plant_count)" != 0 ]; then
        say "Making a safety backup of what is here now (label: before-restore)"
        do_backup before-restore
    fi

    say "Loading the garden database"
    load_db "$DB_NAME" "$dir/database.sql.gz"
    if [ -f "$dir/wiki.sql.gz" ]; then
        say "Loading the wiki database"
        load_db "$WIKI_DB" "$dir/wiki.sql.gz"
    fi

    for part in $PARTS; do
        if [ -f "$dir/$part.tar" ] && [ -d "$G/$part" ]; then
            say "Putting back $part"
            find "$G/$part" -mindepth 1 -delete
            tar -xf "$dir/$part.tar" -C "$G/$part"
            # The programs run as normal users in their containers and must be
            # able to add and remove files here, so everyone may write.
            # Except the sign-in page's folder: it holds the password hashes
            # and the email password, so only its owner may read it (the
            # sign-in page runs as the administrator and reads it anyway).
            if [ "$part" = auth ]; then
                chmod -R u+rwX,go-rwx "$G/$part"
            else
                chmod -R a+rwX "$G/$part"
            fi
        fi
    done
    rm -rf "$(dirname "$dir")"

    say "Restored $(basename "$file"): $(plant_count) plants, $(find "$G/images" -type f | wc -l) image files"
}

# ---------- people: logins for the website and the sign-in page ----------------

# Find the email for NAME in GARDEN_USERS (Name=email,Name2=email2).
email_of() {
    local pair
    for pair in ${GARDEN_USERS//,/ }; do
        [ "${pair%%=*}" = "$1" ] && { printf '%s' "${pair#*=}"; return; }
    done
}

lower() { printf '%s' "$1" | tr '[:upper:]' '[:lower:]'; }

# Read a new password twice with nothing on screen (or one line from
# standard input, for automatic tests). Prints the Argon2id hash only.
# Argon2id with a random salt, 64 MB memory, 3 passes, 4 lanes: the strength
# Authelia recommends. The password itself is never stored or shown.
new_hash() {
    local who=$1 pw pw2 salt
    if [ -t 0 ]; then
        printf 'New password for %s (10+ characters, nothing shows as you type): ' "$who" >&2
        stty -echo; read -r pw; stty echo; printf '\n' >&2
        printf 'Type it again: ' >&2
        stty -echo; read -r pw2; stty echo; printf '\n' >&2
        [ "$pw" = "$pw2" ] || die "the two passwords did not match"
    else
        read -r pw
    fi
    [ "${#pw}" -ge 10 ] || die "too short: use at least 10 characters"
    salt=$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')
    printf '%s' "$pw" | argon2 "$salt" -id -t 3 -m 16 -p 4 -e
}

# Remove NAME from the sign-in page's user list (users.yml).
auth_remove() {
    local file=$G/auth/users.yml key tmp
    [ -f "$file" ] || return 0
    key=$(lower "$1"); tmp=$(mktemp)
    awk -v k="  $key:" 'BEGIN{skip=0} $0==k{skip=1; next} skip && /^  [^ ]/{skip=0} !skip' "$file" > "$tmp"
    cat "$tmp" > "$file"; rm -f "$tmp"
}

# Put NAME into the sign-in page's user list with a password hash and a group:
#   gardeners = may use the garden tools     guests = may sign in, nothing more yet
# The sign-in page notices the change by itself (watch: true).
auth_user() {
    local name=$1 hash=$2 email=$3 group=$4 file=$G/auth/users.yml key tmp
    [ -d "$G/auth" ] || return 0              # the sign-in page is not set up yet
    key=$(lower "$name")
    [ -f "$file" ] || printf 'users:\n' > "$file"
    # Build the whole new list first, then write it in ONE go, so the sign-in
    # page never reads a half-written list. (garden.sh also restarts it.)
    tmp=$(mktemp)
    awk -v k="  $key:" 'BEGIN{skip=0} $0==k{skip=1; next} skip && /^  [^ ]/{skip=0} !skip' "$file" > "$tmp"
    {
        printf '  %s:\n' "$key"
        printf '    disabled: false\n'
        printf "    displayname: '%s'\n" "$name"
        printf "    password: '%s'\n" "$hash"
        printf "    email: '%s'\n" "${email:-$key@localhost}"
        printf '    groups:\n      - %s\n' "$group"
    } >> "$tmp"
    cat "$tmp" > "$file"; rm -f "$tmp"
    chmod 600 "$file"
    say "Sign-in page: $key is in the group '$group'."
}

do_set_password() {
    local name=${1:-} hash n
    [ -n "$name" ] || die "usage: set-password YourName"
    case "$name" in *[!A-Za-z]*) die "the name is letters only" ;; esac
    wait_db
    n=$(sql -N "$DB_NAME" -e "SELECT COUNT(*) FROM garden_users WHERE username='$name'")
    [ "$n" = 1 ] || die "no login called '$name'. Add the person first (Users menu), then try again."
    hash=$(new_hash "$name") || exit 1
    printf "UPDATE garden_users SET pass_hash='%s', reset_hash=NULL, reset_expires=NULL WHERE username='%s';\n" \
        "$hash" "$name" | sql "$DB_NAME"
    say "Website password set for $name."
    auth_user "$name" "$hash" "$(email_of "$name")" gardeners
}

do_add_guest() {
    local name=${1:-} email=${2:-} hash
    case "$name" in *[!A-Za-z]*|"") die "usage: add-guest Name email (name: letters only)" ;; esac
    [ -d "$G/auth" ] && [ -f "$G/auth/configuration.yml" ] || die "the sign-in page is not set up yet (front door step)"
    hash=$(new_hash "$name") || exit 1
    auth_user "$name" "$hash" "$email" guests
}

do_remove_user() {
    local name=${1:-}
    case "$name" in *[!A-Za-z]*|"") die "usage: remove-user Name" ;; esac
    wait_db
    # The website: delete the login row and every saved sign-in of that person.
    printf "DELETE FROM garden_sessions WHERE username='%s'; DELETE FROM garden_users WHERE username='%s';\n" \
        "$name" "$name" | sql "$DB_NAME"
    auth_remove "$name"
    say "Removed $name from the website's logins and the sign-in page."
}

do_logout_all() {
    wait_db
    sql "$DB_NAME" -e "DELETE FROM garden_sessions;"
    say "Everyone is signed out of the website (the phone app too)."
}

do_list_users() {
    wait_db
    echo "WEBSITE LOGINS (may edit the garden):"
    sql -t "$DB_NAME" -e "SELECT username AS name, email, IF(pass_hash IS NULL,'no password yet','password set') AS password,
        (SELECT COUNT(*) FROM garden_sessions s WHERE s.username = u.username AND s.expires_at > NOW()) AS signed_in_now
        FROM garden_users u ORDER BY username"
    echo
    echo "SIGN-IN PAGE ACCOUNTS:"
    if [ -f "$G/auth/users.yml" ]; then
        awk '/^  [^ ]/{u=$1; sub(":","",u)} /^      - /{print "  " u "   group: " $2}' "$G/auth/users.yml"
    else
        echo "  (the sign-in page is not set up yet)"
    fi
}

do_test_email() {
    local to=${1:-${SMTP_USER:-}}
    [ -n "${SMTP_HOST:-}" ] && [ -n "${SMTP_USER:-}" ] && [ -n "${SMTP_PASSWORD:-}" ] ||
        die "email is not set up yet (SMTP_HOST, SMTP_USER and SMTP_PASSWORD in .env)"
    [ -n "$to" ] || die "say who gets the test email"
    printf 'From: %s\nTo: %s\nSubject: Jun'"'"'s Garden test email\n\nIf you can read this, the garden can send email.\n' \
        "$SMTP_USER" "$to" > /tmp/mail.txt
    # STARTTLS on port 587, the same way the website sends its emails.
    curl -sS --ssl-reqd --url "smtp://${SMTP_HOST}:${SMTP_PORT:-587}" \
        --user "${SMTP_USER}:${SMTP_PASSWORD}" --mail-from "$SMTP_USER" --mail-rcpt "$to" \
        --upload-file /tmp/mail.txt || die "Gmail did not accept it. Check the app password (guide 03 part C)."
    say "Sent. Look in $to's inbox (and the spam folder)."
}

case "${1:-help}" in
    backup)       do_backup "${2:-}" ;;
    restore)      do_restore "${2:-}" ;;
    verify)       do_verify "${2:-}" ;;
    list)         ls -lh "$B"/junsgarden-backup_*.tar 2>/dev/null || echo "No backups yet." ;;
    set-password) do_set_password "${2:-}" ;;
    add-guest)    do_add_guest "${2:-}" "${3:-}" ;;
    remove-user)  do_remove_user "${2:-}" ;;
    logout-all)   do_logout_all ;;
    list-users)   do_list_users ;;
    test-email)   do_test_email "${2:-}" ;;
    wait-db)      wait_db; echo "database is ready" ;;
    plant-count)  wait_db; plant_count ;;
    *)            sed -n '2,36p' "$0" ;;
esac
