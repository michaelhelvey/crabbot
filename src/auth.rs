//! Functions for parsing and validating Discord request signatures with a public key.
//! This code was lifted from the serenity project (with minor modifications), and is goverened by
//! the ISC license included with that software as follows:
//!
//! ISC License (ISC)
//!
//! Copyright (c) 2016, Serenity Contributors
//! Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the above copyright notice and this permission notice appear in all copies.
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use axum::{body::Body, extract::Request, middleware::Next, response::IntoResponse, Json};
use http::{header, HeaderMap, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use thiserror::Error;
use tracing::{debug, error, info};

use crate::response::{HttpResult, IntoHttp};

#[derive(Clone)]
pub struct Verifier {
    public_key: ed25519_dalek::VerifyingKey,
}

#[derive(Debug, Error)]
pub struct InvalidKey(#[from] ed25519_dalek::SignatureError);
impl std::fmt::Display for InvalidKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid bot public key: {}", self.0)
    }
}

/// Parses a hex string into an array of `[u8]`
fn parse_hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 {
        return None;
    }

    let mut res = [0; N];
    for (i, byte) in res.iter_mut().enumerate() {
        *byte = u8::from_str_radix(s.get(2 * i..2 * (i + 1))?, 16).ok()?;
    }
    Some(res)
}

impl Verifier {
    /// Creates a new [`Verifier`] from the given public key hex string.
    ///
    /// Panics if the given key is invalid. For a low-level, non-panicking variant, see
    /// [`Self::try_new()`].
    #[must_use]
    pub fn new(public_key: &str) -> Self {
        Self::try_new(parse_hex(public_key).expect("public key must be a 64 digit hex string"))
            .expect("invalid public key")
    }

    /// Creates a new [`Verifier`] from the public key bytes.
    ///
    /// # Errors
    ///
    /// [`InvalidKey`] if the key isn't cryptographically valid.
    pub fn try_new(public_key: [u8; 32]) -> Result<Self, InvalidKey> {
        Ok(Self {
            public_key: ed25519_dalek::VerifyingKey::from_bytes(&public_key).map_err(InvalidKey)?,
        })
    }

    /// Verifies a Discord request for authenticity, given the `X-Signature-Ed25519` HTTP header,
    /// `X-Signature-Timestamp` HTTP headers and request body.
    // We just need to differentiate "pass" and "failure". There's deliberately no data besides ().
    #[allow(clippy::result_unit_err, clippy::missing_errors_doc)]
    pub fn verify(&self, signature: &str, timestamp: &str, body: &[u8]) -> Result<(), ()> {
        use ed25519_dalek::Verifier as _;

        // Extract and parse signature
        let signature_bytes = parse_hex(signature).ok_or(())?;
        let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);

        // Verify
        let message_to_verify = [timestamp.as_bytes(), body].concat();
        self.public_key
            .verify(&message_to_verify, &signature)
            .map_err(|_| ())?;

        info!("successfully validated request");
        Ok(())
    }
}

macro_rules! reject_with_msg {
    ($msg:literal) => {
        return (
            StatusCode::UNAUTHORIZED,
            [(header::CONTENT_TYPE, mime::APPLICATION_JSON.to_string())],
            Json(json!({ "error": $msg })),
        )
            .into_response()
            .into_http();
    };
}
/// Middleware fn that verifies the
pub async fn verify_public_key_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> HttpResult {
    let verifier = Verifier::new(
        std::env::var("DISCORD_PUBLIC_KEY")
            .expect("DISCORD_PUBLIC_KEY is required")
            .as_str(),
    );

    let Some(timestamp) = headers
        .get("X-Signature-Timestamp")
        .and_then(|v| v.to_str().ok())
    else {
        error!("no timestamp on request");
        reject_with_msg!("X-Signature-Timestamp header is not present or is invalid");
    };

    let Some(signature) = headers
        .get("X-Signature-Ed25519")
        .and_then(|v| v.to_str().ok())
    else {
        error!("no signature on request");
        reject_with_msg!("X-Signature-Ed25519 header is not present or is invalid");
    };

    // Take the request apart to inspect the body and then re-construct it to call the next handler
    // in the middleware chain
    let (parts, body) = request.into_parts();
    let bytes = body.collect().await?.to_bytes();
    let debug_body = String::from_utf8_lossy(&bytes);
    debug!("validating body with public key: {debug_body:?}");

    if verifier.verify(signature, timestamp, &bytes).is_err() {
        error!("body failed signature verification");
        reject_with_msg!("Body failed signature verification");
    }

    let request = Request::from_parts(parts, Body::from(bytes));

    let response = next.run(request).await;
    Ok(response)
}
