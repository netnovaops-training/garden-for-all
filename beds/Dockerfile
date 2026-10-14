# The "beds" tools service for Jun's Garden.
#
# Build:  docker build -t junsgarden/beds:mine .
# (or let docker compose build it; see README.md in this folder)
#
# python:3.12-slim is a small Debian with Python 3.12 already installed.
FROM python:3.14-slim

# The only libraries this service needs from outside Python itself:
#   pymysql  talks to the MariaDB database
#   ddgs     DuckDuckGo web search (no key needed) for plant facts
#   Pillow   opens, turns and shrinks tag photos before saving them
RUN pip install --no-cache-dir --root-user-action=ignore \
    pymysql==1.1.1 \
    ddgs \
    Pillow==11.0.0

# A normal (non-root) user to run as. uid/gid 1000 is the first user on
# most Ubuntu laptops, so photos it saves into /photos belong to that user.
# Compose can still pick another id with `user:` (see README.md).
RUN groupadd --gid 1000 garden \
    && useradd --uid 1000 --gid 1000 --no-create-home --shell /usr/sbin/nologin garden \
    && mkdir -p /photos && chown garden:garden /photos

# The code lives in /app/garden_ingest so Python can import it as the
# package "garden_ingest" (that is the name the code uses for itself).
COPY --chown=garden:garden . /app/garden_ingest/
WORKDIR /app
ENV PYTHONPATH=/app \
    PYTHONDONTWRITEBYTECODE=1 \
    PORT=8025

USER garden
EXPOSE 8025

# Docker asks /healthz every 30 seconds; "unhealthy" means the database
# cannot be reached (or the service stopped answering).
HEALTHCHECK --interval=30s --timeout=10s --start-period=20s --retries=3 \
    CMD python -c "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:8025/healthz', timeout=8).status == 200 else 1)"

CMD ["python", "-u", "/app/garden_ingest/service.py"]
