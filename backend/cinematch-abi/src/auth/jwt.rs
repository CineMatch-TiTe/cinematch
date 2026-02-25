use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cinematch_common::Config;

use crate::domain::DomainError;

/// Claims we encode in the JWT.  We keep it simple so we only need
/// a subject and an expiration time.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

/// Sign a token for the provided user ID.  The token lives for one hour
/// (3600 seconds).  The secret is pulled from the global configuration.
///
/// Returns a compact JWT string on success or a `DomainError::Internal` if
/// signing fails.
pub fn sign_token(user_id: Uuid) -> Result<String, DomainError> {
    let config = Config::get();
    let secret = config.secret_token.expose_secret();

    let now = Utc::now();
    let lifetime = Config::get().jwt_expiry_secs;
    let exp = (now + Duration::seconds(lifetime as i64)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| DomainError::Internal(format!("JWT sign error: {}", e)))
}

/// Decode the claims from a token without discarding the expiration.
///
/// This helper is used internally by both the extractor and `verify_token` to
/// pull out the expiration time as well as the subject.  It returns a copy of
/// the `Claims` struct or a `DomainError::Unauthorized` if validation fails.
/// Decode the claims from a token and return the extracted user ID and
/// expiration timestamp.  This keeps the `Claims` struct private to the
/// module, avoiding accidental leakage of internal fields.
pub fn decode_claims(token: &str) -> Result<(Uuid, i64), DomainError> {
    let config = Config::get();
    let secret = config.secret_token.expose_secret();

    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| DomainError::Unauthorized(format!("Invalid token: {}", e)))?;

    let claims = token_data.claims;
    // explicitly enforce expiration even if jsonwebtoken skips it for some reason
    let now = Utc::now().timestamp() as usize;
    if claims.exp < now {
        return Err(DomainError::Unauthorized("Token has expired".to_string()));
    }

    let uid = Uuid::parse_str(&claims.sub)
        .map_err(|_| DomainError::Unauthorized("Bad user id in token".to_string()))?;
    Ok((uid, claims.exp as i64))
}

/// Verify a token and return the contained user ID.  If the token is expired or
/// tampered with, return `DomainError::Unauthorized`.
pub fn verify_token(token: &str) -> Result<Uuid, DomainError> {
    let (uid, _) = decode_claims(token)?;
    Ok(uid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use uuid::Uuid;

    // Ensure the environment has a secret token large enough before the
    // global config is initialized.
    fn set_secret() {
        // ensure all config values are present so `Config::get()` doesn't panic
        let long = "x".repeat(80);
        unsafe {
            env::set_var("SECRET_TOKEN", long.clone());
            env::set_var("DATABASE_URL", "postgres://localhost/test");
            env::set_var("REDIS_URL", "redis://localhost");
            env::set_var("QDRANT_URL", "http://localhost");
        }
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        set_secret();
        let uid = Uuid::new_v4();
        let token = sign_token(uid).expect("sign token");
        let claims_uid = verify_token(&token).expect("verify token");
        assert_eq!(claims_uid, uid);

        // also exercise decode_claims and check expiration is roughly an hour
        let (decoded_uid, exp) = decode_claims(&token).expect("decode claims");
        assert_eq!(decoded_uid, uid);
        let now = Utc::now().timestamp() as usize;
        assert!(exp as usize > now);
    }

    #[test]
    fn expired_token_is_rejected() {
        set_secret();

        // create a token with expiry in the past by directly constructing
        let old = Utc::now() - Duration::seconds(10);
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            exp: old.timestamp() as usize,
        };
        let secret = Config::get().secret_token.expose_secret();
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // make sure decode_claims also fails similarly (expired check)
        assert!(decode_claims(&token).is_err());

        let err = verify_token(&token).unwrap_err();
        match err {
            DomainError::Unauthorized(_) => {}
            _ => panic!("expected unauthorized on expired"),
        }
    }

    #[test]
    fn tampered_token_is_rejected() {
        set_secret();
        let uid = Uuid::new_v4();
        let mut token = sign_token(uid).expect("sign");
        // flip a character in the payload
        if let Some(ch) = token.as_mut_str().chars().nth(10) {
            let replacement = if ch == 'a' { 'b' } else { 'a' };
            token.replace_range(10..11, &replacement.to_string());
        }
        let err = verify_token(&token).unwrap_err();
        match err {
            DomainError::Unauthorized(_) => {}
            _ => panic!("expected unauthorized on tamper"),
        }
    }
}
