# Authentication & Authorization

This document is a reference for the auth subsystem of the loan-machine app. It covers, in order, the server-side token verifier, the wallet model that flows across the boundary, the server-fn middleware that gates protected endpoints, the WASM-side token retrieval and request-client wiring, the session state machine, and the JS/Rust bridge into Privy.

## Overview

Authentication uses two independent pieces of state, joined by the user's Privy ID (`sub`):

```
   ┌─ JWT (Privy access token) ─────────┐
   │   Verified locally against JWKS.    │
   │   Proves identity. Cheap.           │   →  PrivyClaims { sub, aud, iss, exp }
   └─────────────────────────────────────┘

   ┌─ Wallet lookup ────────────────────┐
   │   Given a sub, fetch the user's    │
   │   linked embedded wallet from      │   →  WalletAddress
   │   Privy's REST API. Cached.        │
   └────────────────────────────────────┘
```

The split is deliberate. JWT verification has no network cost beyond a JWKS fetch cached for 5 minutes. The wallet lookup is a separate HTTP call cached per user for 15 minutes, and is only done by server fns that actually need the on-chain identity. Most server fns only care that *someone* is logged in and never read the wallet.

The end-to-end flow:

```
browser                          server fn                       Privy
─────                            ─────────                       ─────
user logged in via Privy
                                
call list_cooperatives()        
   AuthedBrowserClient::send    
     pulls token from window     
     attaches Authorization      
     forwards to BrowserClient   
                                 Authenticated::require()
                                   reads bearer from headers
                                   PrivyAuthService::verify()
                                     get_jwks() ────────────────→  GET jwks.json
                                     verify signature + claims
                                   ✓ returns PrivyClaims

                                 (optional, if fn needs wallet)
                                 auth.wallet().await
                                   fetch_user_wallet(sub) ──────→  GET /users/{sub}
                                   pick wallet_client_type=privy
                                   parse to WalletAddress
                                   ✓ returns WalletAddress
```

---

## 1. Server: `privy_auth/mod.rs`

`PrivyAuthService` owns JWKS-based token verification and user-record lookup. It's constructed once at startup and lives in `AppState`.

Public surface:

```rust
pub struct PrivyAuthService { /* private */ }

impl PrivyAuthService {
    pub fn new(app_id: String, app_secret: SecretString) -> Self;
    pub async fn verify(&self, token: &str) -> Result<PrivyClaims, PrivyAuthError>;
    pub async fn fetch_user_wallet(&self, sub: &str) -> Result<WalletAddress, PrivyAuthError>;
}

pub struct PrivyClaims {
    pub sub: String,   // Privy DID, e.g. "did:privy:abc123"
    pub aud: String,   // app id
    pub iss: String,   // "privy.io"
    pub exp: usize,
}
```

`verify` fetches the JWKS (cached 5 min), looks up the key by `kid`, validates the ES256 signature, checks audience/issuer/expiry, and returns the claims. `fetch_user_wallet` hits Privy's `/api/v1/users/{sub}` with Basic auth, walks `linked_accounts`, and returns the wallet whose `wallet_client_type` is `"privy"` (the embedded wallet, not a connected MetaMask). Result is cached per `sub` for 15 minutes.

Failure modes are exposed via `PrivyAuthError` — `FetchJwks`, `MissingKid`, `KeyNotFound`, `InvalidToken`, `NoEmbeddedWallet`, `InvalidWalletFromPrivy`. The app secret is wrapped in `SecretString`, never logged, redacted in `Debug`.

---

## 2. Model: `loan_machine_models/wallet_address.rs`

`WalletAddress` is the canonical representation of an Ethereum address across the entire app — server, client, models, and database. It's a 20-byte newtype that exists to keep raw hex strings out of business logic and to make "is this thing actually an address" a one-time question at the parse boundary.

```rust
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct WalletAddress([u8; 20]);
```

The choice of representation matters. Bytes (not hex) means equality and hashing are byte-comparisons, not string-comparisons — `0xABC...` and `0xabc...` are the same address and compare equal automatically. `Copy` is safe because the whole thing is 20 bytes inline, no heap, no clone cost. `Hash` makes it usable as a `HashMap` key directly.

### Construction

```rust
pub fn from_bytes(b: [u8; 20]) -> Self;     // when you already have bytes
let w: WalletAddress = "0x742d35Cc...".parse()?;   // from a hex string
```

`from_str` is the validating path. It rejects anything that doesn't start with `0x`/`0X`, anything not exactly 42 characters, and any non-hex byte. The error type carries the position of the first bad byte:

```rust
pub enum WalletAddressParseError {
    MissingPrefix,
    BadLength(usize),
    BadHex(usize),
}
```

Implements `thiserror::Error`, so it composes into any error enum via `#[from]`.

### Display, Debug, serialization

`Display` writes lowercase hex with `0x` prefix — `0x742d35cc6634c0532925a3b844bc9e7595f0beb1`. `Debug` delegates to `Display` so logs stay readable instead of showing `WalletAddress([116, 45, ...])`.

`Serialize` and `Deserialize` use the hex form. On the wire and in JSON, a wallet is a string; in memory, it's 20 bytes. The hex form is canonical (always lowercase) so two equal wallets always serialize identically — important for caching, fingerprinting, and idempotency.

```rust
let w: WalletAddress = "0x742d35Cc6634c0532925A3b844Bc9e7595f0beb1".parse()?;
serde_json::to_string(&w)?;   // "\"0x742d35cc6634c0532925a3b844bc9e7595f0beb1\""
```

### UI helper

```rust
pub fn short(&self) -> String;   // "0x742d…0beb1"
```

For places where a full address would dominate the layout. Truncates to the first 6 + last 4 hex digits joined by an ellipsis.

### Alloy bridge

The Solidity / RPC layer uses `alloy::primitives::Address`. Two free functions bridge between them without leaking alloy types into the rest of the codebase:

```rust
pub fn to_alloy(w: &WalletAddress) -> Address;
pub fn from_alloy(a: Address) -> WalletAddress;
```

Use these when calling contracts (`to_alloy`) or when receiving an event topic (`from_alloy`). Everywhere else, stay in `WalletAddress`.

### Why a newtype matters

If you used `String` everywhere, every function that takes an address would also have to defensively parse and validate. Bugs from accidentally storing `"0xABC"` where `"0xabc"` was expected, or treating an unvalidated input as if it were a real address, are common. With `WalletAddress` the type system enforces the invariant: if you have a `WalletAddress`, it's a real 20-byte address. The validation cost is paid once, at the edge.

---

## 3. Server: `server_fns/auth.rs` — the `Authenticated` proof

`Authenticated` is the type that proves a server fn has been auth-gated. It's a "type-witness" pattern: the only public constructor performs verification, so possessing an `Authenticated` value is unforgeable evidence that the request was valid.

```rust
pub struct Authenticated { claims: PrivyClaims }

impl Authenticated {
    pub async fn require() -> Result<Self, ServerFnError>;
    pub fn claims(&self) -> &PrivyClaims;
    pub fn user_id(&self) -> &str;
    pub async fn wallet(&self) -> Result<WalletAddress, ServerFnError>;
}
```

`require` extracts the `Authorization: Bearer <token>` header via `leptos_axum::extract`, strips the prefix, calls `PrivyAuthService::verify`, and wraps the resulting claims. A missing header, a malformed header, or a verification failure all return `Err(ServerFnError)` and the server fn exits early.

Three usage patterns inside a `#[server]` body:

```rust
// Pattern A — just gate access. You don't care who, just that they're real.
let _auth = Authenticated::require().await?;

// Pattern B — use the Privy user ID. Cheap; the sub is already in the JWT.
let auth = Authenticated::require().await?;
let user_id: &str = auth.user_id();

// Pattern C — use the on-chain wallet. Costs one cached HTTP round-trip.
let auth = Authenticated::require().await?;
let wallet = auth.wallet().await?;
```

The discipline is one line at the top of every protected server fn. Skip it and the function compiles, but it'll have no way to read `user_id` or `wallet` — the compiler enforces the order: verify first, identify second.

Module-gated with `#![cfg(feature = "ssr")]` because it pulls in axum, the AppState, and the JWKS verifier — none of which exist on the WASM side.

---

## 4. Client: `wallet_auth/auth_header.rs` — token retrieval from JS

The Privy SDK lives in JavaScript. Rust needs the access token to attach as a bearer header. `auth_header.rs` bridges that gap.

```rust
#[cfg(target_arch = "wasm32")]
pub async fn current_access_token() -> Option<String> {
    use js_sys::{Function, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()?;
    let func = Reflect::get(&window, &"loan_machine_get_access_token".into()).ok()?;
    let func = func.dyn_into::<Function>().ok()?;
    let promise: Promise = func.call0(&JsValue::NULL).ok()?.dyn_into().ok()?;
    JsFuture::from(promise).await.ok()?.as_string()
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn current_access_token() -> Option<String> { None }
```

How it works step by step. `web_sys::window()` gives a typed handle to the browser's global `window`. `js_sys::Reflect::get` is the dynamic property accessor — equivalent to `window["loan_machine_get_access_token"]` in JS. The result is a `JsValue` that we cast to `Function` via `dyn_into`. Calling `call0` invokes it with no arguments and returns a `Promise`. `JsFuture::from(promise).await` converts the JS Promise into a Rust future. Finally `as_string()` extracts the resolved value if it was indeed a string.

Every step returns `Option`, and the whole chain uses `?` to short-circuit on any failure. A missing function, a non-function value, a rejected promise, or a non-string resolution all collapse to `None`. That's deliberate — token absence is normal (logged-out users), not exceptional.

The non-wasm stub returns `None` unconditionally so the function can be referenced on SSR without #cfg gates at every call site.

---

## 5. Client: `wallet_auth/server_fn_client.rs` — `AuthedBrowserClient`

Leptos's `#[server]` macro accepts a `client = TypePath` argument that names the HTTP client used to dispatch the function call. The default is `BrowserClient`. We need to attach an `Authorization` header to every request, so we wrap `BrowserClient` with our own type that injects the header before delegating.

The pure piece is testable and trivial:

```rust
pub(crate) fn auth_header_value(token: Option<&str>) -> Option<String> {
    token.map(|t| format!("Bearer {t}"))
}
```

Extracted so a typo in `"Bearer "` (wrong case, missing space, accidental colon) is caught by a unit test rather than discovered in production as silent 401s.

The WASM impl wires it into Leptos's `Client` trait:

```rust
impl<E: FromServerFnError> Client<E> for AuthedBrowserClient {
    type Request  = BrowserRequest;
    type Response = BrowserResponse;

    fn send(req: Self::Request) -> impl Future<Output = Result<Self::Response, E>> + Send {
        SendWrapper::new(async move {
            let token = current_access_token().await;
            if let Some(value) = auth_header_value(token.as_deref()) {
                let _ = req.headers().set("Authorization", &value);
            }
            <BrowserClient as Client<E>>::send(req).await
        })
    }
    // open_websocket and spawn delegate directly.
}
```

`SendWrapper` is needed because `Client::send` requires `+ Send` on its returned future, but the WASM JS-interop machinery is fundamentally not Send (it uses `Rc<RefCell<..>>` under the hood). `SendWrapper` lies to the compiler and panics if used cross-thread — which never happens since WASM is single-threaded. This is the standard idiom for this exact situation.

On SSR, `AuthedBrowserClient` is just an alias for `BrowserClient` itself. SSR never calls its own server functions over HTTP (it calls their bodies directly), so the alias is never actually invoked, but it satisfies the macro's trait bounds on both compile targets.

Usage:

```rust
#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn list_cooperatives() -> Result<Vec<CooperativeView>, ServerFnError> {
    let _auth = Authenticated::require().await?;
    // ...
}
```

Note: the macro takes a *type path*, not a string. `client = "crate::..."` is a parse error.

---

## 6. Client: `wallet_auth/session.rs` — `WalletSession` state machine

`WalletSession` is the reactive source of truth for "what does the UI know about the user's wallet right now." It has exactly three states:

```rust
pub enum WalletSession {
    Restoring,
    Disconnected,
    Connected { wallet: WalletAddress },
}
```

Why three. A two-state enum (`Some(wallet) | None`) can't distinguish "we haven't checked yet" from "we checked, no user." On first paint, before the Privy bridge has reported back, the UI would otherwise flash a "you're logged out" message and then re-render once the session restored. The `Restoring` state lets components show a neutral skeleton during that window. Once the bridge fires either `privy_wallet_ready` or `privy_restore_done`, the state moves out of `Restoring` and never returns to it for the rest of the page's lifetime.

The context wiring is one signal pair shared across the whole app:

```rust
#[derive(Copy, Clone)]
pub struct WalletCtx {
    pub session: ReadSignal<WalletSession>,
    pub set:     WriteSignal<WalletSession>,
}

pub fn provide_wallet() -> WalletCtx { /* signal + provide_context */ }
pub fn use_wallet() -> WalletCtx     { expect_context::<WalletCtx>() }
```

`App` calls `provide_wallet()` once during hydration. Any component downstream reads `use_wallet().session.get()` to react to changes. The `set` half is held by `privy_bridge::install` (which flips it from JS events) and a few logout buttons.

The convenience accessor `WalletSession::wallet()` returns `Option<&WalletAddress>` for the common "I just want the wallet if there is one" case.

---

## 7. The JS bridge: `public/privy-bridge.js`

The bridge is the only file in the project that imports the Privy SDK. It exposes a handful of functions on `window` for Rust to call, and dispatches custom DOM events for Rust to listen on.

### Functions installed on `window`

```js
window.loan_machine_try_restore        // dispatches restore result
window.loan_machine_init_privy         // email+OTP login flow, dispatches result
window.loan_machine_get_access_token   // returns Promise<string|null>
window.loan_machine_send_tx            // signs + broadcasts a tx
window.loan_machine_deploy_contract    // signs + broadcasts a deployment
window.loan_machine_logout             // ends the session
window.__privy_bridge_ready            // sentinel set at end of script
```

### Events dispatched on `window`

| Event                    | Detail                                    | When                          |
|--------------------------|-------------------------------------------|-------------------------------|
| `privy_wallet_ready`     | `{ address }`                             | restore or login succeeded    |
| `privy_restore_done`     | `{}`                                      | restore finished, no session  |
| `privy_logged_out`       | `{}`                                      | logout completed              |
| `privy_auth_error`       | `{ error }`                               | login flow failed             |
| `privy_tx_complete`      | `{ tx_hash }`                             | tx mined                      |
| `privy_tx_error`         | `{ error }`                               | tx failed at any stage        |
| `privy_deploy_complete`  | `{ tx_hash, contract_address }`           | deployment mined              |

### Lifecycle

`getClient()` is lazy — the Privy SDK is constructed on first use, the hidden iframe is mounted, and the message-passing channel between iframe and main window is set up. All other functions go through `getClient()` so the SDK exists when needed and not before.

`tryRestoreSession()` is the path that runs on page load. It checks `localStorage` for a refresh token, POSTs to `/api/v1/sessions` to exchange it for an active session, and if successful builds the EIP-1193 provider for the user's embedded wallet. The first address returned by `eth_requestAccounts` is the wallet address Rust sees.

`loan_machine_try_restore` is the public entry point Rust calls. It dispatches `privy_wallet_ready` on success and `privy_restore_done` on no-session. Never silently fails — silence is what kept the UI stuck on `Restoring` in an earlier iteration.

`loan_machine_init_privy` is the login path. It shows the email and OTP overlays (inline DOM, no React), sends the code via Privy, verifies, then builds the wallet provider. Same `privy_wallet_ready` dispatch as restore.

`loan_machine_send_tx` and `loan_machine_deploy_contract` deliberately bypass Privy's broadcast path. They use Privy's iframe to sign (the private key stays in the enclave), then send the signed RLP bytes directly to the local Anvil node. This avoids Privy attempting to route through their relay infrastructure for chains it doesn't know about.

---

## 8. Rust side of the bridge: `wallet_auth/privy_bridge.rs`

The Rust side installs event listeners on `window` and triggers the initial restore. WASM-only — gated with `#![cfg(target_arch = "wasm32")]` at the module level.

```rust
pub fn install(set: WriteSignal<WalletSession>) {
    // Three listeners — wallet_ready, logged_out, restore_done — each
    // mapping a JS event to a WalletSession state transition.
    //
    // Then: kick off restore.
    call_when_ready("loan_machine_try_restore", 30, 100);
}
```

Each listener is a `Closure::<dyn Fn(web_sys::CustomEvent)>` registered via `add_event_listener_with_callback` and then `forget`-ed — meaning the closure leaks for the lifetime of the page, which is what you want for a global handler. The `WriteSignal` is `Copy`, so each closure captures its own copy.

`call_when_ready` exists because `app.rs` runs `install()` during hydration, but `privy-bridge.js` might not have finished executing yet (it's a module script with imports from a CDN). It polls `window[name]` every 100 ms for up to 30 attempts (3 seconds), calls the function once it appears, and stops. The polling state lives in `Rc<RefCell<...>>` because closures need to mutate it across invocations.

`login()` and `logout()` are one-shot calls into `window.loan_machine_init_privy` and `window.loan_machine_logout` via a shared helper `call_window_fn`. They're invoked from button click handlers in components.

### How `install` is called

```rust
// app.rs
#[cfg(target_arch = "wasm32")]
crate::wallet_auth::privy_bridge::install(wallet_ctx.set);
```

Inside the `App` component, after `provide_wallet()` has put `WalletCtx` into context. From that moment on, JS events drive the session signal, and every component reading `use_wallet().session` reacts automatically.

---

## Writing a new authenticated server fn

The full template:

```rust
use leptos::prelude::*;
use leptos::server_fn::ServerFnError;

#[server(client = crate::wallet_auth::server_fn_client::AuthedBrowserClient)]
pub async fn my_protected_call(arg: Foo) -> Result<Bar, ServerFnError> {
    use crate::server_fns::auth::Authenticated;

    let auth = Authenticated::require().await?;
    // Now use any of:
    //   auth.user_id()          -> &str
    //   auth.wallet().await?    -> WalletAddress
    //   auth.claims()           -> &PrivyClaims
    //
    // The rest of the body runs only if verification succeeded.

    // ... your logic ...

    Ok(result)
}
```

The component side calls it as a plain async function. No token plumbing, no manual header attachment — `AuthedBrowserClient` handles that transparently because of the `client = ` attribute.

```rust
let data = LocalResource::new(|| async move { my_protected_call(foo).await });
```

## Testing

Two layers of tests cover this subsystem:

`loan_machine_core/src/services/privy_auth/tests.rs` exercises the JWT verifier and the wallet lookup against a `wiremock` server, with ephemeral ES256 keypairs generated per test. Coverage includes the happy path, every error variant, JWKS caching, key rotation, and signature-mismatch attacks.

`loan_machine_web/src/wallet_auth/server_fn_client_test.rs` pins down the `Authorization` header format — the one piece of the client wrapper whose silent failure would be hard to catch in browser testing.

Run both with `make test-all` (which runs `make test-web` and `make test-core`). Avoid `cargo leptos test` for unit tests — it tries to compile under `--features=hydrate` and trips over SSR-gated code paths that aren't relevant to the tests themselves.
