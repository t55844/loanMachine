// gas_modal.rs

use leptos::prelude::*;
use crate::components::ui::*;

#[derive(Clone)]
pub struct GasEstimate {
    pub label:   String,
    pub gas_hex: String,
}

impl GasEstimate {
    pub fn gas_units(&self) -> u64 {
        let s = self.gas_hex.trim_start_matches("0x");
        u64::from_str_radix(s, 16).unwrap_or(0)
    }
}

// No GasPricing here — the modal fetches it itself.
#[derive(Clone)]
pub struct GasModalRequest {
    pub title:      String,
    pub estimates:  Vec<GasEstimate>,
    pub on_confirm: Callback<()>,
}

// ── Context ──────────────────────────────────────────────────

pub fn provide_gas_modal() {
    let (req, set_req) = signal(Option::<GasModalRequest>::None);
    provide_context(req);
    provide_context(set_req);
}

pub fn use_gas_modal() -> WriteSignal<Option<GasModalRequest>> {
    expect_context::<WriteSignal<Option<GasModalRequest>>>()
}

// ── Price fetching ────────────────────────────────────────────
// CoinGecko's free /simple/price endpoint — no API key needed,
// rate-limited at ~30 req/min which is fine for modal usage.
// Returns (eth_usd, usd_brl).

async fn fetch_prices() -> Option<(f64, f64)> {
    let url = "https://api.coingecko.com/api/v3/simple/price\
               ?ids=ethereum&vs_currencies=usd,brl";

    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Reflect;
        use wasm_bindgen::{JsCast, JsValue};
        use wasm_bindgen_futures::JsFuture;
        use web_sys::Response;

        // Typed fetch + Response::json — same JS calls underneath,
        // a quarter of the lines and no Reflect dance for the network step.
        let window   = web_sys::window()?;
        let response: Response = JsFuture::from(window.fetch_with_str(url))
            .await.ok()?
            .dyn_into().ok()?;
        let json = JsFuture::from(response.json().ok()?).await.ok()?;

        // The JSON shape is { ethereum: { usd, brl } } — small enough
        // to walk with Reflect rather than pulling in serde-wasm-bindgen.
        let eth_obj = Reflect::get(&json, &JsValue::from_str("ethereum")).ok()?;
        let eth_usd = Reflect::get(&eth_obj, &JsValue::from_str("usd"))
            .ok().and_then(|v| v.as_f64())?;
        let brl_per_eth = Reflect::get(&eth_obj, &JsValue::from_str("brl"))
            .ok().and_then(|v| v.as_f64())?;

        Some((eth_usd, brl_per_eth / eth_usd))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url;
        None
    }
}

// ── GasModal ──────────────────────────────────────────────────

#[component]
pub fn GasModal() -> impl IntoView {
    let req   = expect_context::<ReadSignal<Option<GasModalRequest>>>();
    let close = expect_context::<WriteSignal<Option<GasModalRequest>>>();

    // Resource fires whenever the modal opens (req goes from None → Some).
    // When req is None the resource is idle — no unnecessary fetches.
    //
    // Trade-off worth knowing: this *does* refetch on every open, even if
    // the modal was just open ten seconds ago. CoinGecko's free tier (~30
    // req/min) makes this fine in practice; if you ever want it cached for
    // the session, lift the Resource into a context at app root and have
    // this component read from it instead of owning its own.
    let prices = LocalResource::new(
        move || {
            let open = req.get().is_some();
            async move {
                if open { fetch_prices().await } else { None }
            }
        },
    );

    move || {
        req.get().map(|r| {
            // `Callback<T>` is Copy in Leptos 0.7 — no clone needed.
            let on_confirm = r.on_confirm;
            let title      = r.title.clone();

            view! {
                <div
                    class="gas-modal-backdrop"
                    on:click=move |_| close.set(None)
                >
                    <div
                        class="gas-modal-card"
                        on:click=|e| e.stop_propagation()
                    >
                        <span class="card-tag">{title}</span>

                        <div class="flex-col gap-6" style="margin-top: var(--sp-6)">
                            <Alert kind=AlertKind::Info>
                                "Revise o custo estimado de gas antes de assinar."
                            </Alert>

                            {r.estimates.iter().map(|est| {
                                let units = est.gas_units();
                                let label = est.label.clone();
                                let hex   = est.gas_hex.clone();

                                view! {
                                    <div class="gas-estimate-block">

                                        // ── Raw gas ──────────────────
                                        <div class="stat-block">
                                            <span class="stat-label">{label}</span>
                                            <span class="t-mono-sm t-bright">
                                                {format!("{units} gas units")}
                                            </span>
                                            <span class="t-mono-xs t-muted">{hex}</span>
                                        </div>

                                        // ── Price breakdown ───────────
                                        // Suspense shows a spinner while the
                                        // fetch is in-flight, then swaps in
                                        // the values — no extra signals needed.
                                        <Suspense fallback=move || view! {
                                            <div class="gas-price-loading">
                                                <span class="spinner"
                                                    style="width:14px;height:14px;border-width:2px"
                                                />
                                                <span class="t-mono-xs t-muted">
                                                    "buscando cotações…"
                                                </span>
                                            </div>
                                        }>
                                            {move || {
                                                // gas_price_gwei: Anvil = 1 gwei.
                                                // In production, expose an endpoint
                                                // that reads eth_gas_price from the
                                                // RPC and return it in the bundle.
                                                let gas_price_gwei = 1.0_f64;
                                                let wei = units as f64 * gas_price_gwei * 1e9;
                                                let eth = wei / 1e18;

                                                match prices.get().flatten() {
                                                    None => view! {
                                                        <span class="t-mono-xs t-muted">
                                                            "cotação indisponível"
                                                        </span>
                                                    }.into_any(),

                                                    Some((eth_usd, usd_brl)) => {
                                                        let usd = eth * eth_usd;
                                                        let brl = usd * usd_brl;
                                                        view! {
                                                            <div class="gas-price-breakdown">
                                                                <div class="gas-price-row">
                                                                    <span class="gas-price-label">"Wei" </span>
                                                                    <span class="gas-price-value">
                                                                        {format!("{wei:.0}")}
                                                                    </span>
                                                                </div>
                                                                <div class="gas-price-row">
                                                                    <span class="gas-price-label">"ETH"</span>
                                                                    <span class="gas-price-value t-yellow">
                                                                        {format!("{eth:.6}")}
                                                                    </span>
                                                                </div>
                                                                <div class="gas-price-row">
                                                                    <span class="gas-price-label">"USD"</span>
                                                                    <span class="gas-price-value t-green">
                                                                        {format!("${usd:.2}")}
                                                                    </span>
                                                                </div>
                                                                <div class="gas-price-row">
                                                                    <span class="gas-price-label">"BRL"</span>
                                                                    <span class="gas-price-value t-green">
                                                                        {format!("R${brl:.2}")}
                                                                    </span>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    }
                                                }
                                            }}
                                        </Suspense>
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        <div class="gas-modal-actions">
                            <Button
                                variant=BtnVariant::Primary
                                full_width=true
                                on_click=Box::new(move || {
                                    on_confirm.run(());
                                    close.set(None);
                                })
                            >
                                "CONFIRMAR E ASSINAR"
                            </Button>
                            <Button
                                variant=BtnVariant::Ghost
                                full_width=true
                                on_click=Box::new(move || close.set(None))
                            >
                                "CANCELAR"
                            </Button>
                        </div>
                    </div>
                </div>
            }
        })
    }
}