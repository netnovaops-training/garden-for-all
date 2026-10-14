#!/bin/bash
# =============================================================================
#  07-start-extras.sh: start one of the project helpers with a single word.
#
#  Run:   bash lab/07-start-extras.sh kuma        Uptime Kuma    http://localhost:3001
#         bash lab/07-start-extras.sh ha          Home Assistant http://localhost:8123
#         bash lab/07-start-extras.sh esphome     ESPHome        http://localhost:6052
#         bash lab/07-start-extras.sh n8n         n8n            http://localhost:5678
#         bash lab/07-start-extras.sh stop NAME   stop one of the above
#         bash lab/07-start-extras.sh status      what is running
#
#  Each one is a ready-made file in docker/extras/. Nothing here opens a port
#  to the internet. The data of each one is kept in a folder next to this
#  project (for example ./n8n-data), so stopping it loses nothing.
# =============================================================================
set -eu
cd "$(dirname "$0")/.."

file_for() {
    case "$1" in
        kuma)    echo docker/extras/uptime-kuma.yml ;;
        ha)      echo docker/extras/home-assistant.yml ;;
        esphome) echo docker/extras/esphome.yml ;;
        n8n)     echo docker/extras/n8n.yml ;;
        *)       return 1 ;;
    esac
}

CMD="${1:-}"
case "$CMD" in
    kuma|ha|esphome|n8n)
        F=$(file_for "$CMD")
        # For Home Assistant start only the hub; the MQTT helper waits for Project 6.
        if [ "$CMD" = "ha" ]; then
            docker compose -f "$F" up -d homeassistant
        else
            docker compose -f "$F" up -d
        fi
        echo "Started $CMD. Give it a minute, then open its address (see the top of this script)."
        ;;
    stop)
        NAME="${2:-}"
        F=$(file_for "$NAME") || { echo "Usage: $0 stop kuma|ha|esphome|n8n"; exit 1; }
        docker compose -f "$F" down
        ;;
    status)
        docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
        ;;
    *)
        sed -n '3,12p' "$0" | sed 's/^#  \{0,2\}//'
        exit 1
        ;;
esac
