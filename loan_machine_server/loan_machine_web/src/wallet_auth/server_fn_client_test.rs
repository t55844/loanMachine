// loan_machine_web/src/wallet_auth/server_fn_client_test.rs
//
// What's covered (native cargo test, no WASM needed):
//   - Bearer prefix and exact format ("Bearer " with one space)
//   - Some(token) → Some("Bearer <token>")
//   - None → None
//   - Empty token is passed through unchanged (server decides what to do)
//   - JWT-shaped tokens with dots, dashes, equals are preserved verbatim
//
// What's NOT covered here, and where it lives:
//   - The window-interop in `current_access_token` — that reads from the
//     JS bridge and only runs on wasm32. Smoke-tested manually in the
//     browser; would need wasm-bindgen-test for automation.
//   - The actual delegation to BrowserClient::send — that's leptos's
//     responsibility; trusting the impl is correct.
//   - End-to-end header-on-the-wire — needs a real BrowserRequest, which
//     means wasm-bindgen-test. Out of scope for unit tests.
//
// What this DOES catch: every formatting mistake that would silently
// turn authenticated requests into 401s (typo in "Bearer", wrong case,
// missing space, accidental colon, etc.). That's the whole point of
// extracting the helper.

use crate::wallet_auth::server_fn_client::auth_header_value;

#[test]
fn formats_bearer_when_token_present() {
    let v = auth_header_value(Some("abc.def.ghi"));
    assert_eq!(v.as_deref(), Some("Bearer abc.def.ghi"));
}

#[test]
fn returns_none_when_no_token() {
    assert_eq!(auth_header_value(None), None);
}

#[test]
fn preserves_jwt_shaped_tokens() {
    // Privy JWTs have dots, dashes, underscores, sometimes trailing
    // base64 padding. Make sure none of that gets mangled.
    let token = "eyJhbGciOiJFUzI1NiIsImtpZCI6ImtpZC0xIn0.\
                 eyJzdWIiOiJkaWQ6cHJpdnk6YWJjIn0.\
                 abc-DEF_ghi==";
    let v = auth_header_value(Some(token)).expect("Some for Some input");
    assert_eq!(v, format!("Bearer {token}"));
}

#[test]
fn empty_token_still_prefixed() {
    // An empty token is suspicious, but we don't second-guess here —
    // the server's verify() will reject it, which is the right layer
    // for that decision. Just confirm we don't drop the prefix.
    assert_eq!(auth_header_value(Some("")).as_deref(), Some("Bearer "));
}

#[test]
fn whitespace_in_token_is_not_trimmed() {
    // Defensive: callers should never pass whitespace, but if they do,
    // we don't silently fix it. Better to send a broken token and
    // surface a server-side error than to mask the bug upstream.
    let v = auth_header_value(Some("  token-with-spaces  "));
    assert_eq!(v.as_deref(), Some("Bearer   token-with-spaces  "));
}

#[test]
fn exact_prefix_is_capital_b_space() {
    // Pin down the exact format. If this ever changes, the change is
    // deliberate and visible in the diff — not a silent regression.
    let v = auth_header_value(Some("x")).expect("Some");
    assert!(v.starts_with("Bearer "), "got: {v:?}");
    assert_eq!(&v[..7], "Bearer ");  // capital B, space, no colon
    assert!(!v.contains(':'),         "must not be 'Bearer:'");
    assert!(!v.starts_with("bearer"), "must be 'Bearer' (capital B)");
}