//! Shared plumbing for every Rust app in the stack.
//!
//! Deliberately small. This crate holds the four things all four apps need and
//! nothing else: reading config from the environment, starting a server that
//! shuts down cleanly, a `/healthz` that means something, and a client for the
//! LiteLLM router.
//!
//! The router client is the important one. It is the Rust twin of
//! `shared/router.py`: it asks for a **job** -- `chat`, `fast`, `phone` -- never
//! a model name. That's the one idea the whole stack is built on, and it has to
//! survive the port to Rust or the port isn't finished.
// `use` brings names into scope. `anyhow` gives one error type that can hold any error;
// `Result` here is `anyhow`'s `Result`, so functions below can return errors with `?`.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::Router;


/// Read an environment variable, falling back to a default.
///
/// Blank counts as unset. `FOO=` in a `.env` file is how people turn a setting
/// off, and treating that as the empty string rather than "unset" produces
/// baffling failures much later.
// `&str` is a borrowed piece of text. `String` is text the caller can keep. `match` picks
// the arm that fits the value. `Ok(v) if ...` means: a value that also passes the check.
pub fn env_or(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => default.to_string(),
    }
}

/// Read a required environment variable, or fail with a message that names it.
pub fn env_required(key: &str) -> Result<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => anyhow::bail!("{key} is not set -- add it to the .env file the container is started with"),
    }
}

/// Reads a number setting such as a port. A missing or unreadable value gives `default`.
/// `u16` is a whole number from 0 to 65535, the range of network ports.
pub fn env_u16(key: &str, default: u16) -> u16 {
    env_or(key, "").parse().unwrap_or(default)
}

/// Reads a yes/no setting. 1, true, yes and on mean true; 0, false, no and off mean
/// false; anything else (or unset) gives `default`.
pub fn env_bool(key: &str, default: bool) -> bool {
    match env_or(key, "").to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => default,
    }
}

/// Set up logging. `RUST_LOG` overrides; otherwise info for us, warn for deps.
pub fn init_tracing(app: &str) {
    // `use` inside a function makes the names available only in that function.
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    // Log levels per part of the program. `RUST_LOG`, if set, replaces this.
    let default = format!("{app}=info,common=info,tower_http=warn,axum=warn,info");
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    // Build the logger from the filter and a printer, then switch it on for the whole program.
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}

/// Bind and serve until Docker says stop.
///
/// The graceful-shutdown handler is not decoration. Without it `docker compose
/// restart` sends SIGTERM, gets no response, waits the full ten seconds, then
/// SIGKILLs -- so every restart of every app costs ten seconds and any
/// in-flight request is cut off mid-write. With it, restarts are instant.
pub async fn serve(app: Router, port: u16) -> Result<()> {
    // 0.0.0.0 means "listen on every network address of this machine", not only locally.
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    // Claim the port. `.await` waits for it. `.with_context` adds a clear message on failure.
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("could not bind port {port} -- is something else using it?"))?;

    tracing::info!("listening on http://{addr}");

    // Hand requests to the app until the shutdown signal arrives, then finish current ones.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    tracing::info!("stopped cleanly");
    Ok(())
}

/// Completes when the program is asked to stop: Ctrl-C, or SIGTERM (the signal Docker
/// sends). Used by `serve` so it can shut down cleanly.
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    // One task waits for Ctrl-C.
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    // Another waits for SIGTERM. If it cannot be set up, it waits forever (`pending`).
    let term = async {
        match signal(SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    // `select!` finishes as soon as either of the two waits finishes.
    tokio::select! {
        _ = ctrl_c => {}
        _ = term => {}
    }
    tracing::info!("shutdown signal received");
}

// --- local time --------------------------------------------------------------
//
// This looks like more ceremony than "what year is it?" deserves. It isn't.
//
// `OffsetDateTime::now_local()` REFUSES to work once a process has more than
// one thread, on every Unix, by design -- reading the TZ environment while
// another thread might be writing it is a real soundness hole and the crate
// closed it by failing instead. Tokio's runtime spawns threads. So any call
// made from inside an async handler silently falls back to UTC, and your dates
// are seven hours out in Nevada with nothing in the logs to say why.
//
// The fix is to read the offset ONCE, before the runtime exists, and reuse it.
// That's what `init_local_offset` is for, and why every app's `main` is written
// out longhand instead of using `#[tokio::main]`.

use std::sync::OnceLock;
use time::{OffsetDateTime, UtcOffset};

/// Holds the machine's UTC offset once it has been read. `static` lives for the whole
/// program. `OnceLock` can be filled exactly one time, and is safe to read from many threads.
static LOCAL_OFFSET: OnceLock<UtcOffset> = OnceLock::new();

/// Read the machine's UTC offset. **Call this as the first line of `main`,**
/// before any runtime is built and while the process is still single-threaded.
pub fn init_local_offset() {
    // If the offset cannot be read, fall back to UTC.
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    // `set` stores it once. Setting a second time would fail, and that is ignored.
    let _ = LOCAL_OFFSET.set(offset);
}

/// Now, in the machine's local timezone -- which is `TZ` from the compose file.
pub fn now_local() -> OffsetDateTime {
    // Take the current UTC time and shift it by the stored offset.
    let offset = LOCAL_OFFSET.get().copied().unwrap_or(UtcOffset::UTC);
    OffsetDateTime::now_utc().to_offset(offset)
}

/// The current year, for footers and copyright lines.
pub fn current_year() -> i32 {
    now_local().year()
}

/// A multi-threaded Tokio runtime.
///
/// Built by hand rather than with `#[tokio::main]` so that `init_local_offset`
/// can run before any thread exists. See the note above.
pub fn runtime() -> Result<tokio::runtime::Runtime> {
    // Build a runtime with several worker threads, and turn on its I/O and timer support.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("could not start the async runtime")
}
