#!/bin/bash
# Test-only: tools/get-parts.py (several hosts, small encrypted parts, once per version, a pinned part) and
# tools/release/make_photo_parts.py, with fake hosts on localhost and a small made-up photos zip.
exec python3 "$(dirname "$0")/test_get_parts.py" "$@"
