// loan_machine_web/src/services/prices.rs
//
// Single source of ETH→USD pricing for the client.
// Cached in localStorage with a 6h TTL.  Only AuthBar should call
// ensure_fresh(); everything else reads via the context signal that
// AuthBar publishes.

use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "lm.eth_prices.v1";
const TTL_MILLIS:  f64  = 6.0 * 60.0 * 60.0 * 1000.0;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Prices {
    pub eth_usd: f64,
}

#[derive(Serialize, Deserialize)]
struct Cached {
    fetched_at_ms: f64,
    prices:        Prices,
}

// ── Public API ─────────────────────────────────────────────

/// Sync read.  Returns whatever is in localStorage — even if stale.
/// Cheap; safe to call from any render path.
pub fn read_cached() -> Option<Prices> {
    read_storage().map(|c| c.prices)
}

/// Fetches from CoinGecko only when the cached entry is missing or
/// older than the TTL.  Writes back on success.  Returns whatever
/// price we end up with (cached-stale on network failure).
pub async fn ensure_fresh() -> Option<Prices> {
    if let Some(c) = read_storage() {
        if now_ms() - c.fetched_at_ms < TTL_MILLIS {
            return Some(c.prices);
        }
    }
    match fetch_from_coingecko().await {
        Some(p) => {
            write_storage(&Cached { fetched_at_ms: now_ms(), prices: p });
            Some(p)
        }
        // Network down → fall back to stale cache, better than nothing.
        None => read_storage().map(|c| c.prices),
    }
}

// ── Internals ──────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 { js_sys::Date::now() }
#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 { 0.0 }

#[cfg(target_arch = "wasm32")]
fn read_storage() -> Option<Cached> {
    let storage = web_sys::window()?.local_storage().ok()??;
    let raw     = storage.get_item(STORAGE_KEY).ok()??;
    serde_json::from_str(&raw).ok()
}
#[cfg(not(target_arch = "wasm32"))]
fn read_storage() -> Option<Cached> { None }

#[cfg(target_arch = "wasm32")]
fn write_storage(c: &Cached) {
    let Some(w) = web_sys::window() else { return };
    let Ok(Some(s)) = w.local_storage() else { return };
    let Ok(json) = serde_json::to_string(c) else { return };
    let _ = s.set_item(STORAGE_KEY, &json);
}
#[cfg(not(target_arch = "wasm32"))]
fn write_storage(_: &Cached) {}

// Verbatim from the old gas_modal::fetch_prices, just relocated.
#[cfg(target_arch = "wasm32")]
async fn fetch_from_coingecko() -> Option<Prices> {
    use js_sys::Reflect;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::Response;

    let url = "https://api.coingecko.com/api/v3/simple/price\
               ?ids=ethereum&vs_currencies=usd";

    let window   = web_sys::window()?;
    let response: Response = JsFuture::from(window.fetch_with_str(url))
        .await.ok()?
        .dyn_into().ok()?;
    let json = JsFuture::from(response.json().ok()?).await.ok()?;

    let eth_obj = Reflect::get(&json, &JsValue::from_str("ethereum")).ok()?;
    let eth_usd = Reflect::get(&eth_obj, &JsValue::from_str("usd"))
        .ok().and_then(|v| v.as_f64())?;

    Some(Prices { eth_usd })
}
#[cfg(not(target_arch = "wasm32"))]
async fn fetch_from_coingecko() -> Option<Prices> { None }

// ── Context (provided by AuthBar at mount, consumed elsewhere) ──

use leptos::prelude::*;

/// AuthBar calls this once at mount.  Provides a reactive ReadSignal
/// holding the latest known prices, hydrated from localStorage
/// synchronously and refreshed in the background.
pub fn provide_prices_context() {
    let (prices, set_prices) = signal(read_cached());
    provide_context(prices);

    #[cfg(target_arch = "wasm32")]
    leptos::task::spawn_local(async move {
        if let Some(p) = ensure_fresh().await {
            set_prices.set(Some(p));
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _ = set_prices;
}

/// Read-only access for consumers (GasModal, AuthBar balance row, etc.)
pub fn use_prices() -> ReadSignal<Option<Prices>> {
    use_context::<ReadSignal<Option<Prices>>>().unwrap_or_else(|| {
        let (r, _) = signal(None);
        r
    })
}