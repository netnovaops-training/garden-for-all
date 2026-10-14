#!/bin/bash
# =============================================================================
#  Jun's Garden for Ubuntu: RIGHT-CLICK this file and choose "Run as a Program".
#
#  It opens a terminal window with the garden menu (garden.sh). Type 1 and
#  press Enter to install. Everything it does is explained on the screen
#  before it happens, and saved in the logs/ folder.
#
#  The first time, it gets the program itself from GitHub (tools/get-code.py):
#  the zip from Google Drive carries only what is not on GitHub (the starter
#  backup, the music file, the handbook). Every file is checked against
#  FILES-FROM-AUTHOR.txt before it is used.
#
#  No "Run as a Program" in the right-click menu? Then:
#    1. right-click an empty spot in this folder, choose "Open in Terminal"
#    2. type (or paste with Ctrl+Shift+V):   bash Start-Here-Ubuntu.sh
#    3. press Enter
# =============================================================================
HERE=$(cd "$(dirname "$(readlink -f "$0")")" && pwd)
# Started by a click (no terminal window)? Open one, so you can see everything.
if [ ! -t 0 ] && [ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] && [ -z "${GARDEN_IN_TERMINAL:-}" ]; then
    export GARDEN_IN_TERMINAL=1
    for t in gnome-terminal x-terminal-emulator xfce4-terminal konsole xterm; do
        command -v "$t" >/dev/null 2>&1 || continue
        case "$t" in
            gnome-terminal) exec gnome-terminal --title="Jun's Garden" -- bash "$HERE/Start-Here-Ubuntu.sh" "$@" ;;
            *)              exec "$t" -e bash "$HERE/Start-Here-Ubuntu.sh" "$@" ;;
        esac
    done
fi
if [ ! -f "$HERE/garden.sh" ]; then
    if ! python3 "$HERE/tools/get-code.py"; then
        echo; echo "   Press Enter to close. / Enter를 누르면 닫혀요."; read -r _ || true
        exit 1
    fi
fi
exec bash "$HERE/garden.sh" "$@"
