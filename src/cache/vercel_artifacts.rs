use opendal::Operator;
use opendal::layers::{HttpClientLayer, LoggingLayer};
use opendal::services::VercelArtifacts;

use crate::errors::*;

use super::http_client::set_user_agent;

/// Sanitize a cache key so it matches the Vercel Artifacts API's hash regex
/// (`/^[a-fA-F0-9]+$/`).  Only hex characters [0-9a-f] are passed through;
/// every other byte is replaced with its two-character uppercase hex encoding
/// (e.g. `/` → `2F`, `.` → `2E`, `k` → `6B`).
///
/// This keeps already-valid lowercase hex hash keys (the common case) untouched
/// while safely encoding the `/` separators from `normalize_key` and any other
/// non-hex characters.
pub fn sanitize_key(key: &str) -> String {
    let mut out = String::with_capacity(key.len());
    for b in key.bytes() {
        if b.is_ascii_hexdigit() {
            out.push(b as char);
        } else {
            out.push(char::from_digit((b >> 4) as u32, 16).unwrap().to_ascii_uppercase());
            out.push(char::from_digit((b & 0xf) as u32, 16).unwrap().to_ascii_uppercase());
        }
    }
    out
}

/// A cache that stores entries in Vercel Artifacts.
pub struct VercelArtifactsCache;

impl VercelArtifactsCache {
    pub fn build(
        access_token: &str,
        endpoint: Option<&str>,
        team_id: Option<&str>,
        team_slug: Option<&str>,
    ) -> Result<Operator> {
        let mut builder = VercelArtifacts::default().access_token(access_token);
        if let Some(endpoint) = endpoint {
            builder = builder.endpoint(endpoint);
        }
        if let Some(team_id) = team_id {
            builder = builder.team_id(team_id);
        }
        if let Some(team_slug) = team_slug {
            builder = builder.team_slug(team_slug);
        }

        let op = Operator::new(builder)?
            .layer(HttpClientLayer::new(set_user_agent()))
            .layer(LoggingLayer::default())
            .finish();
        Ok(op)
    }
}
