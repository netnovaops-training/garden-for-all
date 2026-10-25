#!/bin/sh
# Side-by-side lab for the "official Rust pages" rewrite. Run ON THE NUC from ~/garden-rust-pages.
#
#   sh lab.sh db            fresh throwaway database: base_db and cand_db, each = the repo's
#                           db/schema.sql + the made-up sample (make_sample.py). No real data.
#   sh lab.sh build base    build today's code (src-base/, from the `fixes` branch)
#   sh lab.sh build cand    build the rewrite (src-cand/, from the rust-pages worktree)
#   sh lab.sh up base|cand  (re)start that side: base on 127.0.0.1:18101, cand on 127.0.0.1:18102
#   sh lab.sh tester        give the lab's Tester account a fresh random password (both sides)
#   sh lab.sh round         db + both sides + tester + every list -> one PASS/FAIL line per list
#   sh lab.sh shots         screenshots of both sides compared (shots.py) -> shots.log
#   sh lab.sh check [side]  fmt, clippy (warnings are errors) and the tests
#   sh lab.sh down          remove everything this lab made (live and test sites never touched)
#
# Isolation: own docker network (gp-net, internal), own MariaDB (gp-db, no published port, random
# root password in .dbpw, mode 600), the repo's own site/ folder read-only, made-up photos and
# data (sample/), ports on 127.0.0.1 only. Nothing here reads live data or live credentials.
set -eu
cd "$(dirname "$0")"
NET=gp-net
DB=gp-db
USERS="Gardener=gardener@example.test,Tester=tester@example.test"

dbpw() { cat .dbpw; }
sql() { docker exec -i $DB mariadb -uroot -p"$(dbpw)" "$@"; }
rnd() { head -c "$1" /dev/urandom | od -An -tx1 | tr -d ' \n'; }

case "${1:-}" in
db)
    docker network inspect $NET >/dev/null 2>&1 || docker network create --internal $NET >/dev/null
    docker rm -f $DB >/dev/null 2>&1 || true
    [ -s .dbpw ] || { umask 077; rnd 24 > .dbpw; }
    docker run -d --name $DB --network $NET -e MARIADB_ROOT_PASSWORD="$(dbpw)" \
        --tmpfs /var/lib/mysql:rw,size=2g mariadb:lts-noble >/dev/null
    i=0; until sql -e 'SELECT 1' >/dev/null 2>&1; do
        i=$((i+1)); [ $i -lt 60 ] || { echo "gp-db did not start"; exit 1; }; sleep 2; done
    # The schema's views are owned by `dbadmin`; without that user they fail to run.
    sql -e "CREATE USER IF NOT EXISTS 'dbadmin'@'%' IDENTIFIED BY '$(rnd 18)'; GRANT ALL ON *.* TO 'dbadmin'@'%'"
    rm -rf sample && python3 src-cand/docs/rust-pages/make_sample.py sample >/dev/null
    for d in base_db cand_db; do
        # schema.sql creates and uses `juns_garden_db`; each side gets its own copy under its name.
        sed "s/juns_garden_db/$d/g" src-cand/db/schema.sql | sql > db.log 2>&1 || { echo "FAIL schema (db.log)"; exit 1; }
        sql "$d" < sample/sample-data.sql > db.log 2>&1 || { echo "FAIL sample data: $(tail -c 300 db.log)"; exit 1; }
    done
    echo "gp-db ready: base_db and cand_db = repo schema + sample"
    ;;
build)
    side=$2
    docker build -q --build-arg APP=junsgarden -t "gardenlab/$side" "src-$side/rust" >/dev/null
    echo "built gardenlab/$side"
    ;;
up)
    side=$2
    port=18101; [ "$side" = cand ] && port=18102
    rm -rf "$side-data" "$side-ai-cache" "$side-www" && cp -r sample/data "$side-data" && mkdir -p "$side-ai-cache"
    # The site folder = the repo's site/ plus the sample photos (an assembled copy, because the
    # photos cannot be mounted inside a read-only site/ that has no images folder).
    # The pages put each file's time in `?v=` on every script and stylesheet; the two sources come
    # from git archive and a working copy (different times), so both get one fixed time.
    cp -r "src-$side/site" "$side-www" && cp -r sample/images "$side-www/images"
    find "$side-www" -exec touch -h -d @1790000000 {} +
    chmod -R a+rwX "$side-data" "$side-ai-cache" "$side-www/images"
    docker rm -f "gp-$side" >/dev/null 2>&1 || true
    # Two networks: gp-net to reach the lab database, and a plain bridge for the published port.
    docker create --name "gp-$side" --network bridge -p "127.0.0.1:$port:8030" \
        -e JUNSGARDEN_DATABASE_URL="mysql://root:$(dbpw)@$DB:3306/${side}_db" \
        -e APP_PORT=8030 -e TZ=America/New_York \
        -e JUNSGARDEN_WWW_DIR=/www -e JUNSGARDEN_DATA_DIR=/data -e JUNSGARDEN_AI_CACHE_DIR=/ai-cache \
        -e JUNSGARDEN_USERS="$USERS" -e JUNSGARDEN_PUBLIC_URL="http://127.0.0.1:$port" \
        -e JUNSGARDEN_INGEST_URL="http://127.0.0.1:9" -e OLLAMA_URL="http://127.0.0.1:9/api/generate" \
        -v "$PWD/$side-www:/www" \
        -v "$PWD/$side-data:/data" -v "$PWD/$side-ai-cache:/ai-cache" \
        -v "$PWD/src-$side/START-HERE:/guides:ro" \
        "gardenlab/$side" >/dev/null
    docker network connect $NET "gp-$side"
    docker start "gp-$side" >/dev/null
    i=0; until curl -fs "http://127.0.0.1:$port/healthz" >/dev/null 2>&1; do
        i=$((i+1)); [ $i -lt 40 ] || { docker logs --tail 20 "gp-$side"; exit 1; }; sleep 1; done
    echo "gp-$side up on 127.0.0.1:$port"
    ;;
tester)
    # A fresh random password for the lab's Tester (never printed), hashed the way the repo's
    # toolbox does (argon2 CLI, docker/tools/garden-tools.sh), set on both sides.
    umask 077; printf 'TEST_PW=%s-garden\n' "$(rnd 12)" > test-login.env; umask 022
    pw=$(sed -n 's/^TEST_PW=//p' test-login.env)
    hash=$(printf '%s' "$pw" | docker run --rm -i alpine:3.22 sh -c \
        "apk add -q argon2 >/dev/null && argon2 \"\$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')\" -id -t 3 -m 16 -p 4 -e")
    for d in base_db cand_db; do
        sql "$d" -e "UPDATE garden_users SET pass_hash='$hash' WHERE username='Tester'; DELETE FROM login_attempts;"
    done
    ;;
round)
    # One full side-by-side round from a clean start. One line per list; details in round.log.
    sh "$0" db >/dev/null
    sh "$0" up base >/dev/null && sh "$0" up cand >/dev/null && sh "$0" tester
    : > round.log
    fail=0
    for list in urls-port.txt urls-extra.txt urls-app.txt urls-post.txt; do
        if python3 sbs.py "$list" >> round.log 2>&1; then r=PASS; else r=FAIL; fail=1; fi
        printf '%s %-15s %s\n' "$r" "$list" "$(tail -1 round.log)"
    done
    [ $fail = 0 ] || grep -E "^DIFF" -A3 round.log | cut -c1-200 | head -30
    ;;
shots)
    # Screenshots of both sides (shots.py) in a Playwright container on the host network
    # (it only talks to 127.0.0.1:18101/18102). Summary in shots.log. Both sides are restarted
    # first, so their folder mounts are current after a sync.
    sh "$0" up base >/dev/null && sh "$0" up cand >/dev/null && sh "$0" tester
    docker run --rm -v "$PWD:/w" alpine rm -rf /w/shots
    docker run --rm --network host -v "$PWD:/w" -w /w -e OWNER="$(id -u):$(id -g)" \
        mcr.microsoft.com/playwright/python:v1.56.0-noble \
        sh -c 'pip install -q --break-system-packages playwright==1.56.0 pillow >/dev/null 2>&1;
               python3 shots.py pages-visual.txt shots; chown -R "$OWNER" shots' > shots.log 2>&1
    echo "shots: $(grep -ac ' 0\.000%' shots.log) identical, $(grep -aE '^ *[0-9]+ +[0-9]+ ' shots.log | grep -avc ' 0\.000%') not"
    ;;
address)
    # The garden's own address, end to end (addr_test.py): the new code living in a folder
    # (/garden) behind Caddy on 127.0.0.1:18103, an old address on :18104 that forwards to it,
    # a fresh signing key, and a bed that gets renamed. Uses cand_db and the cand files.
    docker rm -f gp-sub gp-proxy >/dev/null 2>&1 || true
    sql cand_db -e "INSERT IGNORE INTO garden_bed (name, shape) VALUES ('Lab Bed', '')"
    bed=$(sql cand_db -N -e "SELECT id FROM garden_bed WHERE name IN ('Lab Bed','Lab Bed Renamed') LIMIT 1")
    sql cand_db -e "UPDATE garden_bed SET name='Lab Bed Renamed' WHERE id=$bed"
    docker run -d --name gp-sub --network $NET \
        -e JUNSGARDEN_DATABASE_URL="mysql://root:$(dbpw)@$DB:3306/cand_db" -e APP_PORT=8030 -e TZ=America/New_York \
        -e JUNSGARDEN_WWW_DIR=/www -e JUNSGARDEN_DATA_DIR=/data -e JUNSGARDEN_AI_CACHE_DIR=/ai-cache \
        -e JUNSGARDEN_USERS="$USERS" -e JUNSGARDEN_PUBLIC_URL="http://127.0.0.1:18103/garden" \
        -e JUNSGARDEN_SIGNING_KEY="$(rnd 32)" \
        -e JUNSGARDEN_INGEST_URL="http://127.0.0.1:9" -e OLLAMA_URL="http://127.0.0.1:9/api/generate" \
        -v "$PWD/cand-www:/www" -v "$PWD/cand-data:/data" -v "$PWD/cand-ai-cache:/ai-cache" \
        -v "$PWD/src-cand/START-HERE:/guides:ro" gardenlab/cand >/dev/null
    docker create --name gp-proxy -p 127.0.0.1:18103:18103 -p 127.0.0.1:18104:18104 \
        -v "$PWD/src-cand/docs/rust-pages/lab-proxy.Caddyfile:/etc/caddy/Caddyfile:ro" caddy:2-alpine >/dev/null
    docker network connect $NET gp-proxy && docker start gp-proxy >/dev/null
    i=0; until curl -fs http://127.0.0.1:18103/garden/healthz >/dev/null 2>&1; do
        i=$((i+1)); [ $i -lt 40 ] || { docker logs --tail 15 gp-sub; docker logs --tail 15 gp-proxy; exit 1; }; sleep 1; done
    docker run --rm --network host -v "$PWD/src-cand/docs/rust-pages:/t:ro" python:3.14-slim \
        sh -c "pip install -q cryptography >/dev/null 2>&1; python /t/addr_test.py $bed 'Lab Bed' 'Lab Bed Renamed'" > address.log 2>&1
    grep -E "^(FAIL|[0-9]+/[0-9]+ passed)" address.log
    docker rm -f gp-sub gp-proxy >/dev/null 2>&1 || true
    ;;
check)
    side=${2:-cand}
    docker run --rm -v "$PWD/src-$side/rust:/build" -w /build \
        -v gp-cargo-registry:/usr/local/cargo/registry -v gp-target-$side:/build/target \
        rust:1.99-trixie sh -c 'rustup component add rustfmt clippy >/dev/null 2>&1;
            cargo fmt --all --check >/dev/null 2>&1 && echo "PASS fmt" || echo "FAIL fmt";
            cargo clippy --locked -p junsgarden --all-targets -q -- -D warnings > /build/target/clippy.log 2>&1 && echo "PASS clippy" || echo "FAIL clippy ($(grep -c "^error" /build/target/clippy.log) lints)";
            cargo audit --version >/dev/null 2>&1 || cargo install -q --locked cargo-audit >/dev/null 2>&1;
            cargo audit -q > /build/target/audit.log 2>&1 && echo "PASS audit" || echo "FAIL audit ($(grep -c "^ID:" /build/target/audit.log) advisories)";
            cargo test --locked -p junsgarden > /build/target/test.log 2>&1 && echo "PASS tests $(grep -o "[0-9]* passed" /build/target/test.log | head -1)" || echo "FAIL tests"'
    ;;
down)
    docker rm -f gp-base gp-cand $DB >/dev/null 2>&1 || true
    docker network rm $NET >/dev/null 2>&1 || true
    echo "lab removed"
    ;;
*)
    sed -n '2,15p' "$0"; exit 2 ;;
esac
