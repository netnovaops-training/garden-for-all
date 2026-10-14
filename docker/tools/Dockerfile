# =============================================================================
#  The garden "toolbox" container: backup, restore and set-password.
#
#  Why a container? So the SAME backup and restore code runs on Ubuntu and on
#  Windows. Docker hands it your folders (images, data, cashe, backups) using
#  the paths in your .env file, so it never needs to know where they really are.
#
#  You never start this by hand. garden.sh (Ubuntu) and Garden-Menu.ps1
#  (Windows) run it for you, like this:
#      docker compose run --rm tools backup
#  "--rm" means the container is thrown away as soon as the job is done.
# =============================================================================
FROM alpine:3.24

# mariadb-client : talks to the database (dump = save, mariadb = load)
# argon2         : makes password hashes for set-password
# tar, gzip, coreutils : packing files and checking them (sha256sum)
# curl           : sends the test email (garden.sh email)
RUN apk add --no-cache bash mariadb-client argon2 tar gzip coreutils findutils curl

COPY garden-tools.sh /usr/local/bin/garden-tools
RUN sed -i 's/\r$//' /usr/local/bin/garden-tools && chmod 755 /usr/local/bin/garden-tools

ENTRYPOINT ["garden-tools"]
