#!/bin/bash
# shellcheck disable=SC2016,SC2012  # each check runs in its own bash -c on purpose; ls only shows a folder
# Test-only: the rename / move tool and the on / off switch (menu M, 18 and 14),
# on a throwaway, installed garden in ~/junsgarden-for-jun. Each step changes
# something and then puts it back, and prints what it found. It rebuilds the
# website a few times (a few minutes each). This is the test that ran before
# release (CHANGES.md, Part 6).
set -u
G=~/junsgarden-for-jun
cd $G || exit 1
sql() { docker exec -i garden-db sh -c 'MYSQL_PWD="$MARIADB_ROOT_PASSWORD" exec mariadb -uroot -N -B juns_garden_db'; }
code() { curl -s -o /dev/null -w '%{http_code}' "http://localhost:8030/$1"; }
for _ in $(seq 1 60); do curl -fs -o /dev/null http://localhost:8030/healthz && break; sleep 3; done
until echo 'SELECT 1;' | sql >/dev/null 2>&1; do sleep 2; done

echo "######## A. a photo folder: Gallery -> Photos, and back"
printf '1\nlocal/images/Gallery\nPhotos\ny\nn\ny\n' | bash garden.sh rename 2>&1 | grep -E 'place|Renamed|Changed|Checked|STOPPED|WARN' | head -8
f=$(find local/images/Photos -maxdepth 1 -name '*.webp' | head -1); f=${f#local/}
echo "a photo under the new name: $(code "$f")   IMAGE_FOLDERS: $(grep IMAGE_FOLDERS .env)"
printf '1\nlocal/images/Photos\nGallery\ny\nn\ny\n' | bash garden.sh rename >/dev/null 2>&1
ls local/images | tr '\n' ' '; echo

echo "######## B. a plant photo named in the database"
P=$(echo "SELECT picture FROM data_source WHERE picture LIKE '%.webp%' LIMIT 50;" | sql | tr ',' '\n' | while read -r x; do [ -f "local/images/Database/$x" ] && { echo "$x"; break; }; done)
N=${P%.webp}-renamed.webp
printf '1\nlocal/images/Database/%s\n%s\ny\nn\ny\n' "$P" "$N" | bash garden.sh rename 2>&1 | grep -E '      [a-z_]+\.|row|Renamed|STOPPED|WARN' | head -8
echo "rows with the new name: $(echo "SELECT COUNT(*) FROM data_source WHERE picture LIKE '%$N%';" | sql)   new address: $(code "images/Database/$N")   old: $(code "images/Database/$P")"

echo "######## C. a restore brings the old name back: redo"
mv "local/images/Database/$N" "local/images/Database/$P"
echo "UPDATE data_source SET picture = REPLACE(picture, '$N', '$P') WHERE picture LIKE '%$N%';" | sql
printf 'y\ny\ny\ny\n' | bash garden.sh redo-renames 2>&1 | grep -E 'old name|row|Renamed|STOPPED' | head
echo "after redo: new file there: $([ -f "local/images/Database/$N" ] && echo yes)   rows: $(echo "SELECT COUNT(*) FROM data_source WHERE picture LIKE '%$N%';" | sql)"
printf '1\nlocal/images/Database/%s\n%s\ny\nn\ny\n' "$N" "$P" | bash garden.sh rename >/dev/null 2>&1

echo "######## D. a page: privacy.html -> privacy-policy.html, and back"
printf '1\nsite/privacy.html\nprivacy-policy.html\ny\nn\ny\n' | bash garden.sh rename 2>&1 | grep -E '^      [a-z.]|place|Checked|STOPPED' | head -8
echo "new page: $(code privacy-policy.html)   old page: $(code privacy.html)   home page links to new: $(curl -s http://localhost:8030/ | grep -c 'privacy-policy.html')"
printf '1\nsite/privacy-policy.html\nprivacy.html\ny\nn\ny\n' | bash garden.sh rename >/dev/null 2>&1
echo "refused (a folder the programs use):"; printf '1\nlocal/images/plan/tags\n' | bash garden.sh rename 2>&1 | grep STOPPED
echo "refused (the programs themselves):"; printf '1\nrust/junsgarden\n' | bash garden.sh rename 2>&1 | grep 'programs themselves'

echo "######## E. a big folder moved and back"
printf '2\n4\n./local/cashe2\ny\ny\n' | bash garden.sh rename 2>&1 | grep -E 'Moved|STOPPED'
printf '2\n4\n./local/cashe\ny\ny\n' | bash garden.sh rename 2>&1 | grep -E 'Moved|STOPPED'
grep '^CACHE_DIR' .env; echo "site $(code healthz)"

echo "######## F. clean-up: an old garden version goes, another project's image stays"
docker pull -q hello-world >/dev/null; docker tag hello-world junsgarden/rs:old-test
printf 'y\n' | bash garden.sh cleanup 2>&1 | grep -E '^      |removed|STOPPED' | head
echo "hello-world kept: $(docker images -q hello-world | wc -l)   old garden version gone: $([ -z "$(docker images -q junsgarden/rs:old-test)" ] && echo yes)"
docker rmi hello-world >/dev/null 2>&1

echo "######## G. check-up: a stopped website comes back, a finished helper is left alone"
docker stop garden-site >/dev/null; echo "website stopped: $(code healthz)"
bash garden.sh checkup 2>&1 | tail -2; sleep 5; echo "site $(code healthz)"
bash garden.sh checkup 2>&1 | tail -1

echo "######## H. everything off, then on"
printf 'y\n' | bash garden.sh off 2>&1 | grep -E 'Off|STOPPED'
echo "running: $(docker ps -q --filter label=com.docker.compose.project=junsgarden | wc -l)   photos still: $(find local/images -type f | wc -l)"
bash garden.sh backup nightly 2>&1 | tail -1
printf 'n\n' | bash garden.sh on 2>&1 | grep -E 'answers|STOPPED'

echo "######## I. the whole garden folder moved, and back onto its signpost"
printf '3\n%s/garden-moved\ny\ny\ny\n' "$HOME" | bash garden.sh rename 2>&1 | grep -E 'arrived|Signpost|lives|STOPPED'
crontab -l | grep 'garden.sh backup'; echo "site $(code healthz)"
cd ~/garden-moved && printf '3\n%s/junsgarden-for-jun\ny\ny\nn\n' "$HOME" | bash garden.sh rename 2>&1 | grep -E 'arrived|lives|STOPPED'
cd $G && crontab -l | grep 'garden.sh backup'; echo "site $(code healthz)"
bash garden.sh renames | tail -3
bash garden.sh check 2>&1 | grep -E 'RESULT'
