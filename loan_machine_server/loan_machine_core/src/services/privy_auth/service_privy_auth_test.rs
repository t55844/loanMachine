// loan_machine_core/src/services/privy_auth/tests.rs
//
// Strategy: generate a real ES256 (P-256) keypair per test, serve the matching
// public key as JWKS from a wiremock server, sign tokens with the private key,
// and let the actual `jsonwebtoken` crate verify them. No mocking of crypto
// — we exercise the real signature path.
//
// Coverage:
//   1. Happy path — valid token returns claims.
//   2. Wrong audience → InvalidToken.
//   3. Wrong issuer → InvalidToken.
//   4. Expired token → InvalidToken.
//   5. Token header missing kid → MissingKid.
//   6. kid not in JWKS (key rotation) → KeyNotFound.
//   7. Token signed by a different key with same kid → InvalidToken.
//   8. JWKS endpoint returns 500 → FetchJwks (and source chain preserved).
//   9. JWKS cached across multiple verify() calls (one HTTP fetch).
//  10. Cache refetches when stale (TTL = 0).

use super::*;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use p256::ecdsa::SigningKey;
use p256::pkcs8::{EncodePrivateKey, LineEnding};
use rand::rngs::OsRng;
use serde::Serialize;
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const APP_ID: &str = "test-app-id";
const JWKS_PATH: &str = "/jwks";

#[derive(Serialize)]
struct TestClaims<'a> {
    sub: &'a str,
    aud: &'a str,
    iss: &'a str,
    exp: usize,
}

struct Keypair {
    encoding_key: EncodingKey,
    jwk_json: Value,
    kid: String,
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Generate a fresh ES256 keypair, packaged with both the JWT-signing key
/// and the JWK representation of the public side.
fn make_keypair(kid: &str) -> Keypair {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    // Uncompressed point: 0x04 || X(32) || Y(32). JWK wants X and Y separately,
    // base64url-encoded, no padding.
    let point = verifying_key.to_encoded_point(false);
    let x = point.x().expect("uncompressed point has X");
    let y = point.y().expect("uncompressed point has Y");

    let jwk_json = json!({
        "kty": "EC",
        "crv": "P-256",
        "x":   b64url(x.as_ref()),
        "y":   b64url(y.as_ref()),
        "kid": kid,
        "use": "sig",
        "alg": "ES256",
    });

    let pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .expect("encode private key as PKCS#8 PEM");
    let encoding_key =
        EncodingKey::from_ec_pem(pem.as_bytes()).expect("EncodingKey from PKCS#8 PEM");

    Keypair {
        encoding_key,
        jwk_json,
        kid: kid.to_string(),
    }
}

fn now_secs() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

fn sign_with_kid(kp: &Keypair, claims: &TestClaims, kid: Option<&str>) -> String {
    let mut header = Header::new(Algorithm::ES256);
    header.kid = kid.map(|s| s.to_string());
    encode(&header, claims, &kp.encoding_key).expect("sign token")
}

fn sign(kp: &Keypair, claims: &TestClaims) -> String {
    sign_with_kid(kp, claims, Some(&kp.kid))
}

fn valid_claims() -> TestClaims<'static> {
    TestClaims {
        sub: "did:privy:abc123",
        aud: APP_ID,
        iss: "privy.io",
        exp: now_secs() + 3600,
    }
}

async fn server_serving_jwks(jwks: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(JWKS_PATH))
        .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
        .mount(&server)
        .await;
    server
}

fn service(server: &MockServer, ttl: Duration) -> PrivyAuthService {
    PrivyAuthService::with_url(
        format!("{}{}", server.uri(), JWKS_PATH),
        APP_ID.to_string(),
        ttl,
    )
}

fn jwks_with(keys: &[&Value]) -> Value {
    json!({ "keys": keys })
}

// --- 1. Happy path -------------------------------------------------------

#[tokio::test]
async fn verify_returns_claims_for_valid_token() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let token = sign(&kp, &valid_claims());
    let claims = svc.verify(&token).await.expect("verification should succeed");

    assert_eq!(claims.sub, "did:privy:abc123");
    assert_eq!(claims.aud, APP_ID);
    assert_eq!(claims.iss, "privy.io");
}

// --- 2. Wrong audience ---------------------------------------------------

#[tokio::test]
async fn rejects_wrong_audience() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let claims = TestClaims {
        aud: "different-app",
        ..valid_claims()
    };
    let token = sign(&kp, &claims);

    let err = svc.verify(&token).await.unwrap_err();
    assert!(
        matches!(err, PrivyAuthError::InvalidToken(_)),
        "expected InvalidToken, got {err:?}"
    );
}

// --- 3. Wrong issuer -----------------------------------------------------

#[tokio::test]
async fn rejects_wrong_issuer() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let claims = TestClaims {
        iss: "evil.example.com",
        ..valid_claims()
    };
    let token = sign(&kp, &claims);

    let err = svc.verify(&token).await.unwrap_err();
    assert!(
        matches!(err, PrivyAuthError::InvalidToken(_)),
        "expected InvalidToken, got {err:?}"
    );
}

// --- 4. Expired token ----------------------------------------------------

#[tokio::test]
async fn rejects_expired_token() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    // Validation has 60s default leeway, so go well past it.
    let claims = TestClaims {
        exp: now_secs() - 600,
        ..valid_claims()
    };
    let token = sign(&kp, &claims);

    let err = svc.verify(&token).await.unwrap_err();
    assert!(
        matches!(err, PrivyAuthError::InvalidToken(_)),
        "expected InvalidToken, got {err:?}"
    );
}

// --- 5. Token header missing kid -----------------------------------------

#[tokio::test]
async fn errors_when_token_missing_kid() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let token = sign_with_kid(&kp, &valid_claims(), None);

    let err = svc.verify(&token).await.unwrap_err();
    assert!(
        matches!(err, PrivyAuthError::MissingKid),
        "expected MissingKid, got {err:?}"
    );
}

// --- 6. kid not in JWKS (key rotation) -----------------------------------

#[tokio::test]
async fn errors_when_kid_not_in_jwks() {
    let kp_token = make_keypair("kid-stale");
    let kp_jwks  = make_keypair("kid-current"); // different kid published
    let server = server_serving_jwks(jwks_with(&[&kp_jwks.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let token = sign(&kp_token, &valid_claims());

    let err = svc.verify(&token).await.unwrap_err();
    match err {
        PrivyAuthError::KeyNotFound(k) => assert_eq!(k, "kid-stale"),
        other => panic!("expected KeyNotFound, got {other:?}"),
    }
}

// --- 7. Wrong signature (impersonation attempt) --------------------------

#[tokio::test]
async fn rejects_token_signed_by_different_key() {
    // Same kid, different keys: attacker tries to impersonate.
    let kp_attacker = make_keypair("kid-1");
    let kp_real     = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp_real.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let token = sign(&kp_attacker, &valid_claims());

    let err = svc.verify(&token).await.unwrap_err();
    assert!(
        matches!(err, PrivyAuthError::InvalidToken(_)),
        "expected InvalidToken (signature mismatch), got {err:?}"
    );
}

// --- 8. JWKS fetch failure (and source chain preserved) ------------------

#[tokio::test]
async fn jwks_fetch_failure_surfaces_as_fetch_jwks() {
    use std::error::Error;

    let kp = make_keypair("kid-1");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(JWKS_PATH))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let svc = service(&server, Duration::from_secs(300));

    let token = sign(&kp, &valid_claims());

    let err = svc.verify(&token).await.unwrap_err();
    assert!(
        matches!(err, PrivyAuthError::FetchJwks(_)),
        "expected FetchJwks, got {err:?}"
    );

    // The thiserror payoff: the underlying reqwest::Error is reachable via source().
    assert!(
        err.source().is_some(),
        "FetchJwks should expose the underlying reqwest::Error via source()"
    );
}

// --- 9. JWKS is cached across calls --------------------------------------

#[tokio::test]
async fn jwks_is_fetched_once_across_verifications() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    let svc = service(&server, Duration::from_secs(300));

    let token = sign(&kp, &valid_claims());

    svc.verify(&token).await.unwrap();
    svc.verify(&token).await.unwrap();
    svc.verify(&token).await.unwrap();

    let received = server.received_requests().await.unwrap();
    assert_eq!(
        received.len(),
        1,
        "JWKS should be fetched only once across multiple verifications"
    );
}

// --- 10. Cache refetches when stale --------------------------------------

#[tokio::test]
async fn jwks_refetched_when_cache_stale() {
    let kp = make_keypair("kid-1");
    let server = server_serving_jwks(jwks_with(&[&kp.jwk_json])).await;
    // TTL = 0 → every check is stale → every verify refetches.
    let svc = service(&server, Duration::from_secs(0));

    let token = sign(&kp, &valid_claims());

    svc.verify(&token).await.unwrap();
    svc.verify(&token).await.unwrap();

    let received = server.received_requests().await.unwrap();
    assert_eq!(
        received.len(),
        2,
        "JWKS should be refetched when cache is stale"
    );
}