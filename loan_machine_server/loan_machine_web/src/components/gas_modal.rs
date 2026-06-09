// gas_modal.rs

use leptos::prelude::*;
use crate::components::ui::*;
use crate::components::helpers::prices::use_prices;

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


// ── GasModal ──────────────────────────────────────────────────

#[component]
pub fn GasModal() -> impl IntoView {
    let req   = expect_context::<ReadSignal<Option<GasModalRequest>>>();
    let close = expect_context::<WriteSignal<Option<GasModalRequest>>>();

    let visible = Signal::derive(move || req.get().is_some());

   let prices = use_prices();

    // Stable handlers — created once at component mount, live as long as GasModal does.
    let on_backdrop_click = move |e: web_sys::MouseEvent| {
        if e.target().as_ref() == e.current_target().as_ref() {
            close.set(None);
        }
    };
    let on_cancel = move || close.set(None);
    let on_confirm_click = move || {
        if let Some(r) = req.get_untracked() {
            r.on_confirm.run(());
        }
        close.set(None);
    };

    view! {
        <div
            class="gas-modal-backdrop"
            class:open=move || visible.get()      // CSS toggles display/visibility
            on:click=on_backdrop_click
        >
            <div class="gas-modal-card">
                // Content reads from req reactively — it can mount/unmount freely
                // because no click handlers live in here.
                {move || req.get().map(|r| view! {
                    <span class="card-tag">{r.title.clone()}</span>
                    <div class="flex-col gap-6" style="margin-top: var(--sp-6)">
                        <Alert kind=AlertKind::Info>
                            "Review the estimated gas cost before signing."
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
                                       {move || {
                                            let gas_price_gwei = 1.0_f64;
                                            let wei = units as f64 * gas_price_gwei * 1e9;
                                            let eth = wei / 1e18;

                                            match prices.get() {
                                                None => view! {
                                                    <span class="t-mono-xs t-muted">"price unavailable"</span>
                                                }.into_any(),
                                                Some(p) => {
                                                    let usd = eth * p.eth_usd;
                                                    let brl = usd * p.usd_brl;
                                                    view! {
                                                        <div class="gas-price-breakdown">
                                                            <div class="gas-price-row">
                                                                <span class="gas-price-label">"Wei"</span>
                                                                <span class="gas-price-value">{format!("{wei:.0}")}</span>
                                                            </div>
                                                            <div class="gas-price-row">
                                                                <span class="gas-price-label">"ETH"</span>
                                                                <span class="gas-price-value t-yellow">{format!("{eth:.6}")}</span>
                                                            </div>
                                                            <div class="gas-price-row">
                                                                <span class="gas-price-label">"USD"</span>
                                                                <span class="gas-price-value t-green">{format!("${usd:.2}")}</span>
                                                            </div>
                                                            <div class="gas-price-row">
                                                                <span class="gas-price-label">"BRL"</span>
                                                                <span class="gas-price-value t-green">{format!("R${brl:.2}")}</span>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                }
                                            }
                                        }}
                                    </div>
                                }
                        }).collect_view()}
                    </div>
                })}

                // Buttons stay mounted. Their on_click closures never get dropped
                // while the modal is alive.
                <div class="gas-modal-actions">
                    <Button
                        variant=BtnVariant::Primary
                        full_width=true
                        on_click=Box::new(on_confirm_click)
                    >
                        "CONFIRM AND SIGN"
                    </Button>
                    <Button
                        variant=BtnVariant::Ghost
                        on_click=Box::new(on_cancel)
                    >
                        "CANCEL"
                    </Button>
                </div>
            </div>
        </div>
    }
}
