#!/bin/sh
# The throwaway garden the Add-plant guide's pictures are taken from: this source folder's own
# site, the repo's schema and the made-up sample data (docs/rust-pages/make_sample.py). Never
# anyone's real data, and no published port: the picture taker joins the same private network.
#   sh walkthrough/streamlit/lab.sh up      build + start (pa-net, pa-db, pa-site)
#   sh walkthrough/streamlit/lab.sh down    remove everything it made
# Two accounts: Jun (the owner: the first account) and Helper (to show a non-owner).
set -u
# On Windows (Git Bash) Docker wants D:/... paths, and container paths must stay as written.
export MSYS_NO_PATHCONV=1
here=$(cd "$(dirname "$0")" && { pwd -W 2>/dev/null || pwd; })
src=$(cd "$here/../.." && { pwd -W 2>/dev/null || pwd; })
work="$here/out/lab"
NET=pa-net; DB=pa-db; SITE=pa-site
rnd() { head -c "$1" /dev/urandom | od -An -tx1 | tr -d ' \n'; }
sql() { docker exec -i $DB mariadb -uroot -p"$(cat "$work/.dbpw")" "$@"; }
case "${1:-}" in
up)
    mkdir -p "$work" && cd "$work" || exit 1
    docker rm -f $SITE $DB >/dev/null 2>&1
    docker network inspect $NET >/dev/null 2>&1 || docker network create $NET >/dev/null
    umask 077; rnd 24 > .dbpw; printf 'GUIDE_PW=%s-garden\n' "$(rnd 10)" > lab-login.env; umask 022
    docker run -d --name $DB --network $NET -e MARIADB_ROOT_PASSWORD="$(cat .dbpw)" \
        --tmpfs /var/lib/mysql:rw,size=2g mariadb:lts-noble >/dev/null || exit 1
    i=0; until sql -e 'SELECT 1' >/dev/null 2>&1; do i=$((i+1)); [ $i -lt 60 ] || { echo "FAIL db"; exit 1; }; sleep 2; done
    sql -e "CREATE USER IF NOT EXISTS 'dbadmin'@'%' IDENTIFIED BY '$(rnd 18)'; GRANT ALL ON *.* TO 'dbadmin'@'%'"
    docker build -q --build-arg APP=junsgarden -t palab/site "$src/rust" >/dev/null || { echo "FAIL build"; exit 1; }
    docker run --rm -v "$src:/src:ro" -v "$work:/w" python:3.13-slim sh -c \
        "rm -rf /w/sample && python3 /src/docs/rust-pages/make_sample.py /w/sample >/dev/null" || exit 1
    sed "s/juns_garden_db/guide_db/g" "$src/db/schema.sql" | sql >/dev/null 2>&1 || { echo "FAIL schema"; exit 1; }
    sql guide_db < sample/sample-data.sql >/dev/null || { echo "FAIL sample"; exit 1; }
    rm -rf www data cache && cp -r "$src/site" www && cp -r sample/images www/images && cp -r sample/data data && mkdir -p cache
    chmod -R a+rwX data cache www/images
    docker run -d --name $SITE --network $NET \
        -e JUNSGARDEN_DATABASE_URL="mysql://root:$(cat .dbpw)@$DB:3306/guide_db" -e APP_PORT=8030 -e TZ=America/New_York \
        -e JUNSGARDEN_WWW_DIR=/www -e JUNSGARDEN_DATA_DIR=/data -e JUNSGARDEN_AI_CACHE_DIR=/ai-cache \
        -e JUNSGARDEN_USERS="Jun=jun@example.test,Helper=helper@example.test" -e JUNSGARDEN_PUBLIC_URL="https://name.zapto.org" \
        -e JUNSGARDEN_INGEST_URL="http://127.0.0.1:9" -e OLLAMA_URL="http://127.0.0.1:9/api/generate" \
        -v "$work/www:/www" -v "$work/data:/data" -v "$work/cache:/ai-cache" -v "$src/START-HERE:/guides:ro" \
        palab/site >/dev/null || exit 1
    i=0; until sql guide_db -e "SELECT 1 FROM garden_users WHERE username='Jun'" 2>/dev/null | grep -q 1; do
        i=$((i+1)); [ $i -lt 60 ] || { docker logs --tail 20 $SITE; exit 1; }; sleep 1; done
    pw=$(sed -n 's/^GUIDE_PW=//p' lab-login.env)
    hash=$(printf '%s' "$pw" | docker run --rm -i alpine:3.22 sh -c \
        "apk add -q argon2 >/dev/null && argon2 \"\$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')\" -id -t 3 -m 16 -p 4 -e")
    sql guide_db -e "UPDATE garden_users SET pass_hash='$hash' WHERE username IN ('Jun','Helper'); DELETE FROM login_attempts;"
    echo "PASS lab up: http://$SITE:8030 on $NET (sample data only)"
    ;;
down)
    docker rm -f $SITE $DB >/dev/null 2>&1; docker network rm $NET >/dev/null 2>&1
    docker run --rm -v "$here/out:/o" alpine:3.22 rm -rf /o/lab
    echo "PASS lab removed"
    ;;
*) sed -n '2,7p' "$0"; exit 2 ;;
esac
