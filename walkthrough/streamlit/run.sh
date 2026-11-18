#!/bin/sh
# Make the two Add-plant guide PDFs (English, Korean) from THIS source folder, with sample data
# only, on any computer with Docker (Linux, or Windows with Git Bash):
#   sh walkthrough/streamlit/run.sh
# 1. lab.sh up: a throwaway garden (own network, own database, no published port)
# 2. shots.py: the pictures (a recorded sample run plays the desktop helper; no AI model runs)
# 3. build.py: the PDFs (headless Chromium, Korean fonts)
# 4. the PDFs into walkthrough/streamlit/; pictures and HTML stay in out/ (not kept in git)
# 5. lab.sh down
set -u
export MSYS_NO_PATHCONV=1
here=$(cd "$(dirname "$0")" && { pwd -W 2>/dev/null || pwd; })
img=mcr.microsoft.com/playwright/python:v1.56.0-noble
# `sh run.sh pdf`: only the PDFs again, from the pictures already in out/shots (after a text edit).
if [ "${1:-}" = pdf ]; then
    docker run --rm -v "$here:/g" -w /g $img sh -c '
        apt-get -qq update >/dev/null && apt-get -qq install -y fonts-noto-cjk >/dev/null 2>&1;
        pip install -q --break-system-packages playwright==1.56.0 markdown >/dev/null 2>&1;
        python3 build.py out/shots out/pdf' && cp "$here"/out/pdf/*.pdf "$here/" && echo "PASS PDFs in walkthrough/streamlit/"
    exit $?
fi
sh "$here/lab.sh" up || exit 1
docker run --rm -v "$here:/g" alpine:3.22 rm -rf /g/out/shots /g/out/pdf
docker run --rm --network pa-net --env-file "$here/out/lab/lab-login.env" -v "$here:/g" -w /g $img sh -c '
    apt-get -qq update >/dev/null && apt-get -qq install -y fonts-noto-cjk >/dev/null 2>&1;
    pip install -q --break-system-packages playwright==1.56.0 pillow markdown >/dev/null 2>&1;
    python3 shots.py http://pa-site:8030 out/shots && python3 build.py out/shots out/pdf' > "$here/out/last-run.log" 2>&1
grep -E "^(PASS|FAIL)|Error|Traceback" "$here/out/last-run.log"
cp "$here"/out/pdf/*.pdf "$here/" && echo "PASS PDFs in walkthrough/streamlit/"
sh "$here/lab.sh" down
