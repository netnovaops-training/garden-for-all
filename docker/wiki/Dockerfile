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
# The @sha256 digest pins the exact image (Dependabot's docker updates keep it current).
FROM python:3.15-alpine@sha256:0f9552bb28711333da16915ec146f5e6bb44a7fdac9ced15dc17c2e0206346dd

# Distro package versions are not pinned (hadolint DL3018): the base image is pinned by digest and
# kept current by Dependabot; pinned package versions break the build when the mirror drops them.
# hadolint ignore=DL3018
RUN apk add --no-cache mariadb-client

COPY wiki_setup.py /usr/local/bin/wiki_setup.py

# Run as a normal user, not root: this job needs no special powers.
# uid 1000 is the one adduser picked by itself before; now written down, and USER by number (hadolint DL3066).
RUN adduser -D -H -u 1000 wikisetup
USER 1000

ENTRYPOINT ["python3", "-u", "/usr/local/bin/wiki_setup.py"]
