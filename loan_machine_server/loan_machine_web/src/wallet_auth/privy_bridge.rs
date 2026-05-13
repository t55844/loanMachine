// src/privy_bridge.rs
#![cfg(target_arch = "wasm32")]

use leptos::prelude::*;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use loan_machine_models::wallet_address::{WalletAddress };
use crate::wallet_auth::session::WalletSession;

/// Wire up Privy → WalletSession.
/// Must run during hydration (inside a component body or Effect).
#[cfg(target_arch = "wasm32")]
pub fn install(set: WriteSignal<WalletSession>) {
    let window = web_sys::window().expect("no window");

    // wallet ready
    {
        let set = set;  // WriteSignal is Copy
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let raw = js_sys::Reflect::get(&e.detail(), &JsValue::from_str("address"))
                .ok().and_then(|v| v.as_string());
            match raw.as_deref().map(str::parse::<WalletAddress>) {
                Some(Ok(wallet)) => set.set(WalletSession::Connected { wallet }),
                Some(Err(err))   => web_sys::console::error_1(
                    &JsValue::from_str(&format!("[rust] bad wallet from privy: {err}"))
                ),
                None => {}
            }
        });
        window.add_event_listener_with_callback("privy_wallet_ready", cb.as_ref().unchecked_ref())
            .expect("listener");
        cb.forget();
    }

    // logged out
    {
        let set = set;
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |_| {
            set.set(WalletSession::Disconnected);
        });
        window.add_event_listener_with_callback("privy_logged_out", cb.as_ref().unchecked_ref())
            .expect("listener");
        cb.forget();
    }

    // Optional but recommended: privy-bridge.js dispatches this when restore
    // finishes without a session, so we can leave Restoring.
    {
        let set = set;
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |_| {
            set.update(|s| if matches!(s, WalletSession::Restoring) {
                *s = WalletSession::Disconnected;
            });
        });
        window.add_event_listener_with_callback("privy_restore_done", cb.as_ref().unchecked_ref())
            .expect("listener");
        cb.forget();
    }

    call_when_ready("loan_machine_try_restore", 30, 100);
}

fn call_when_ready(name: &'static str, max_attempts: u32, interval_ms: i32) {
    use std::cell::RefCell;
    use std::rc::Rc;

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

pub fn login()  { call_window_fn("loan_machine_init_privy") }
pub fn logout() { call_window_fn("loan_machine_logout") }

fn call_window_fn(name: &str) {
    let Some(window) = web_sys::window() else { return };
    let Ok(val) = js_sys::Reflect::get(&window, &JsValue::from_str(name)) else { return };
    if let Ok(func) = val.dyn_into::<js_sys::Function>() {
        let _ = func.call0(&JsValue::NULL);
    }
}