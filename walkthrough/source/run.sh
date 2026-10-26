#!/usr/bin/env bash
# Make the two manual PDFs (English, Korean) from GitHub main with the repo's sample data,
# again for each release. From a computer with ssh to a lab machine that has Docker and the lab
# database (docs/rust-pages/lab.sh db):
#   GARDEN_LAB=user@host bash walkthrough/source/run.sh <main sha>
# 1. the lab: a fresh clone of main + sample data on 127.0.0.1:18110 (guide-site.sh)
# 2. the pictures (shots.py, desktop and phone widths), 3. the PDFs (build.py, headless Chromium)
# 4. the PDFs copied into walkthrough/ (pictures and HTML into walkthrough/source/out/, not kept
#    in git), 5. the lab copy removed.
# The phone app pictures come from the app emulator test (docs/rust-pages/app-matrix/matrix.sh);
# APP_SHOTS says where they are (default: that test's own out/ folder).
set -u
here=$(cd "$(dirname "$0")" && pwd)
LAB=${GARDEN_LAB:?set GARDEN_LAB=user@host}
sha=${1:?main sha}
APP_SHOTS=${APP_SHOTS:-$here/../../docs/rust-pages/app-matrix/out}
img=mcr.microsoft.com/playwright/python:v1.56.0-noble
ls "$APP_SHOTS"/07-*.png >/dev/null 2>&1 || { echo "FAIL no app pictures in $APP_SHOTS (run matrix.sh first)"; exit 1; }
ssh -o BatchMode=yes "$LAB" 'mkdir -p ~/gp-guide/app-shots ~/gp-guide/menu'
scp -q "$here"/guide-site.sh "$here"/shots.py "$here"/build.py "$here"/guide-en.md "$here"/guide-ko.md "$LAB:gp-guide/"
scp -q "$APP_SHOTS"/07-*.png "$LAB:gp-guide/app-shots/"
# The menu pictures the manual shows (@menu lines), from the guides' own picture folder.
menus=$(sed -n 's/^ *@menu \([^ ]*\) *$/\1/p' "$here"/guide-en.md "$here"/guide-ko.md | sort -u)
for m in $menus; do scp -q "$here/../../START-HERE/images/$m.png" "$LAB:gp-guide/menu/"; done
ssh -o BatchMode=yes "$LAB" "cd ~/gp-guide && sh guide-site.sh up $sha" || exit 1
mkdir -p "$here/out"
ssh -o BatchMode=yes "$LAB" "cd ~/gp-guide && docker run --rm -v \$PWD:/g alpine:3.22 rm -rf /g/shots /g/out && docker run --rm --network host -v \$PWD:/g -w /g \
    -e GUIDE_PW=\$(sed -n 's/^GUIDE_PW=//p' guide-login.env) $img sh -c '
      apt-get -qq update >/dev/null && apt-get -qq install -y fonts-noto-cjk >/dev/null 2>&1;
      pip install -q --break-system-packages playwright==1.56.0 markdown >/dev/null 2>&1;
      python3 shots.py http://127.0.0.1:18110 shots; python3 build.py shots app-shots out menu;
      chown -R \$(id -u):\$(id -g) shots out'" > "$here/out/last-run.log" 2>&1
grep -E "^(PASS|FAIL)|Error|Traceback" "$here/out/last-run.log"
scp -q "$LAB:gp-guide/out/*.pdf" "$here/../" && scp -q "$LAB:gp-guide/out/*.html" "$LAB:gp-guide/shots/*.png" "$here/out/" \
    && echo "PASS copied back (PDFs in walkthrough/)"
ssh -o BatchMode=yes "$LAB" "cd ~/gp-guide && sh guide-site.sh down" >/dev/null
