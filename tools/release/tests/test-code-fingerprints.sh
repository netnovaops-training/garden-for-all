#!/bin/sh
# Test-only: CODE-FINGERPRINTS.txt (the public list the first start checks the program against).
#   sh tools/release/tests/test-code-fingerprints.sh
# 1. the list in the staged files is exactly right (a stale list, or a tracked file missing from it, fails here);
# 2. the check really catches a changed file, a new file, and a removed file (tried on a copy of the index);
# 3. the pre-commit hook makes the list by itself (tried in a throwaway repository).
# Nothing in the working tree is changed.
cd "$(dirname "$0")/../../.." || exit 1
ROOT=$(pwd)
PY=""
for c in python3 python py; do
    if "$c" -c "import sys; sys.exit(0 if sys.version_info[0] >= 3 else 1)" >/dev/null 2>&1; then PY=$c; break; fi
done
[ -n "$PY" ] || { echo "FAIL  no Python 3"; exit 1; }
export PYTHONIOENCODING=utf-8
fail=0
ok() { echo "PASS  $1"; }
bad() { echo "FAIL  $1"; fail=1; }

if $PY tools/release/make_fingerprints.py --check; then ok "the list in the staged files is up to date"; else bad "the list in the staged files is stale (run: python tools/release/make_fingerprints.py, then git add CODE-FINGERPRINTS.txt)"; fi

T=$(mktemp -d "${TMPDIR:-/tmp}/fingerprints-test.XXXXXX") || exit 1
trap 'rm -rf "$T"' EXIT
TREE=$(git write-tree)
real_index=$(git rev-parse --git-path index)
try() {   # try NAME EXPECTED-TEXT  (the temp index was changed by the caller)
    out=$(GIT_INDEX_FILE="$T/idx" $PY tools/release/make_fingerprints.py --check 2>&1); rc=$?
    if [ "$rc" -ne 0 ] && echo "$out" | grep -q "$2"; then ok "$1"; else bad "$1 (exit $rc: $out)"; fi
}
fresh() { rm -f "$T/idx"; GIT_INDEX_FILE="$T/idx" git read-tree "$TREE"; }

fresh
blob=$(printf 'changed\n' | git hash-object -w --stdin)
GIT_INDEX_FILE="$T/idx" git update-index --cacheinfo "100644,$blob,garden.sh"
try "a changed file makes the list stale" "stale: garden.sh"

fresh
GIT_INDEX_FILE="$T/idx" git update-index --add --cacheinfo "100644,$blob,zz-new-file.txt"
try "a new tracked file missing from the list is caught" "missing from the list: zz-new-file.txt"

fresh
GIT_INDEX_FILE="$T/idx" git update-index --force-remove tools/get-pdfs.py
try "a file removed but still on the list is caught" "in the list but not a file: tools/get-pdfs.py"

fresh
GIT_INDEX_FILE="$T/idx" git update-index --force-remove CODE-FINGERPRINTS.txt
try "no list at all is caught" "is not in the tree"

fresh
GIT_INDEX_FILE="$T/idx" git update-index --add --cacheinfo "100644,$blob,README.md"
out=$(GIT_INDEX_FILE="$T/idx" $PY tools/release/make_fingerprints.py --check 2>&1) && ok "README.md (kept off GitHub) is not on the list, and may change freely" || bad "README.md should not matter: $out"

# 3. the hook, in a throwaway repository
R="$T/repo"
mkdir -p "$R/tools/release/hooks" "$R/tools/release/tests"
cp tools/release/make_fingerprints.py tools/release/install-hooks.sh "$R/tools/release/"
cp tools/release/hooks/pre-commit "$R/tools/release/hooks/"
(
    cd "$R" || exit 1
    git init -q . && git config user.email t@example.invalid && git config user.name test && git config core.autocrlf false
    printf '* text=auto eol=lf\n' > .gitattributes
    sh tools/release/install-hooks.sh >/dev/null || exit 1
    printf 'one\n' > "a file (1).txt"
    printf '@echo off\r\n' > start.cmd
    git add -A && git commit -q -m first || exit 1
    git show HEAD:CODE-FINGERPRINTS.txt > /dev/null || exit 2
    $PY tools/release/make_fingerprints.py --check --tree HEAD >/dev/null || exit 3
    printf 'two\n' > b.txt
    git add b.txt && git commit -q -m second || exit 1
    $PY tools/release/make_fingerprints.py --check --tree HEAD >/dev/null || exit 4
    git show HEAD:CODE-FINGERPRINTS.txt | grep -q '  b.txt$' || exit 5
    git show HEAD:CODE-FINGERPRINTS.txt | grep -q '  a file (1).txt$' || exit 6
    exit 0
)
rc=$?
if [ "$rc" -eq 0 ]; then ok "the pre-commit hook adds the list to the commit and keeps it right (a file with spaces too)"; else bad "the pre-commit hook test failed at step $rc"; fi

[ "$fail" -eq 0 ] && echo "ALL PASS" || echo "SOME FAILED"
exit $fail
