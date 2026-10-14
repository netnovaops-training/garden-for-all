#!/bin/bash
# shellcheck disable=SC2016,SC2034  # checks are strings run by eval (they use n0); the password is read inside its container
# Test-only: the automatic backup schedule (garden.sh backup auto, menu 6, 7) on a
# throwaway, installed garden. It adds a few test blog posts, looks what the
# schedule decides (quiet, medium, busy, the weekly one), checks the thinning of
# old automatic backups, and removes its test posts again. Prints what it found.
#   bash tools/release/tests/test-smart-backups.sh [GARDEN-FOLDER]
set -u
G=${1:-$HOME/junsgarden-for-jun}
cd "$G" || exit 1
B=$(grep -m1 '^BACKUP_DIR=' .env | cut -d= -f2-); B=${B:-./backups}
sql() { docker exec -i garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" exec mariadb -uroot -N -B juns_garden_db'; }
posts() { for i in $(seq 1 "$1"); do echo "INSERT INTO garden_blog (author, title, content) VALUES ('test', 'smart-backup test $i', 'test');"; done | sql; }
auto() { GARDEN_AUTO_EVENING=24 bash garden.sh backup auto 2>&1 | grep ' auto: ' | tail -1; }
autos() { find "$B" -maxdepth 1 -name 'junsgarden-backup_*_auto.tar' | wc -l; }
pass=0 failn=0
check() { if eval "$2"; then echo "PASS  $1"; pass=$((pass + 1)); else echo "FAIL  $1"; failn=$((failn + 1)); fi; }
until echo 'SELECT 1;' | sql >/dev/null 2>&1; do sleep 2; done
echo "DELETE FROM garden_blog WHERE title LIKE 'smart-backup test%';" | sql

echo "######## 1. a backup made by hand is the starting point"
bash garden.sh backup >/dev/null 2>&1
check "the look at the last backup was saved" '[ -s logs/backup-look/rows.at-backup ] && [ -s logs/backup-look/time.at-backup ]'

echo "######## 2. nothing changed: quiet, no backup"
n0=$(autos); line=$(auto); echo "   $line"
check "quiet, and no backup made" '[[ "$line" == *"auto: quiet (0 edit"* ]] && [ "$(autos)" = "$n0" ]'

echo "######## 3. two new posts: medium"
posts 2; line=$(auto); echo "   $line"
check "medium with 2 edits" '[[ "$line" == *"auto: medium (2 edit"* ]]'

echo "######## 4. old automatic backups, then five more posts: busy, a backup, the thinning"
for d in "10 minutes ago" "20 minutes ago" "30 minutes ago" "2 days ago" "2 days ago 1 hour ago" "10 days ago"; do
    f="$B/junsgarden-backup_test-$(date -d "$d" +%Y-%m-%d_%H%M%S)_auto.tar"; : > "$f"; touch -d "$d" "$f"
done
echo "   before: $(autos) automatic backups (6 of them are empty test files)"
posts 3; line=$(auto); echo "   $line"
check "busy with 5 edits, backing up now" '[[ "$line" == *"auto: busy (5 edit"* && "$line" == *"backing up now"* ]]'
check "a new automatic backup is there and is the newest" '[[ "$(ls -t "$B"/junsgarden-backup_*.tar | head -1)" == *_auto.tar ]] && [ -s "$(ls -t "$B"/junsgarden-backup_*.tar | head -1)" ]'
echo "   after:  $(autos) automatic backups"
check "thinned: 3 from today, 1 from 2 days ago, none from 10 days ago" \
    '[ "$(find "$B" -maxdepth 1 -name "*_auto.tar" -newermt "$(date +%Y-%m-%d)" | wc -l)" = 3 ] && [ "$(find "$B" -maxdepth 1 -name "*test-$(date -d "2 days ago" +%Y-%m-%d)*_auto.tar" | wc -l)" = 1 ] && [ "$(find "$B" -maxdepth 1 -name "*test-$(date -d "10 days ago" +%Y-%m-%d)*" | wc -l)" = 0 ]'
check "backups made by hand are kept" '[ "$(find "$B" -maxdepth 1 -name "junsgarden-backup_*.tar" ! -name "*_auto.tar" | wc -l)" -ge 1 ]'

echo "######## 5. right after that backup: quiet again"
line=$(auto); echo "   $line"
check "quiet after the backup" '[[ "$line" == *"auto: quiet (0 edit"* ]]'

echo "######## 6. quiet for 8 days: the weekly backup"
date -d '8 days ago' +%s > logs/backup-look/time.at-backup
line=$(auto); echo "   $line"
check "the weekly backup is due" '[[ "$line" == *"auto: quiet"* && "$line" == *"backing up now"* ]]'

echo "######## 7. switched off: no backup"
touch .garden-off; line=$(auto); rm -f .garden-off; echo "   $line"
check "nothing while switched off" '[[ "$line" == *"switched off"* ]]'

echo "######## 8. the schedule in cron"
check "cron has the hourly automatic line" 'crontab -l 2>/dev/null | grep -q "garden.sh backup auto"'

echo "######## tidy up: the test posts and the empty test files go"
echo "DELETE FROM garden_blog WHERE title LIKE 'smart-backup test%';" | sql
find "$B" -maxdepth 1 -name 'junsgarden-backup_test-*' -delete
echo "RESULT: $pass passed, $failn failed"
[ "$failn" = 0 ]
