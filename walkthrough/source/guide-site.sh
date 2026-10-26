#!/bin/sh
# The garden the walkthrough pictures are taken from: a FRESH clone of GitHub main, the repo's
# own schema and made-up sample data (docs/rust-pages/make_sample.py), never anyone's real data.
# Runs on the lab machine next to the lab database (gp-db, from garden-rust-pages/lab.sh db),
# on 127.0.0.1:18110 only.   sh guide-site.sh up <main sha> | down
set -u
cd "$(dirname "$0")" || exit 1
LAB=$HOME/garden-rust-pages
NET=gp-net; DB=gp-db
dbpw() { cat "$LAB/.dbpw"; }
sql() { docker exec -i $DB mariadb -uroot -p"$(dbpw)" "$@"; }
rnd() { head -c "$1" /dev/urandom | od -An -tx1 | tr -d ' \n'; }
case "${1:-}" in
up)
    sha=${2:?main sha}
    docker rm -f gp-guide >/dev/null 2>&1; rm -rf src www data cache sample
    git clone -q https://github.com/netnovaops-training/garden-for-all src && git -C src checkout -q "$sha" || exit 1
    docker build -q --build-arg APP=junsgarden -t gardenlab/guide src/rust >/dev/null || exit 1
    python3 src/docs/rust-pages/make_sample.py sample >/dev/null || exit 1
    sql -e "DROP DATABASE IF EXISTS guide_db"
    sed "s/juns_garden_db/guide_db/g" src/db/schema.sql | sql >/dev/null 2>&1 || exit 1
    sql guide_db < sample/sample-data.sql >/dev/null || exit 1
    # Jun's login for the pictures: a random password on this copy only, never printed.
    umask 077; printf 'GUIDE_PW=%s-garden\n' "$(rnd 10)" > guide-login.env; umask 022
    pw=$(sed -n 's/^GUIDE_PW=//p' guide-login.env)
    hash=$(printf '%s' "$pw" | docker run --rm -i alpine:3.22 sh -c \
        "apk add -q argon2 >/dev/null && argon2 \"\$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')\" -id -t 3 -m 16 -p 4 -e")
    cp -r src/site www && cp -r sample/images www/images && cp -r sample/data data && mkdir -p cache
    chmod -R a+rwX data cache www/images
    docker create --name gp-guide --network bridge -p 127.0.0.1:18110:8030 \
        -e JUNSGARDEN_DATABASE_URL="mysql://root:$(dbpw)@$DB:3306/guide_db" -e APP_PORT=8030 -e TZ=America/New_York \
        -e JUNSGARDEN_WWW_DIR=/www -e JUNSGARDEN_DATA_DIR=/data -e JUNSGARDEN_AI_CACHE_DIR=/ai-cache \
        -e JUNSGARDEN_USERS="Jun=jun@example.test" -e JUNSGARDEN_PUBLIC_URL="https://name.zapto.org" \
        -e JUNSGARDEN_SIGNING_KEY="$(rnd 32)" \
        -e JUNSGARDEN_INGEST_URL="http://127.0.0.1:9" -e OLLAMA_URL="http://127.0.0.1:9/api/generate" \
        -v "$PWD/www:/www" -v "$PWD/data:/data" -v "$PWD/cache:/ai-cache" -v "$PWD/src/START-HERE:/guides:ro" \
        gardenlab/guide >/dev/null
    docker network connect $NET gp-guide && docker start gp-guide >/dev/null
    i=0; until curl -fs http://127.0.0.1:18110/healthz >/dev/null 2>&1; do
        i=$((i+1)); [ $i -lt 40 ] || { docker logs --tail 20 gp-guide; exit 1; }; sleep 1; done
    # The site made Jun's account at start (JUNSGARDEN_USERS); now his password on this copy.
    sql guide_db -e "UPDATE garden_users SET pass_hash='$hash' WHERE username='Jun'; DELETE FROM login_attempts;"
    echo "guide garden up (main $(git -C src rev-parse --short HEAD), sample data)"
    ;;
down)
    docker rm -f gp-guide >/dev/null 2>&1; sql -e "DROP DATABASE IF EXISTS guide_db" 2>/dev/null
    docker run --rm -v "$PWD:/w" alpine:3.22 rm -rf /w/src /w/www /w/data /w/cache /w/sample /w/guide-login.env
    echo "guide garden removed"
    ;;
*) echo "guide-site.sh up <sha> | down"; exit 2 ;;
esac
