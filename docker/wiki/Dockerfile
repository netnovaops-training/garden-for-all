# =============================================================================
#  The "wiki-setup" helper container.
#
#  It runs wiki_setup.py ONE time to set up your wiki and fill it with the
#  garden pages, then it is thrown away. You start it like this:
#      docker compose run --rm wiki-setup
#  ("--rm" means: delete the helper container when the job is done.)
#
#  Inside it:
#    python3        runs wiki_setup.py (only Python's own built-in parts)
#    mariadb-client lets the script read/write the wiki's own database
#                   (page dates and the "done" note)
#    /seed          the garden's wiki pages (wiki-pages.json.gz), from the wiki
#                   add-on download. Not built in: docker-compose.yml lends the
#                   folder docker/wiki/seed to the helper, read-only. Without the
#                   add-on the wiki is still set up, just empty.
# =============================================================================
FROM python:3.14-alpine

RUN apk add --no-cache mariadb-client

COPY wiki_setup.py /usr/local/bin/wiki_setup.py

# Run as a normal user, not root: this job needs no special powers.
RUN adduser -D -H wikisetup
USER wikisetup

ENTRYPOINT ["python3", "-u", "/usr/local/bin/wiki_setup.py"]
