//! `/app/garden/{action}` — the app's camera tools (Identify, Tag, Barcode
//! lookup, Receipt, the bulk tag queue), passed on to garden-ingest.
//!
//! HOW TO READ THIS FILE (for Jun). The phone sends a photograph here with
//! its login key. This checks the key (only somebody logged in, never a
//! guest), checks the action is one of six allowed ones, and forwards the
//! request to the garden-ingest service on the NUC with the person's email
//! address attached. garden-ingest does the reading (free cloud AI only,
//! see its freeai.py) and answers; the answer goes back to the phone as is.
//!
//! WHY THE PHONE CANNOT CALL garden-ingest DIRECTLY. garden-ingest decides
//! who you are from an `X-Auth-Email` header and trusts it completely. Only
//! this route may set it, after checking the login. Until 2026-10-07 this
//! job was done by the author's household sign-in service; the author asked for the
//! garden's own login instead.
//!
//! WHY A FIXED LIST OF ACTIONS. garden-ingest also has routes that delete
//! beds and approve proposals. Forwarding whatever path the phone named,
//! with a real identity attached, would expose all of them.

// `Arc` is a shared, reference-counted pointer: many requests can hold the
// same AppState at once without copying it.
use std::sync::Arc;

// A `use` line brings a name into scope so it can be written short.
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use serde_json::json;

use crate::session::Visitor;
use crate::AppState;

/// Reversible, low-risk actions only: identify writes nothing, tag and
/// tag-queue create proposals waiting for approval, receipt records a
/// purchase, diagnose records a check-up, lookup only reads.
/// `[&str; 6]` is an array of exactly six text slices (`&str` is borrowed text,
/// unlike `String`, which owns its text). `const` means fixed at compile time.
const ACTIONS: [&str; 6] = [
    "identify",
    "diagnose",
    "tag",
    "receipt",
    "lookup",
    "tag-queue",
];

/// Build a JSON reply with the given HTTP status code.
/// Takes the status (for example 401) and a `serde_json::Value` (a JSON tree).
/// Gives back an axum `Response`. Every early exit in this file uses it so
/// that all errors look the same to the phone app: `{"error": "..."}`.
fn reply(status: StatusCode, body: serde_json::Value) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.to_string(),
    )
        .into_response()
}

/// The route handler for `/app/garden/{action}`. The app sends a photo (the
/// request body, as raw bytes) and this forwards it to garden-ingest.
/// Takes: `State` (shared app settings and database), `Extension<Visitor>` (who
/// is asking, set earlier by session.rs), `Path` (the `{action}` part of the URL)
/// and `body` (the bytes the phone sent).
/// Gives back: garden-ingest's own answer, or a JSON error.
/// `async fn` means the function can wait (for the network) without blocking
/// other visitors; `.await` is where it waits.
pub async fn garden_camera(
    State(state): State<Arc<AppState>>,
    Extension(visitor): Extension<Visitor>,
    Path(action): Path<String>,
    body: Bytes,
) -> Response {
    // 1. Only a signed-in garden account may use this. Guests get 401.
    if !visitor.is_admin() {
        return reply(
            StatusCode::UNAUTHORIZED,
            json!({ "error": "Log in to use the camera tools.", "needs_sign_in": true }),
        );
    }
    // 2. Only the six allowed actions pass (see ACTIONS above). Anything else is 404.
    // `.contains(&action.as_str())` asks the array whether it holds that text.
    if !ACTIONS.contains(&action.as_str()) {
        return reply(
            StatusCode::NOT_FOUND,
            json!({ "error": "No such garden action." }),
        );
    }
    // 3. Find this person's email address. `let ... else` runs the else block
    // (return an error) when the value is `None`, which is Rust's "maybe nothing".
    // garden-ingest needs the email because it decides who you are from it.
    let Some(email) = crate::accounts::find(&state, &visitor.username).map(|a| a.email.clone())
    else {
        return reply(
            StatusCode::FORBIDDEN,
            json!({ "error": "Not a garden account." }),
        );
    };

    // 4. Build the address of the matching garden-ingest route. `trim_end_matches`
    // removes a trailing slash so we never produce `//api/...`.
    let url = format!("{}/api/{}", state.ingest_url.trim_end_matches('/'), action);
    // 5. Forward the request. The `X-Auth-Email` header is the identity badge.
    // A photograph read by a free cloud model, then looked up: a minute is
    // normal and five is possible when a free tier is slow. Shorter makes a
    // working camera look broken.
    let sent = state
        .http
        .post(&url)
        .header("X-Auth-Email", email)
        .header(header::CONTENT_TYPE, "application/json")
        .timeout(std::time::Duration::from_secs(300))
        .body(body)
        .send()
        .await;
    // 6. Pass the answer back. `match` looks at which shape the result has:
    // `Ok` (the call worked) or `Err` (it failed). Inside `Ok`, we copy the
    // status code and the body bytes straight through to the phone.
    match sent {
        Ok(r) => {
            // Convert the status number between the two HTTP libraries' types. If that
            // fails, say 502 (bad gateway: the thing behind us misbehaved).
            let status =
                StatusCode::from_u16(r.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            match r.bytes().await {
                Ok(b) => (status, [(header::CONTENT_TYPE, "application/json")], b).into_response(),
                Err(_) => reply(
                    StatusCode::BAD_GATEWAY,
                    json!({ "error": "The garden sent no answer." }),
                ),
            }
        }
        // The call itself failed (garden-ingest is down or timed out). Log the real
        // error for the author, and give the phone a plain message without details.
        Err(e) => {
            tracing::error!("/app/garden/{action}: {e}");
            reply(
                StatusCode::BAD_GATEWAY,
                json!({ "error": "The garden's reader did not answer. Try again in a minute." }),
            )
        }
    }
}
