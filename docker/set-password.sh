#!/bin/sh
# =============================================================================
#  set-password.sh: set a login password WITHOUT needing email.
#  Kept so older guides still work; it hands over to garden.sh.
#
#  Run from the project folder:   sh docker/set-password.sh Jun
#  (the same as:                  bash garden.sh password Jun)
#
#  How it works: the site stores only an Argon2 "hash" of a password (a
#  one-way scramble). The toolbox container makes that hash from the password
#  you type and saves it in the garden_users table. The password itself is
#  never shown or saved. The name must match one in GARDEN_USERS in your .env.
# =============================================================================
cd "$(dirname "$0")/.." || exit 1
exec bash garden.sh password "$@"
