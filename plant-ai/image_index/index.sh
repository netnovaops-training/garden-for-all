#!/bin/sh
# Run the image index in Docker (nothing is installed on the computer itself).
#   GFA_DATA=/path/to/_local-data sh plant-ai/image_index/index.sh build|status|eval|search FILE|serve|stop
# GFA_DATA is the folder that holds the garden's own data (never in git):
#   in/current.json, in/future.json        the two table exports
#   in/current-photos/images/Database/     the garden's photos
#   images/<future plant id>/<n>.webp      the wish-list photos (more arrive over time: run build again)
# The index itself goes to $GFA_DATA/image-index/index (override with INDEX_DIR).
# CPUS (default 12) limits how much of the computer a build or the service may use.
# serve: published on 127.0.0.1 ONLY. For another container on the Docker network use
#   NETWORK=ai-stack_default sh index.sh serve     (no port published at all; name: plant-image-index)
set -eu
: "${GFA_DATA:?set GFA_DATA to the folder with in/ and images/ (the local garden data)}"
IMG="${IMAGE:-gfa-image-index}"
INDEX="${INDEX_DIR:-$GFA_DATA/image-index/index}"
CPUS="${CPUS:-12}"
mkdir -p "$INDEX"
w() { if command -v cygpath >/dev/null 2>&1; then cygpath -w "$1"; else echo "$1"; fi; }
export MSYS_NO_PATHCONV=1
cmd="${1:-status}"
shift || true
# Every run: nothing writable except /index, no network unless serving, no extra rights.
case "$cmd" in
  build)
    exec docker run --rm --cpus "$CPUS" --read-only --tmpfs /tmp --cap-drop ALL \
      --security-opt no-new-privileges --network none \
      -v "$(w "$GFA_DATA/in"):/data:ro" \
      -v "$(w "$GFA_DATA/in/current-photos/images/Database"):/photos/current:ro" \
      -v "$(w "$GFA_DATA/images"):/photos/future:ro" -v "$(w "$INDEX"):/index" \
      "$IMG" build --photos current=/photos/current --photos future=/photos/future \
      --db /data "$@" ;;
  status|eval)
    exec docker run --rm --read-only --tmpfs /tmp --cap-drop ALL --security-opt no-new-privileges \
      --network none -v "$(w "$INDEX"):/index" "$IMG" "$cmd" "$@" ;;
  search)
    f="$1"; shift
    exec docker run --rm --cpus "$CPUS" --read-only --tmpfs /tmp --cap-drop ALL \
      --security-opt no-new-privileges --network none -v "$(w "$INDEX"):/index" \
      -v "$(w "$(dirname "$f")"):/q:ro" "$IMG" search "/q/$(basename "$f")" "$@" ;;
  serve)
    docker rm -f plant-image-index >/dev/null 2>&1 || true
    if [ -n "${NETWORK:-}" ]; then
      exec docker run -d --name plant-image-index --restart unless-stopped --cpus "$CPUS" \
        --read-only --tmpfs /tmp --cap-drop ALL --security-opt no-new-privileges \
        --network "$NETWORK" -e IMAGE_INDEX_ALLOWED_HOSTS=plant-image-index \
        -e IMAGE_INDEX_IN_DOCKER=1 -v "$(w "$INDEX"):/index" "$IMG" \
        serve --bind 0.0.0.0 --port 8099
    fi
    exec docker run -d --name plant-image-index --restart unless-stopped --cpus "$CPUS" \
      --read-only --tmpfs /tmp --cap-drop ALL --security-opt no-new-privileges \
      -p "127.0.0.1:${PORT:-8099}:8099" -e IMAGE_INDEX_IN_DOCKER=1 \
      -v "$(w "$INDEX"):/index" "$IMG" serve --bind 0.0.0.0 --port 8099 ;;
  stop) exec docker rm -f plant-image-index ;;
  *) echo "usage: index.sh build|status|eval|search FILE|serve|stop" >&2; exit 2 ;;
esac
