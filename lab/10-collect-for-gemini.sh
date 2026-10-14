#!/bin/bash
# =============================================================================
#  10-collect-for-gemini.sh: gather the facts Gemini needs to help you, with
#  passwords and keys removed.
#
#  Run:   bash lab/10-collect-for-gemini.sh
#  Then:  open the file it names, read it once, and paste it into Gemini
#         together with the "I am stuck" prompt (START-HERE/05-gemini-prompts.md).
#
#  What it collects (it only READS):
#    - Ubuntu version, memory, disk space
#    - which containers are running
#    - the last 40 lines of the garden's logs
#    - the output of the health check
#
#  What it does NOT collect: the .env file, any password, any key, any token.
#  It also blanks any line that looks like it holds one. YOU STILL READ THE
#  FILE BEFORE PASTING, because no filter is perfect.
# =============================================================================
cd "$(dirname "$0")/.." || exit 1

OUT="gemini-help-$(date +%F-%H%M).txt"

# Hide any line that mentions a secret-looking word, and anything shaped like a
# long key or an email password. Sends the clean text to the output file.
clean() {
    sed -E \
        -e '/(PASSWORD|PASSWD|SECRET|TOKEN|API_?KEY|APIKEY|AUTHORIZATION|BEARER|PRIVATE KEY|DDNS)/Id' \
        -e 's/[A-Za-z0-9_\/+=-]{40,}/[LONG-VALUE-HIDDEN]/g' \
        -e 's/mysql:\/\/[^ ]*/mysql:\/\/[HIDDEN]/g'
}

{
    echo "# Facts for Gemini ($(date))"
    echo; echo "## Ubuntu"; lsb_release -ds 2>/dev/null; uname -r
    echo; echo "## Memory and disk"; free -h; df -h / | tail -2
    echo; echo "## Docker"; docker --version; docker compose version
    echo; echo "## Containers"; docker ps -a --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
    echo; echo "## Garden website log (last 40 lines)"; docker compose logs --tail 40 site 2>&1
    echo; echo "## Garden database log (last 15 lines)"; docker compose logs --tail 15 db 2>&1
    echo; echo "## Health check"; bash lab/09-health-check.sh 2>&1
} 2>&1 | clean > "$OUT"

chmod 600 "$OUT"
echo "Wrote $OUT"
echo "READ IT FIRST. If you see anything private, delete that line, then paste it into Gemini."
