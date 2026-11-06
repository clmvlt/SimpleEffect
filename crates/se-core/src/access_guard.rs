//! Access control applied to every HTTP route: loopback `Host` header and session token.
//!
//! The core listens on 127.0.0.1 only, but any local process or web page could still try to reach
//! it. The `Host` check defeats DNS-rebinding attacks from web pages; the 256-bit token (known only
//! to the Electron app through the handshake) authenticates the client (`docs/architecture.md` §13).

use std::{fmt, sync::Arc};

use axum::{
    extract::{Request, State},
    http::{StatusCode, header::HOST, uri::Authority},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Query parameter carrying the session token: `/rpc?t=<hex>`.
const TOKEN_PREFIX: &str = "t=";

/// Number of random bytes in a session token (256 bits).
const TOKEN_BYTES: usize = 32;

/// The secret every client must present. Never logged: `Debug` is redacted.
pub struct SessionToken(String);

impl SessionToken {
    /// Generates a token from 256 bits of the thread-local CSPRNG, encoded as lowercase hex.
    pub fn generate() -> Self {
        Self(hex::encode(rand::random::<[u8; TOKEN_BYTES]>()))
    }

    /// The token as sent to clients in the handshake.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Compares in constant time so response timing does not reveal how many characters match.
    fn matches(&self, candidate: &str) -> bool {
        constant_time_eq(self.0.as_bytes(), candidate.as_bytes())
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionToken(<redacted>)")
    }
}

/// Decides whether a request may reach a route.
#[derive(Debug)]
pub struct AccessGuard {
    token: SessionToken,
    port: u16,
}

/// Why a request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rejection {
    ForeignHost,
    InvalidToken,
}

impl IntoResponse for Rejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::ForeignHost => (
                StatusCode::FORBIDDEN,
                "Forbidden: the Host header must be 127.0.0.1:<port> or localhost:<port>.",
            ),
            Self::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized: pass the session token from the se-core handshake as ?t=<token>.",
            ),
        };
        (status, message).into_response()
    }
}

impl AccessGuard {
    /// Creates a guard for a core listening on `port` and protected by `token`.
    pub fn new(token: SessionToken, port: u16) -> Self {
        Self { token, port }
    }

    /// The session token clients must present.
    pub fn token(&self) -> &SessionToken {
        &self.token
    }

    fn check(&self, host: Option<&str>, query: Option<&str>) -> Result<(), Rejection> {
        if !host.is_some_and(|host| is_loopback_host(host, self.port)) {
            return Err(Rejection::ForeignHost);
        }
        let token = query.and_then(token_from_query);
        if !token.is_some_and(|token| self.token.matches(token)) {
            return Err(Rejection::InvalidToken);
        }
        Ok(())
    }
}

/// Axum middleware rejecting requests with a foreign `Host` (403) or a missing/wrong token (401).
pub async fn require_local_access(
    State(guard): State<Arc<AccessGuard>>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .or_else(|| request.uri().authority().map(Authority::as_str));
    match guard.check(host, request.uri().query()) {
        Ok(()) => next.run(request).await,
        Err(rejection) => {
            tracing::warn!(?rejection, path = request.uri().path(), "request rejected");
            rejection.into_response()
        }
    }
}

/// Accepts exactly `127.0.0.1:<port>` or `localhost:<port>`.
fn is_loopback_host(host: &str, port: u16) -> bool {
    let Some((name, host_port)) = host.rsplit_once(':') else {
        return false;
    };
    let is_loopback_name = name == "127.0.0.1" || name.eq_ignore_ascii_case("localhost");
    is_loopback_name && host_port == port.to_string()
}

fn token_from_query(query: &str) -> Option<&str> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix(TOKEN_PREFIX))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    // The token length is public (fixed), so returning early on a length mismatch leaks nothing.
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> AccessGuard {
        AccessGuard::new(SessionToken("cafe".repeat(16)), 4242)
    }

    #[test]
    fn generated_tokens_are_64_lowercase_hex_characters_and_unique() {
        let first = SessionToken::generate();
        let second = SessionToken::generate();
        assert_eq!(first.as_str().len(), 64);
        assert!(
            first
                .as_str()
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert_ne!(first.as_str(), second.as_str());
    }

    #[test]
    fn token_debug_output_is_redacted() {
        assert_eq!(
            format!("{:?}", SessionToken("secret".to_owned())),
            "SessionToken(<redacted>)"
        );
    }

    #[test]
    fn accepts_loopback_hosts_with_the_core_port_only() {
        assert!(is_loopback_host("127.0.0.1:4242", 4242));
        assert!(is_loopback_host("localhost:4242", 4242));
        assert!(!is_loopback_host("127.0.0.1:4243", 4242));
        assert!(!is_loopback_host("127.0.0.1", 4242));
        assert!(!is_loopback_host("evil.example:4242", 4242));
        assert!(!is_loopback_host("localhost.evil.example:4242", 4242));
    }

    #[test]
    fn extracts_the_token_from_any_query_position() {
        assert_eq!(token_from_query("t=abc"), Some("abc"));
        assert_eq!(token_from_query("x=1&t=abc"), Some("abc"));
        assert_eq!(token_from_query("tt=abc"), None);
    }

    #[test]
    fn check_requires_both_a_loopback_host_and_the_right_token() {
        let guard = guard();
        let good_query = format!("t={}", "cafe".repeat(16));
        assert_eq!(
            guard.check(Some("127.0.0.1:4242"), Some(&good_query)),
            Ok(())
        );
        assert_eq!(
            guard.check(Some("evil.example:4242"), Some(&good_query)),
            Err(Rejection::ForeignHost)
        );
        assert_eq!(
            guard.check(None, Some(&good_query)),
            Err(Rejection::ForeignHost)
        );
        assert_eq!(
            guard.check(Some("127.0.0.1:4242"), Some("t=wrong")),
            Err(Rejection::InvalidToken)
        );
        assert_eq!(
            guard.check(Some("127.0.0.1:4242"), None),
            Err(Rejection::InvalidToken)
        );
    }
}
