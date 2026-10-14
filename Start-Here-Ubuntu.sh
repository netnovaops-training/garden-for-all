#!/bin/bash
# =============================================================================
#  Jun's Garden for Ubuntu: RIGHT-CLICK this file and choose "Run as a Program".
#
#  It opens a terminal window with the garden menu (garden.sh). Type 1 and
#  press Enter to install. Everything it does is explained on the screen
#  before it happens, and saved in the logs/ folder.
#
#  No "Run as a Program" in the right-click menu? Then:
#    1. right-click an empty spot in this folder, choose "Open in Terminal"
#    2. type (or paste with Ctrl+Shift+V):   bash garden.sh
#    3. press Enter
# =============================================================================
HERE=$(cd "$(dirname "$(readlink -f "$0")")" && pwd)
exec bash "$HERE/garden.sh" "$@"
