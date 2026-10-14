#!/bin/sh
# Check the repo for secrets and common security mistakes before a commit.
# Run from anywhere inside the repo:   sh tools/check-secrets.sh
#
# Two scanners, both run in Docker so nothing needs installing:
#   gitleaks  looks for passwords, tokens and keys, in the files AND in every
#             past commit (a secret deleted later is still in the history).
#   semgrep   reads the code for security bugs (SQL built from input,
#             unescaped output, ...) using its public rule sets.
# Findings are printed with secret values redacted. Exit code 0 = clean.
set -u
export MSYS_NO_PATHCONV=1   # Git Bash on Windows: keep Docker's /repo and /src paths as they are
cd "$(git rev-parse --show-toplevel)" || exit 2
fail=0

echo "== gitleaks: files and history"
docker run --rm -v "$PWD":/repo zricethezav/gitleaks:latest \
    git /repo --redact --no-banner || fail=1

echo "== semgrep: code"
docker run --rm -v "$PWD":/src -w /src semgrep/semgrep \
    semgrep scan --metrics=off --error --quiet \
    --config p/rust --config p/secrets --config p/javascript \
    --exclude site --exclude local --exclude backups --exclude '*.min.js' . || fail=1

echo "== semgrep: the browser scripts the site serves"
docker run --rm -v "$PWD":/src -w /src semgrep/semgrep \
    semgrep scan --metrics=off --error --quiet \
    --config p/javascript --config p/secrets site/js || fail=1

if [ "$fail" = 0 ]; then echo "CLEAN"; else echo "FOUND SOMETHING: fix it before committing"; fi
exit "$fail"
