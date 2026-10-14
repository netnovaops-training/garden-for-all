#!/bin/bash
# Test-only: every file the website's pages link to by address (href="/....pdf",
# .html, .apk ...) must be in site/, or visitors get "not found". Run from the
# garden folder:   bash tools/release/tests/check-site-links.sh
# (Found 2026-10-14: the footer still linked a guide PDF that was removed from site/.)
cd "$(dirname "$0")/../../.." || exit 1
bad=0 n=0
while IFS= read -r link; do
    n=$((n + 1))
    if [ -e "site/$link" ]; then echo "OK       /$link"
    else echo "MISSING  /$link   (linked from the templates, not in site/)"; bad=$((bad + 1)); fi
done < <(grep -rhoE 'href="/[A-Za-z0-9._/-]+\.(pdf|html|png|jpg|webp|apk|css|js|json|mp3)"' rust/junsgarden/templates |
         sed 's/^href="\///; s/"$//' | sort -u)
echo "RESULT: $n file link(s), $bad missing"
[ "$bad" = 0 ]
