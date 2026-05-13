// loan_machine_web/src/wallet_auth/server_fn_client.rs
//
// AuthedBrowserClient wraps Leptos's BrowserClient to inject the user's
// Privy bearer token into every outgoing server-fn request.
//
// Architecture:
//   • Pure helper `auth_header_value` does the only thing worth testing.
//   • WASM impl: real Client<E> impl that fetches the token from JS,
//     sets the Authorization header, then delegates to BrowserClient.
//   • SSR alias: re-export BrowserClient. SSR never calls its own
//     server fns over HTTP, so the impl is never invoked there.

// ── Pure (testable on any target) ──────────────────────────────

/// Build the value of the `Authorization` header for a given token state.
/// Returns `None` when there's no token — caller skips header insertion.
///
/// Extracted as a free function so it's unit-testable without WASM. A
/// silent typo here ("Beare", wrong case, missing space) turns every
/// authenticated request into a 401, so this small surface is worth
/// pinning down.
pub(crate) fn auth_header_value(token: Option<&str>) -> Option<String> {
    token.map(|t| format!("Bearer {t}"))
}

// ── WASM: real Client impl ────────────────────────────────────

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::future::Future;
    use bytes::Bytes;
    use futures::{Sink, Stream};
    use send_wrapper::SendWrapper;

    use leptos::server_fn::client::{browser::BrowserClient, Client};
    use leptos::server_fn::error::FromServerFnError;
    use leptos::server_fn::request::browser::BrowserRequest;
    use leptos::server_fn::response::browser::BrowserResponse;

    use super::auth_header_value;
    use crate::wallet_auth::auth_header::current_access_token;

    pub struct AuthedBrowserClient;

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

        fn open_websocket(url: &str) -> impl Future<
            Output = Result<
                (impl Stream<Item = Result<Bytes, Bytes>> + Send + 'static,
                 impl Sink<Bytes> + Send + 'static),
                E,
            >,
        > + Send {
            <BrowserClient as Client<E>>::open_websocket(url)
        }

        fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
            <BrowserClient as Client<E>>::spawn(fut)
        }
    }
}

// ── SSR: alias to BrowserClient ───────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    pub use leptos::server_fn::client::browser::BrowserClient as AuthedBrowserClient;
}

pub use imp::AuthedBrowserClient;