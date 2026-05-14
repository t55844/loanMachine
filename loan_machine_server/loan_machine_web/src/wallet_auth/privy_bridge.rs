// src/wallet_auth/privy_bridge.rs
//
// Single point of contact between Rust and the Privy JS bridge.
//
// Everything that talks to `window.loan_machine_*` or listens for
// `privy_*` DOM events lives in this file.  Nobody else in the crate
// imports `wasm_bindgen::JsCast`, `js_sys::Reflect`, or
// `web_sys::CustomEvent` — those types stop here.
//
// Every public function works on both wasm and non-wasm targets:
//   - On wasm:  the real implementation.
//   - On SSR:   no-op (subscriptions never fire; sends are dropped;
//               `get_access_token` returns None).
// So callers don't need `#[cfg(target_arch = "wasm32")]` around
// `privy_bridge::send_tx(...)`, `privy_bridge::on_tx_outcome(...)`, etc.
//
// Two listener patterns coexist in this module, by design:
//   - `install` registers session-wide listeners with `.forget()`.
//     They live as long as the page; that's intentional, since they
//     write to the app-root WalletSession signal.
//   - `on_tx_outcome` / `on_deploy_outcome` register per-component
//     listeners with `on_cleanup`.  They go away when the component
//     unmounts so listeners don't pile up across navigations.
//
// Requires `send_wrapper = "0.6"` in Cargo.toml.

use leptos::prelude::*;
use loan_machine_models::wallet_address::WalletAddress;
use crate::wallet_auth::session::WalletSession;

// ── Public types ─────────────────────────────────────────────

/// Outcome of a transaction signed and broadcast by the bridge.
#[derive(Clone, Debug)]
pub enum TxOutcome {
    Complete(String), // tx hash
    Failed(String),   // error message
}

/// Outcome of a contract deployment.
#[derive(Clone, Debug)]
pub enum DeployOutcome {
    Complete {
        tx_hash: String,
        contract_address: String,
    },
    Failed(String),
}

// ── Imperative calls into JS ─────────────────────────────────

/// Start the email + OTP login flow.
pub fn login() {
    #[cfg(target_arch = "wasm32")]
    call_window_fn("loan_machine_init_privy");
}

/// Log out and clear the embedded wallet.
pub fn logout() {
    #[cfg(target_arch = "wasm32")]
    call_window_fn("loan_machine_logout");
}

/// Fetch the current Privy JWT. Returns None if not signed in or on SSR.
pub async fn get_access_token() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;
        use js_sys::{Function, Promise, Reflect};

        let window = web_sys::window()?;
        let func = Reflect::get(&window, &JsValue::from_str("loan_machine_get_access_token"))
            .ok()?
            .dyn_into::<Function>()
            .ok()?;
        let promise: Promise = func.call0(&JsValue::NULL).ok()?.dyn_into().ok()?;
        JsFuture::from(promise).await.ok()?.as_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Sign and broadcast a generic call tx.
///
/// `gas`:
///   - `Some(hex)` — pin the gas limit to this value (skips one
///     `eth_estimateGas` round-trip on the JS side).  Use this when
///     the server-prepared bundle already includes a gas estimate.
///   - `None` — the JS bridge calls `eth_estimateGas` itself.
///
/// Outcome arrives via `on_tx_outcome`.
pub fn send_tx(to: &str, data: &str, gas: Option<&str>) {
    #[cfg(target_arch = "wasm32")]
    {
        let json = match gas {
            Some(g) => format!(r#"{{"to":"{}","data":"{}","gas":"{}"}}"#, to, data, g),
            None    => format!(r#"{{"to":"{}","data":"{}"}}"#, to, data),
        };
        call_window_fn_with_arg("loan_machine_send_tx", &json);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (to, data, gas);
    }
}

/// Sign and broadcast a contract deployment.
/// Outcome arrives via `on_deploy_outcome`.
pub fn deploy_contract(data: &str, gas_hex: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        call_window_fn_with_args2("loan_machine_deploy_contract", data, gas_hex);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (data, gas_hex);
    }
}

// ── Event subscriptions ──────────────────────────────────────

/// Subscribe to tx outcomes for the lifetime of the calling component.
/// Cleanup is automatic — listeners deregister on unmount.
pub fn on_tx_outcome(handler: impl Fn(TxOutcome) + 'static) {
    #[cfg(target_arch = "wasm32")]
    {
        register_two_event_listeners(
            handler,
            "privy_tx_complete",
            "privy_tx_error",
            "tx_hash",
            "error",
            |hash| TxOutcome::Complete(hash),
            |err| TxOutcome::Failed(err),
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handler;
    }
}

/// Subscribe to deploy outcomes for the lifetime of the calling component.
/// Same lifecycle rules as `on_tx_outcome`.
pub fn on_deploy_outcome(handler: impl Fn(DeployOutcome) + 'static) {
    #[cfg(target_arch = "wasm32")]
    {
        use std::rc::Rc;
        use wasm_bindgen::{closure::Closure, JsCast, JsValue};
        use send_wrapper::SendWrapper;

        let window = web_sys::window().expect("no window");
        let handler = Rc::new(handler);

        // Complete: read both tx_hash and contract_address.  Different
        // shape from the tx-outcome case (two fields, not one), so
        // this doesn't share the helper below.
        let h1 = handler.clone();
        let on_complete = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
            move |e: web_sys::CustomEvent| {
                let hash = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("tx_hash"))
                    .ok().and_then(|v| v.as_string()).unwrap_or_default();
                let addr = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("contract_address"))
                    .ok().and_then(|v| v.as_string()).unwrap_or_default();
                h1(DeployOutcome::Complete {
                    tx_hash: hash,
                    contract_address: addr,
                });
            }
        );
        let _ = window.add_event_listener_with_callback(
            "privy_deploy_complete",
            on_complete.as_ref().unchecked_ref(),
        );

        let h2 = handler.clone();
        let on_error = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
            move |e: web_sys::CustomEvent| {
                let err = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("error"))
                    .ok().and_then(|v| v.as_string()).unwrap_or_else(|| "unknown".into());
                h2(DeployOutcome::Failed(err));
            }
        );
        let _ = window.add_event_listener_with_callback(
            "privy_tx_error",
            on_error.as_ref().unchecked_ref(),
        );

        let bundle = SendWrapper::new((window, on_complete, on_error));
        on_cleanup(move || {
            let (window, on_complete, on_error) = bundle.take();
            let _ = window.remove_event_listener_with_callback(
                "privy_deploy_complete",
                on_complete.as_ref().unchecked_ref(),
            );
            let _ = window.remove_event_listener_with_callback(
                "privy_tx_error",
                on_error.as_ref().unchecked_ref(),
            );
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = handler;
    }
}

// ── Session wiring ───────────────────────────────────────────

/// Wire Privy → WalletSession.  Must run during hydration (inside a
/// component body or Effect).  No-op on SSR.
pub fn install(set: WriteSignal<WalletSession>) {
    #[cfg(target_arch = "wasm32")]
    {
        install_wasm(set);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = set;
    }
}

// ─────────────────────────────────────────────────────────────
// WASM-only internals.
// ─────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn install_wasm(set: WriteSignal<WalletSession>) {
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};

    let window = web_sys::window().expect("no window");

    {
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let raw = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("address"))
                .ok()
                .and_then(|v| v.as_string());
            match raw.as_deref().map(str::parse::<WalletAddress>) {
                Some(Ok(wallet)) => set.set(WalletSession::Connected { wallet }),
                Some(Err(err)) => web_sys::console::error_1(&JsValue::from_str(&format!(
                    "[rust] bad wallet from privy: {err}"
                ))),
                None => {}
            }
        });
        window
            .add_event_listener_with_callback("privy_wallet_ready", cb.as_ref().unchecked_ref())
            .expect("listener");
        cb.forget();
    }

    {
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |_| {
            set.set(WalletSession::Disconnected);
        });
        window
            .add_event_listener_with_callback("privy_logged_out", cb.as_ref().unchecked_ref())
            .expect("listener");
        cb.forget();
    }

    {
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |_| {
            set.update(|s| {
                if matches!(s, WalletSession::Restoring) {
                    *s = WalletSession::Disconnected;
                }
            });
        });
        window
            .add_event_listener_with_callback("privy_restore_done", cb.as_ref().unchecked_ref())
            .expect("listener");
        cb.forget();
    }

    call_when_ready("loan_machine_try_restore", 30, 100);
}

/// Shared implementation for the simple "two events, single-field
/// payload each" pattern.  Used by `on_tx_outcome`.  Not used by
/// `on_deploy_outcome` because the complete-event there carries two
/// fields (tx_hash + contract_address).
#[cfg(target_arch = "wasm32")]
fn register_two_event_listeners<O, F, FOk, FErr>(
    handler: F,
    complete_event: &'static str,
    error_event: &'static str,
    ok_field: &'static str,
    err_field: &'static str,
    map_ok: FOk,
    map_err: FErr,
) where
    O: 'static,
    F: Fn(O) + 'static,
    FOk: Fn(String) -> O + 'static,
    FErr: Fn(String) -> O + 'static,
{
    use std::rc::Rc;
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};
    use send_wrapper::SendWrapper;

    let window = web_sys::window().expect("no window");
    let handler = Rc::new(handler);
    let map_ok = Rc::new(map_ok);
    let map_err = Rc::new(map_err);

    let h1 = handler.clone();
    let m1 = map_ok.clone();
    let on_complete = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
        move |e: web_sys::CustomEvent| {
            let val = js_sys::Reflect::get(&e.detail(), &JsValue::from_str(ok_field))
                .ok().and_then(|v| v.as_string()).unwrap_or_default();
            h1(m1(val));
        }
    );
    let _ = window.add_event_listener_with_callback(
        complete_event,
        on_complete.as_ref().unchecked_ref(),
    );

    let h2 = handler.clone();
    let m2 = map_err.clone();
    let on_error = Closure::<dyn Fn(web_sys::CustomEvent)>::new(
        move |e: web_sys::CustomEvent| {
            let val = js_sys::Reflect::get(&e.detail(), &JsValue::from_str(err_field))
                .ok().and_then(|v| v.as_string()).unwrap_or_else(|| "unknown".into());
            h2(m2(val));
        }
    );
    let _ = window.add_event_listener_with_callback(
        error_event,
        on_error.as_ref().unchecked_ref(),
    );

    let bundle = SendWrapper::new((window, on_complete, on_error));
    on_cleanup(move || {
        let (window, on_complete, on_error) = bundle.take();
        let _ = window.remove_event_listener_with_callback(
            complete_event,
            on_complete.as_ref().unchecked_ref(),
        );
        let _ = window.remove_event_listener_with_callback(
            error_event,
            on_error.as_ref().unchecked_ref(),
        );
    });
}

#[cfg(target_arch = "wasm32")]
fn call_window_fn(name: &str) {
    use wasm_bindgen::{JsCast, JsValue};
    let Some(window) = web_sys::window() else { return };
    let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) else { return };
    if let Ok(func) = val.dyn_into::<js_sys::Function>() {
        let _ = func.call0(&JsValue::NULL);
    }
}

#[cfg(target_arch = "wasm32")]
fn call_window_fn_with_arg(name: &str, arg: &str) {
    use wasm_bindgen::{JsCast, JsValue};
    let Some(window) = web_sys::window() else { return };
    let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) else { return };
    if let Ok(func) = val.dyn_into::<js_sys::Function>() {
        let _ = func.call1(&JsValue::NULL, &JsValue::from_str(arg));
    }
}

#[cfg(target_arch = "wasm32")]
fn call_window_fn_with_args2(name: &str, arg1: &str, arg2: &str) {
    use wasm_bindgen::{JsCast, JsValue};
    let Some(window) = web_sys::window() else { return };
    let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) else { return };
    if let Ok(func) = val.dyn_into::<js_sys::Function>() {
        let _ = func.call2(
            &JsValue::NULL,
            &JsValue::from_str(arg1),
            &JsValue::from_str(arg2),
        );
    }
}

/// Poll `window` for a named function and call it once it appears.
/// Used at startup because `privy-bridge.js` may load slightly after
/// hydration.  Self-clearing interval — no Leptos cleanup needed.
#[cfg(target_arch = "wasm32")]
fn call_when_ready(name: &'static str, max_attempts: u32, interval_ms: i32) {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::{closure::Closure, JsCast, JsValue};

    let window = web_sys::window().expect("no window");
    let attempts: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
    let handle: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    let attempts_c = attempts.clone();
    let handle_c = handle.clone();

    let closure = Closure::<dyn FnMut()>::new(move || {
        let Some(win) = web_sys::window() else { return };

        let found = js_sys::Reflect::get(&win, &JsValue::from_str(name))
            .ok()
            .and_then(|v| v.dyn_into::<js_sys::Function>().ok());

        if let Some(f) = found {
            let _ = f.call0(&JsValue::NULL);
            if let Some(h) = handle_c.borrow_mut().take() {
                win.clear_interval_with_handle(h);
            }
            return;
        }

        *attempts_c.borrow_mut() += 1;
        if *attempts_c.borrow() >= max_attempts {
            if let Some(h) = handle_c.borrow_mut().take() {
                win.clear_interval_with_handle(h);
            }
        }
    });

    let h = window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            interval_ms,
        )
        .expect("set_interval");
    *handle.borrow_mut() = Some(h);
    closure.forget();
}