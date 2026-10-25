#!/bin/sh
# Before every push of this branch: no secret and no private detail may leave.
#   sh docs/rust-pages/prepush.sh && git push origin rust-pages
# Scans the commits not yet on GitHub (origin/rust-pages..HEAD): gitleaks over their full
# patch, and a search of the added lines for home-network addresses, server paths, personal
# names and addresses, keys and tokens. Prints PASS, or FAIL with what was found (values of
# secrets are redacted by gitleaks; the search shows only the kind of match). Exit 1 = do not push.
set -u
cd "$(dirname "$0")/../.." || exit 1
range="origin/rust-pages..HEAD"
git rev-parse -q --verify origin/rust-pages >/dev/null || range="HEAD~1..HEAD"
fail=0
if ! git log -p "$range" | docker run --rm -i ghcr.io/gitleaks/gitleaks:latest stdin --redact --no-banner >/dev/null 2>&1; then
    echo "FAIL gitleaks found something (run: git log -p $range | gitleaks stdin --redact -v)"
    fail=1
fi
# The Android package name org.jasonagibson.junsgarden (and its folder path) is public and
# expected; everything else on this list must not appear in an added line.
# (This script itself is left out: its own search words would match.)
hits=$(git diff "$range" -- . ':!docs/rust-pages/prepush.sh' | grep -E '^\+' \
    | sed -e 's#org\.jasonagibson\.junsgarden##g' -e 's#org/jasonagibson/junsgarden##g' \
    | grep -oiE '192\.168\.[0-9]+\.[0-9]+|/home/server|server@[a-z0-9.]+|jasonagibson|1jasong|[a-z0-9._-]+@gmail\.com|nuc-homelab|kopoboi|BEGIN [A-Z ]*PRIVATE KEY|ghp_[A-Za-z0-9]{6}|github_pat_' \
    | sed -E 's/(ghp_|github_pat_).*/\1<token>/' | sort | uniq -c)
if [ -n "$hits" ]; then
    echo "FAIL private details in added lines:"
    echo "$hits"
    fail=1
fi
[ $fail = 0 ] && echo "PASS prepush ($range)"
exit $fail
