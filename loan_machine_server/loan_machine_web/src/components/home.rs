// src/components/home.rs
use leptos::prelude::*;
use crate::components::ui::*;

#[component]
pub fn HomePage(#[prop(into)] on_login: Callback<()>) -> impl IntoView {
    view! {
        <HeroSection on_login=on_login />
        <FeaturesSection />
        <CtaSection on_login=on_login />
        <HomeFooter />
    }
}

// ── HERO ──────────────────────────────────────────────────────

#[component]
pub(crate) fn HeroSection(#[prop(into)] on_login: Callback<()>) -> impl IntoView {
    view! {
        <section class="hero">
            <div class="container">
                <div style="max-width:680px; display:flex; flex-direction:column; gap:var(--sp-8)">
                    <div>
                        <Badge color=BadgeColor::Green live=true>"Live on Blockchain"</Badge>
                    </div>
                    <h1 class="t-display-xl t-bright" style="line-height:0.92">
                        "COOPERATIVE"
                        <br/>
                        <span class="t-yellow">"LENDING"</span>
                        <br/>
                        "ON CHAIN"
                    </h1>
                    <p class="t-mono-md" style="color:var(--c-gray-5); max-width:480px; line-height:1.8">
                        "A transparent, member-governed loan platform. "
                        "Donate to the pool, fund other members' loans, and access credit — "
                        "all settled on-chain with no central custody."
                    </p>
                    <div class="flex gap-4">
                        <button
                            class="btn btn-primary btn-lg"
                            on:click=move |_| on_login.run(())
                        >
                            "JOIN THE COOPERATIVE"
                        </button>
                        <a href="#features" class="btn btn-ghost btn-lg">"SEE FEATURES"</a>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── FEATURES CAROUSEL ─────────────────────────────────────────

// Reversed order: coop dashboard → my payments → my loans →
//                 request loan → open market → donate
const TOTAL_SLIDES: usize = 6;

#[component]
pub(crate) fn FeaturesSection() -> impl IntoView {
    let current = RwSignal::new(0usize);

    // Auto-advance every 4 s — client only
    Effect::new(move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            use wasm_bindgen::{closure::Closure, JsCast};
            let cb = Closure::<dyn Fn()>::new(move || {
                current.update(|i| *i = (*i + 1) % TOTAL_SLIDES);
            });
            if let Some(w) = web_sys::window() {
                let _ = w.set_interval_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    4000,
                );
            }
            cb.forget();
        }
    });

    let prev = move |_| {
        current.update(|i| *i = if *i == 0 { TOTAL_SLIDES - 1 } else { *i - 1 });
    };
    let next = move |_| {
        current.update(|i| *i = (*i + 1) % TOTAL_SLIDES);
    };

    view! {
        <section class="section" id="features">
            <div class="container">
                <div class="t-cordel-rule t-display-md t-yellow" style="margin-bottom:var(--sp-12)">
                    "PLATFORM FEATURES"
                </div>

                // Carousel wrapper — holds arrows + track together
                <div style="position:relative; padding:0 var(--sp-8)">

                    // ← arrow
                    <button class="carousel-arrow carousel-arrow-prev" on:click=prev>"‹"</button>

                    // Slides (CSS grid overlap: all share the same cell)
                    <div class="carousel-track">
                        <div
                            class="carousel-slide"
                            class:carousel-slide-active=move || current.get() == 0
                        >
                            <FeatureCoopDashboard />
                        </div>
                        <div
                            class="carousel-slide"
                            class:carousel-slide-active=move || current.get() == 1
                        >
                            <FeatureMyPayments />
                        </div>
                        <div
                            class="carousel-slide"
                            class:carousel-slide-active=move || current.get() == 2
                        >
                            <FeatureMyLoans />
                        </div>
                        <div
                            class="carousel-slide"
                            class:carousel-slide-active=move || current.get() == 3
                        >
                            <FeatureRequestLoan />
                        </div>
                        <div
                            class="carousel-slide"
                            class:carousel-slide-active=move || current.get() == 4
                        >
                            <FeatureOpenMarket />
                        </div>
                        <div
                            class="carousel-slide"
                            class:carousel-slide-active=move || current.get() == 5
                        >
                            <FeatureDonate />
                        </div>
                    </div>

                    // → arrow
                    <button class="carousel-arrow carousel-arrow-next" on:click=next>"›"</button>
                </div>

                // Dot navigation
                <div class="carousel-dots">
                    {(0..TOTAL_SLIDES).map(|i| view! {
                        <button
                            class="carousel-dot"
                            class:carousel-dot-active=move || current.get() == i
                            on:click=move |_| current.set(i)
                        />
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── FEATURE: COOPERATIVE DASHBOARD (slide 0) ──────────────────

fn FeatureCoopDashboard() -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-header">
                <span class="t-display-md t-yellow">"COOPERATIVE DASHBOARD"</span>
                <p class="t-mono-sm" style="color:var(--c-gray-5)">
                    "Each cooperative has its own on-chain contract. View live stats: active members, "
                    "average reputation, total donations in the pool, and outstanding loan balances."
                </p>
            </div>
            <div class="mock-panel">
                // Coop info card
                <div style="background:var(--c-surface-3); border:var(--border-yellow); \
                             border-radius:var(--radius-sm); padding:var(--sp-4); \
                             display:flex; flex-direction:column; gap:var(--sp-3)">
                    <div style="display:flex; justify-content:space-between; align-items:center">
                        <span class="mock-badge">"COOPERATIVE"</span>
                        <span style="color:var(--c-gray-4); font-size:0.62rem">"ts 1782617814"</span>
                    </div>
                    <div style="display:flex; align-items:center; gap:var(--sp-2)">
                        <span style="background:var(--c-green); width:7px; height:7px; \
                                     border-radius:50%; display:inline-block"></span>
                        <span style="color:var(--c-green); font-size:0.65rem; \
                                     font-family:var(--font-display); letter-spacing:0.06em">"ACTIVE"</span>
                    </div>
                    <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:1rem">
                        "COOPERATIVE TEST"
                    </div>
                    <div style="display:flex; flex-direction:column; gap:var(--sp-2)">
                        <span class="mock-label">"LOAN MACHINE"</span>
                        <div style="background:var(--c-surface-2); border:var(--border-thin); \
                                     border-radius:var(--radius-sm); padding:6px 10px; \
                                     display:flex; justify-content:space-between; align-items:center">
                            <span style="color:var(--c-gold); font-size:0.62rem">
                                "\u{25C8} 0xeec3323e\u{2026}780fd"
                            </span>
                            <span style="color:var(--c-gray-4); font-size:0.65rem">"⧉"</span>
                        </div>
                        <span class="mock-label">"COOP ID"</span>
                        <div style="background:var(--c-surface-2); border:var(--border-thin); \
                                     border-radius:var(--radius-sm); padding:6px 10px; \
                                     display:flex; justify-content:space-between; align-items:center">
                            <span style="color:var(--c-gold); font-size:0.62rem">
                                "\u{25C8} 0x3370eda0\u{2026}b4e6c"
                            </span>
                            <span style="color:var(--c-gray-4); font-size:0.65rem">"⧉"</span>
                        </div>
                    </div>
                </div>
                // Money flow card
                <div style="background:var(--c-surface-3); border:var(--border-yellow); \
                             border-radius:var(--radius-sm); padding:var(--sp-4); \
                             display:flex; flex-direction:column; gap:var(--sp-3)">
                    <div style="display:flex; align-items:center; gap:var(--sp-3)">
                        <span class="mock-badge">"MONEY FLOW"</span>
                        <span style="display:flex; align-items:center; gap:4px">
                            <span style="background:var(--c-green); width:6px; height:6px; \
                                         border-radius:50%; display:inline-block"></span>
                            <span style="color:var(--c-green); font-size:0.62rem; \
                                         font-family:var(--font-display)">"ACTIVE"</span>
                        </span>
                    </div>
                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:var(--sp-4)">
                        <div>
                            <div style="color:var(--c-gray-4); font-size:0.62rem; margin-bottom:2px">
                                "ACTIVE MEMBERS"
                            </div>
                            <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:1.4rem">
                                "1"
                            </div>
                        </div>
                        <div>
                            <div style="color:var(--c-gray-4); font-size:0.62rem; margin-bottom:2px">
                                "AVERAGE REPUTATION"
                            </div>
                            <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:1.4rem">
                                "1"
                            </div>
                        </div>
                    </div>
                    <div style="display:grid; grid-template-columns:1fr 1fr; gap:var(--sp-4); \
                                 border-top:var(--border-thin); padding-top:var(--sp-3)">
                        <div>
                            <div style="color:var(--c-gray-4); font-size:0.62rem; margin-bottom:2px">
                                "TOTAL DONATIONS"
                            </div>
                            <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:0.95rem">
                                "1119300 USDT"
                            </div>
                        </div>
                        <div>
                            <div style="color:var(--c-gray-4); font-size:0.62rem; margin-bottom:2px">
                                "OUTSTANDING LOANS"
                            </div>
                            <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:0.95rem">
                                "0 USDT"
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── FEATURE: MY PAYMENTS (slide 1) ───────────────────────────

fn FeatureMyPayments() -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-header">
                <span class="t-display-md t-yellow">"MY PAYMENTS"</span>
                <p class="t-mono-sm" style="color:var(--c-gray-5)">
                    "Manage your active loan repayments. Each parcel has a due date; pay on time "
                    "to build reputation. Late payments are flagged on-chain and affect your credit score."
                </p>
            </div>
            <div class="mock-panel">
                <span class="mock-badge">"MY PAYMENTS"</span>
                <div style="background:var(--c-surface-3); border-radius:var(--radius-sm); \
                             padding:var(--sp-4); display:flex; flex-direction:column; gap:var(--sp-3)">
                    <div style="display:flex; justify-content:space-between; align-items:flex-start">
                        <span style="color:var(--c-gray-4); font-size:0.65rem">
                            "REQ #2 \u{00B7} 0/3 parcels paid"
                        </span>
                        <span style="color:var(--c-green); font-size:0.65rem; \
                                     font-family:var(--font-display)">"ACTIVE"</span>
                    </div>
                    <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:1.1rem">
                        "100 USDT"
                    </div>
                    <div style="display:flex; justify-content:space-between">
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"Repayment progress"</span>
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"0 / 3"</span>
                    </div>
                    <div style="background:var(--c-gray-3); border-radius:9999px; height:4px"></div>
                    <div style="display:flex; flex-direction:column; gap:2px; \
                                 border-top:var(--border-thin); padding-top:var(--sp-2)">
                        <div class="mock-parcel-row">
                            <span>"#1"</span>
                            <span>"2026-07-18"</span>
                            <span>"33.333334 USDT"</span>
                            <span style="color:var(--c-yellow)">"PENDING"</span>
                        </div>
                        <div class="mock-parcel-row">
                            <span>"#2"</span>
                            <span>"2026-08-07"</span>
                            <span>"33.333333 USDT"</span>
                            <span style="color:var(--c-yellow)">"PENDING"</span>
                        </div>
                        <div class="mock-parcel-row">
                            <span>"#3"</span>
                            <span>"2026-08-27"</span>
                            <span>"33.333333 USDT"</span>
                            <span style="color:var(--c-yellow)">"PENDING"</span>
                        </div>
                    </div>
                    <div style="display:flex; justify-content:space-between; align-items:center; \
                                 border-top:var(--border-thin); padding-top:var(--sp-2)">
                        <span style="color:var(--c-gray-5); font-size:0.65rem">
                            "Next payment: "
                            <span style="color:var(--c-yellow)">"33.333333 USDT"</span>
                        </span>
                        <div style="background:var(--c-yellow); color:var(--c-ink); \
                                     border-radius:var(--radius-sm); padding:5px 14px; \
                                     font-family:var(--font-display); font-size:0.75rem; \
                                     font-weight:700; letter-spacing:0.06em">"PAY"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── FEATURE: MY LOANS (slide 2) ──────────────────────────────

fn FeatureMyLoans() -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-header">
                <span class="t-display-md t-yellow">"MY LOANS"</span>
                <p class="t-mono-sm" style="color:var(--c-gray-5)">
                    "Track all your active requisitions. Pending loans wait for member coverage; "
                    "you can cancel before they are fully funded and no funds have been disbursed."
                </p>
            </div>
            <div class="mock-panel">
                <div style="display:grid; grid-template-columns:repeat(4,1fr); gap:4px">
                    <div class="mock-tab">"NEW LOAN"</div>
                    <div class="mock-tab mock-tab-active">"MY LOANS"</div>
                    <div class="mock-tab">"OPEN MARKET"</div>
                    <div class="mock-tab">"MY PAYMENTS"</div>
                </div>
                <span class="mock-badge">"MY LOAN REQUISITIONS"</span>
                <div style="background:var(--c-surface-3); border-radius:var(--radius-sm); \
                             padding:var(--sp-3); display:flex; flex-direction:column; gap:var(--sp-2)">
                    <div style="display:flex; justify-content:space-between">
                        <span style="color:var(--c-gray-4); font-size:0.65rem">
                            "REQ #2 \u{00B7} 3 parcels \u{00B7} 2026-06-28"
                        </span>
                        <span style="color:var(--c-yellow); font-size:0.65rem; \
                                     font-family:var(--font-display)">"PENDING"</span>
                    </div>
                    <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:0.9rem">
                        "100 USDT"
                    </div>
                    <div style="display:flex; justify-content:space-between">
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"Coverage"</span>
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"0% / 100%"</span>
                    </div>
                    <div style="background:var(--c-gray-3); border-radius:9999px; height:4px"></div>
                    <div style="display:flex; justify-content:flex-end">
                        <div class="mock-btn-secondary" style="width:auto; padding:4px 12px">"CANCEL"</div>
                    </div>
                </div>
                <div style="background:var(--c-surface-3); border-radius:var(--radius-sm); \
                             padding:var(--sp-3); display:flex; flex-direction:column; gap:var(--sp-2)">
                    <div style="display:flex; justify-content:space-between">
                        <span style="color:var(--c-gray-4); font-size:0.65rem">
                            "REQ #1 \u{00B7} 10 parcels \u{00B7} 2026-06-28"
                        </span>
                        <span style="color:var(--c-yellow); font-size:0.65rem; \
                                     font-family:var(--font-display)">"PENDING"</span>
                    </div>
                    <div style="color:var(--c-yellow); font-family:var(--font-display); font-size:0.9rem">
                        "1000000 USDT"
                    </div>
                    <div style="display:flex; justify-content:space-between">
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"Coverage"</span>
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"0% / 100%"</span>
                    </div>
                    <div style="background:var(--c-gray-3); border-radius:9999px; height:4px"></div>
                    <div style="display:flex; justify-content:flex-end">
                        <div class="mock-btn-secondary" style="width:auto; padding:4px 12px">"CANCEL"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── FEATURE: REQUEST A LOAN (slide 3) ────────────────────────

fn FeatureRequestLoan() -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-header">
                <span class="t-display-md t-yellow">"REQUEST A LOAN"</span>
                <p class="t-mono-sm" style="color:var(--c-gray-5)">
                    "Submit a loan requisition to the cooperative. Set the total amount, number of "
                    "installments (1\u{2013}12), and payment interval. Members vote with their donations to fund it."
                </p>
            </div>
            <div class="mock-panel">
                <span class="mock-badge">"REQUEST LOAN"</span>
                <div style="display:flex; flex-direction:column; gap:var(--sp-3)">
                    <div style="display:flex; flex-direction:column; gap:6px">
                        <span class="mock-label">"AMOUNT (USDT)"</span>
                        <div class="mock-input">"1000000"</div>
                        <span style="color:var(--c-gray-4); font-size:0.65rem">"How much you need to borrow."</span>
                    </div>
                    <div style="display:flex; flex-direction:column; gap:6px">
                        <span class="mock-label">"NUMBER OF INSTALLMENTS"</span>
                        <div class="mock-input">"10"</div>
                        <span style="color:var(--c-gray-4); font-size:0.65rem">"How many monthly payments (1\u{2013}12)."</span>
                    </div>
                    <div style="display:flex; flex-direction:column; gap:6px">
                        <span class="mock-label">"PAYMENT INTERVAL (DAYS)"</span>
                        <div class="mock-input">"30"</div>
                        <span style="color:var(--c-gray-4); font-size:0.65rem">"Days between each payment (1\u{2013}30)."</span>
                    </div>
                    <div class="mock-btn-primary">"REQUEST LOAN"</div>
                </div>
            </div>
        </div>
    }
}

// ── FEATURE: OPEN MARKET (slide 4) ───────────────────────────

fn FeatureOpenMarket() -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-header">
                <span class="t-display-md t-yellow">"OPEN MARKET"</span>
                <p class="t-mono-sm" style="color:var(--c-gray-5)">
                    "Browse pending loan requisitions and allocate part of your donated balance to cover them. "
                    "Choose any percentage — your contribution earns reputation proportional to coverage."
                </p>
            </div>
            <div class="mock-panel">
                <div style="background:var(--c-surface-3); border-radius:var(--radius-sm); \
                             padding:var(--sp-4); display:flex; flex-direction:column; gap:var(--sp-3)">
                    <div style="display:flex; justify-content:space-between; align-items:flex-start">
                        <div>
                            <span style="color:var(--c-gray-4); font-size:0.65rem">
                                "REQ #1 \u{00B7} 10 parcels \u{00B7} 2026-06-28"
                            </span>
                            <div style="color:var(--c-yellow); font-family:var(--font-display); \
                                         font-size:1.1rem; margin-top:2px">"1000000 USDT"</div>
                            <div style="color:var(--c-gray-4); font-size:0.62rem; margin-top:2px">
                                "0x8fb852\u{2026}887d"
                            </div>
                        </div>
                        <span style="color:var(--c-yellow); font-size:0.65rem; \
                                     font-family:var(--font-display); letter-spacing:0.06em">"PENDING"</span>
                    </div>
                    <div style="display:flex; justify-content:space-between">
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"Coverage"</span>
                        <span style="color:var(--c-gray-5); font-size:0.65rem">"0% / 100%"</span>
                    </div>
                    <div style="background:var(--c-gray-3); border-radius:9999px; height:5px"></div>
                    <div style="background:var(--c-surface-2); border-radius:var(--radius-sm); \
                                 padding:var(--sp-3); display:flex; flex-direction:column; gap:var(--sp-2)">
                        <div style="display:flex; justify-content:space-between; align-items:center">
                            <span style="color:var(--c-gray-5); font-size:0.65rem">"Your contribution"</span>
                            <span style="color:var(--c-yellow); font-family:var(--font-display); font-size:1rem">
                                "13%"
                            </span>
                        </div>
                        <div style="background:var(--c-gray-3); border-radius:9999px; \
                                     height:5px; position:relative; overflow:visible">
                            <div style="background:var(--c-yellow); height:5px; width:100%; \
                                         border-radius:9999px; position:relative">
                                <div style="position:absolute; right:-5px; top:-3px; \
                                             width:11px; height:11px; border-radius:50%; \
                                             background:var(--c-yellow)"></div>
                            </div>
                        </div>
                        <div style="display:flex; justify-content:space-between">
                            <span style="color:var(--c-gray-4); font-size:0.6rem">"1%"</span>
                            <span style="color:var(--c-gray-4); font-size:0.6rem">"max 13% of 100% remaining"</span>
                        </div>
                        <div class="mock-btn-secondary">"COVER 13%"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── FEATURE: DONATE & WITHDRAW (slide 5) ─────────────────────

fn FeatureDonate() -> impl IntoView {
    view! {
        <div class="feature-card">
            <div class="feature-header">
                <span class="t-display-md t-yellow">"DONATE & WITHDRAW"</span>
                <p class="t-mono-sm" style="color:var(--c-gray-5)">
                    "Contribute USDT to the cooperative pool. Your donation earns reputation "
                    "and can be allocated to cover member loans. Withdraw anytime your balance is free."
                </p>
            </div>
            <div class="mock-panel">
                <div style="display:grid; grid-template-columns:1fr 1fr; gap:4px; \
                             background:var(--c-surface-3); border-radius:var(--radius-sm); padding:4px">
                    <div style="background:var(--c-yellow); color:var(--c-ink); \
                                border-radius:4px; padding:8px; text-align:center; \
                                font-family:var(--font-display); font-size:0.75rem; \
                                letter-spacing:0.06em">"DONATE"</div>
                    <div style="color:var(--c-gray-4); text-align:center; padding:8px; \
                                font-family:var(--font-display); font-size:0.75rem; \
                                letter-spacing:0.06em">"WITHDRAW"</div>
                </div>
                <div style="background:var(--c-surface-3); border-radius:var(--radius-sm); \
                             padding:var(--sp-4); display:flex; flex-direction:column; gap:var(--sp-4)">
                    <span class="mock-badge">"DONATE"</span>
                    <div style="display:flex; flex-direction:column; gap:var(--sp-2)">
                        <span class="mock-label">"AMOUNT (USDT)"</span>
                        <div class="mock-input">"0.000001 \u{2013} 1,000,000"</div>
                        <span style="color:var(--c-gray-4); font-size:0.65rem">
                            "Two signatures are required: approve, then donate."
                        </span>
                    </div>
                    <div class="mock-btn-primary">"DONATE"</div>
                </div>
            </div>
        </div>
    }
}

// ── CTA ───────────────────────────────────────────────────────

#[component]
pub(crate) fn CtaSection(#[prop(into)] on_login: Callback<()>) -> impl IntoView {
    view! {
        <section class="section">
            <div class="container-sm t-center">
                <div class="flex-col flex-center gap-6">
                    <h2 class="t-display-lg t-bright">
                        "READY TO"
                        <br/>
                        <span class="t-yellow">"JOIN IN?"</span>
                    </h2>
                    <p class="t-mono-md t-muted">
                        "You will need your cooperative's access code to link up."
                    </p>
                    <button
                        class="btn btn-primary btn-lg"
                        on:click=move |_| on_login.run(())
                    >
                        "GET STARTED"
                    </button>
                </div>
            </div>
        </section>
    }
}

// ── FOOTER ────────────────────────────────────────────────────

#[component]
pub(crate) fn HomeFooter() -> impl IntoView {
    view! {
        <footer style="border-top:1px solid var(--c-gray-2); padding:var(--sp-8) 0">
            <div class="container flex-between">
                <span class="t-display-md t-yellow" style="opacity:0.6">"LOAN MACHINE"</span>
                <span class="t-mono-xs t-muted">"Immutable contracts. Real community."</span>
            </div>
        </footer>
    }
}
