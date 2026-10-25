# =============================================================================
#  One Dockerfile for every Rust app in the workspace.
#
#      docker compose build landing        ->  ARG APP=landing
#      docker compose build teabook-rs     ->  ARG APP=booking     (later)
#
#  This is the "you never have to install Rust on Windows" half of the setup.
#  The whole toolchain -- compiler, cargo, the crate registry, several hundred
#  megabytes of build artefacts -- lives in the build stage and is thrown away.
#  What lands on your disk is one binary in a small image.
#
#  Debian slim rather than Alpine, deliberately. Alpine would give a ~15MB image
#  instead of ~90MB, but musl builds of the TLS and crypto crates are the single
#  most common way a Rust Docker build fails, and the errors it produces name
#  neither the crate nor the cause. On a desktop that already holds 40GB of
#  model weights, 75MB is not worth an evening. `docker exec -it landing bash`
#  also works here, which matters the first time something misbehaves.
# =============================================================================

# Pinned, not `1`, so a Rust release can never silently change your build.
# Raise it deliberately when you want to.
ARG RUST_VERSION=1.95

# --- build -------------------------------------------------------------------
# The @sha256 digest pins the exact image, so a changed or hijacked tag can't slip into the
# build. It belongs to rust 1.95: when you raise RUST_VERSION, update the digest too
# (Dependabot's docker updates do it for you).
FROM rust:${RUST_VERSION}-slim-bookworm@sha256:d7482085ff5b415f84dba5647ae71606650bdef00db7aeb69f4b3d170c3e4082 AS builder

# pkg-config and libssl-dev aren't used by today's dependency tree (it's rustls
# all the way down) but nearly every crate you might add next wants them, and
# their absence produces a linker error that names neither.
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# The whole workspace. Cargo needs every member's manifest present before it
# can resolve anything, so the usual "copy manifests first" caching trick buys
# much less here than the BuildKit cache mounts below do.
COPY . .

ARG APP
RUN test -n "$APP" || (echo "build with --build-arg APP=<crate name>" >&2 && exit 1)

# BuildKit cache mounts: the downloaded registry and the compiled dependencies
# survive between builds. First build takes a few minutes; every one after it
# takes seconds, because only your own code recompiles.
#
# The binary has to be copied OUT of the cache mount before the RUN ends --
# once the mount is released, /build/target is empty again. That's the `cp`.
#
# --locked means the build uses Cargo.lock exactly. A dependency cannot quietly
# update itself between your machine and this image.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    cargo build --release --locked --bin "$APP" \
    && cp "/build/target/release/$APP" /build/app

# --- run ---------------------------------------------------------------------
FROM debian:bookworm-slim@sha256:7ff6ba31527b9e336162fa061746c366c4b2032f4d2cff2078bea43f78efe3f8

# ca-certificates: any outbound HTTPS.
# tzdata:  without it every date in the app is UTC, silently. The apps read TZ
#          from the compose file, and that only works if the database is here.
# wget:    what the healthcheck uses.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates tzdata wget \
    && rm -rf /var/lib/apt/lists/*

# Never run something that faces the internet as root, even in a container,
# even at home.
RUN useradd --system --create-home --shell /usr/sbin/nologin app

COPY --from=builder /build/app /usr/local/bin/app

# Apps that keep state write here; the compose file mounts a volume over it.
RUN mkdir -p /data && chown app:app /data
USER app

ENV APP_PORT=8010
EXPOSE 8010

# Docker Desktop shows this as a green/red dot next to the container, which is
# the fastest way to tell "started" from "actually working".
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- "http://127.0.0.1:${APP_PORT}/healthz" >/dev/null 2>&1 || exit 1

ENTRYPOINT ["/usr/local/bin/app"]
