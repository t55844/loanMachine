// src/components/ui.rs
// CordelWave Crypto — Reusable Leptos Component Library
// Usage: use crate::components::ui::*;

use leptos::prelude::*;

// ── LAYOUT ──────────────────────────────────────────────────

/// Page wrapper with navbar + content
#[component]
pub fn PageLayout(
    #[prop(optional)] title: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div>
            <Navbar title=title.unwrap_or("LOAN MACHINE") />
            <main>
                {children()}
            </main>
        </div>
    }
}

/// Centered container
#[component]
pub fn Container(
    #[prop(optional, default="container")] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! { <div class=class>{children()}</div> }
}

/// Section with top/bottom padding
#[component]
pub fn Section(
    #[prop(optional)] yellow_bg: bool,
    #[prop(optional)] small: bool,
    children: Children,
) -> impl IntoView {
    let class = match (yellow_bg, small) {
        (true,  true)  => "section-sm",
        (true,  false) => "section",
        (false, true)  => "section-sm",
        (false, false) => "section",
    };
    let style = if yellow_bg {
        "background: var(--c-yellow); color: var(--c-ink);"
    } else {
        ""
    };
    view! {
        <section class=class style=style>
            <div class="container">{children()}</div>
        </section>
    }
}

// ── NAVIGATION ──────────────────────────────────────────────

#[component]
pub fn Navbar(
    #[prop(optional, default="LOAN MACHINE")] title: &'static str,
) -> impl IntoView {
    view! {
        <nav class="navbar">
            <div class="container navbar-inner">
                <a href="/" class="navbar-brand">{title}</a>
                <ul class="navbar-links">
                    <li><a href="/" class="navbar-link">"Home"</a></li>
                    <li><a href="/vinculate" class="navbar-link">"Vinculate"</a></li>
                    <li><a href="/donate" class="navbar-link">"Donate"</a></li>
                    <li>
                        <span class="badge badge-live badge-yellow">"Live"</span>
                    </li>
                </ul>
            </div>
        </nav>
    }
}

// ── TYPOGRAPHY ──────────────────────────────────────────────

/// Big display heading (Bebas Neue)
#[component]
pub fn DisplayHeading(
    #[prop(optional, default="t-display-xl")] size: &'static str,
    #[prop(optional)] yellow: bool,
    #[prop(optional)] center: bool,
    children: Children,
) -> impl IntoView {
    let mut class = size.to_string();
    if yellow  { class.push_str(" t-yellow"); }
    if center  { class.push_str(" t-center"); }
    view! { <h1 class=class>{children()}</h1> }
}

/// Ornamental section title with dashed rules
#[component]
pub fn SectionTitle(children: Children) -> impl IntoView {
    view! {
        <div class="t-cordel-rule t-display-md t-yellow" style="margin-bottom: var(--sp-8)">
            {children()}
        </div>
    }
}

/// Mono-spaced data label
#[component]
pub fn DataLabel(children: Children) -> impl IntoView {
    view! { <span class="stat-label">{children()}</span> }
}

// ── BUTTONS ─────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum BtnVariant { Primary, Secondary, Ghost, Danger }

#[derive(Clone, PartialEq)]
pub enum BtnSize { Sm, Md, Lg }

/// Universal button
#[component]
pub fn Button(
    #[prop(optional, default=BtnVariant::Primary)] variant: BtnVariant,
    #[prop(optional, default=BtnSize::Md)]         size: BtnSize,
    #[prop(optional)]                              full_width: bool,
    #[prop(optional)]                              loading: bool,
    #[prop(optional)]                              disabled: bool,
    #[prop(optional)]                              on_click: Option<Box<dyn Fn() + 'static>>,
    children: Children,
) -> impl IntoView {
    let variant_class = match variant {
        BtnVariant::Primary   => "btn btn-primary",
        BtnVariant::Secondary => "btn btn-secondary",
        BtnVariant::Ghost     => "btn btn-ghost",
        BtnVariant::Danger    => "btn btn-danger",
    };
    let size_class = match size {
        BtnSize::Sm => " btn-sm",
        BtnSize::Md => "",
        BtnSize::Lg => " btn-lg",
    };
    let full_class = if full_width { " btn-full" } else { "" };
    let class = format!("{}{}{}", variant_class, size_class, full_class);

    let handle_click = move |_| {
        if let Some(f) = &on_click { f(); }
    };

    view! {
        <button
            class=class
            disabled=disabled || loading
            on:click=handle_click
        >
            {if loading {
                view! { <span class="spinner"></span> }.into_any()
            } else {
                view! { {children()} }.into_any()
            }}
        </button>
    }
}

// ── CARDS ────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum CardVariant { Default, Yellow, Gold }

/// Card container
#[component]
pub fn Card(
    #[prop(optional, default=CardVariant::Default)] variant: CardVariant,
    #[prop(optional)]                               tag: Option<&'static str>,
    #[prop(optional)]                               hover: bool,
    children: Children,
) -> impl IntoView {
    let card_class = match variant {
        CardVariant::Default => "card",
        CardVariant::Yellow  => "card card-yellow",
        CardVariant::Gold    => "card card-gold",
    };
    
    // ← Only disable the transition/transform, NEVER pointer-events
    let style = if !hover { "transition: none; transform: none;" } else { "" };

    view! {
        <div class=card_class style=style>
            {tag.map(|t| view! { <span class="card-tag">{t}</span> })}
            {children()}
        </div>
    }
}

// ── FORMS ─────────────────────────────────────────────────── 

/// Labeled text input
#[component]
pub fn TextInput(
    label: &'static str,
    #[prop(optional, default="")] placeholder: &'static str,
    #[prop(optional)]             hint: Option<&'static str>,
    #[prop(optional)]             error: Option<String>,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
) -> impl IntoView {
    let input_class = if error.is_some() {
        "form-input"
    } else {
        "form-input"
    };
    let border_style = if error.is_some() {
        "border-color: var(--c-red);"
    } else {
        ""
    };

    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <input
                class=input_class
                style=border_style
                type="text"
                placeholder=placeholder
                prop:value=value
                on:input=move |ev| set_value.set(event_target_value(&ev))
            />
            {hint.map(|h| view! { <span class="form-hint">{h}</span> })}
            {error.map(|e| view! { <span class="form-error">"⚠ "{e}</span> })}
        </div>
    }
}

/// Wallet address input (special gold styling)
#[component]
pub fn AddressInput(
    label: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    #[prop(optional)] error: Option<String>,
) -> impl IntoView {
    view! {
        <div class="form-group">
            <label class="form-label">
                <span style="color: var(--c-gold)">{"◈ "}</span>
                {label}
            </label>
            <input
                class="form-input form-input-address"
                type="text"
                placeholder="0x0000...0000"
                prop:value=value
                on:input=move |ev| set_value.set(event_target_value(&ev))
            />
            <span class="form-hint">"Enter full EVM-compatible wallet address"</span>
            {error.map(|e| view! { <span class="form-error">"⚠ "{e}</span> })}
        </div>
    }
}

/// Number input (member ID etc.)
#[component]
pub fn NumberInput(
    label: &'static str,
    value: ReadSignal<u32>,
    set_value: WriteSignal<u32>,
    #[prop(optional)] hint: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="form-group">
            <label class="form-label">{label}</label>
            <input
                class="form-input"
                type="number"
                min="0"
                prop:value=move || value.get().to_string()
                on:input=move |ev| {
                    let val = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                    set_value.set(val);
                }
            />
            {hint.map(|h| view! { <span class="form-hint">{h}</span> })}
        </div>
    }
}

// ── STATS / DATA ─────────────────────────────────────────────

/// Big stat number display
#[component]
pub fn StatBlock(
    label: &'static str,
    value: String,
    #[prop(optional)] delta: Option<String>,
    #[prop(optional)] delta_up: bool,
) -> impl IntoView {
    view! {
        <div class="stat-block">
            <span class="stat-label">{label}</span>
            <span class="stat-value">{value}</span>
            {delta.map(|d| {
                let class = if delta_up { "stat-delta-up" } else { "stat-delta-down" };
                let prefix = if delta_up { "▲ " } else { "▼ " };
                view! { <span class=class>{prefix}{d}</span> }
            })}
        </div>
    }
}

/// Monospaced address/hash display with copy button
#[component]
pub fn HashDisplay(value: String) -> impl IntoView {
    // Truncate long hashes for display
    let display = if value.len() > 20 {
        format!("{}...{}", &value[..10], &value[value.len()-8..])
    } else {
        value.clone()
    };

    view! {
        <div class="hash-display" title=value>
            <span style="color: var(--c-gold)">{"◈ "}</span>
            {display}
        </div>
    }
}

// ── BADGES ───────────────────────────────────────────────────

#[derive(Clone)]
pub enum BadgeColor { Yellow, Green, Gold, Red }

#[component]
pub fn Badge(
    color: BadgeColor,
    #[prop(optional)] filled: bool,
    #[prop(optional)] live: bool,
    children: Children,
) -> impl IntoView {
    let class = match (color, filled) {
        (BadgeColor::Yellow, true)  => "badge badge-filled-yellow",
        (BadgeColor::Yellow, false) => "badge badge-yellow",
        (BadgeColor::Green,  true)  => "badge badge-filled-green",
        (BadgeColor::Green,  false) => "badge badge-green",
        (BadgeColor::Gold,   _)     => "badge badge-gold",
        (BadgeColor::Red,    _)     => "badge badge-red",
    };
    let live_class = if live { " badge-live" } else { "" };
    let full_class = format!("{}{}", class, live_class);

    view! { <span class=full_class>{children()}</span> }
}

// ── ALERTS ───────────────────────────────────────────────────

#[derive(Clone)]
pub enum AlertKind { Success, Warning, Error, Info }

#[component]
pub fn Alert(kind: AlertKind, children: Children) -> impl IntoView {
    let (class, icon) = match kind {
        AlertKind::Success => ("alert alert-success", "✓ "),
        AlertKind::Warning => ("alert alert-warning", "⚠ "),
        AlertKind::Error   => ("alert alert-error",   "✕ "),
        AlertKind::Info    => ("alert alert-info",    "◈ "),
    };
    view! {
        <div class=class>
            <span style="font-weight:700">{icon}</span>
            {children()}
        </div>
    }
}

// ── TX RESULT CARD ────────────────────────────────────────────

/// Displays a blockchain transaction response
#[component]
pub fn TxCard(
    to: String,
    data: String,
    gas_estimate: String,
) -> impl IntoView {
    view! {
        <div class="card card-gold" style="margin-top: var(--sp-6)">
            <span class="card-tag">"TX READY"</span>
            <div style="margin-top: var(--sp-4); display: flex; flex-direction: column; gap: var(--sp-4)">
                <div class="stat-block">
                    <span class="stat-label">"Contract Address"</span>
                    <HashDisplay value=to />
                </div>
                <div class="stat-block">
                    <span class="stat-label">"Gas Estimate"</span>
                    <span class="stat-value-sm">{gas_estimate}" gas"</span>
                </div>
                <div class="stat-block">
                    <span class="stat-label">"Calldata"</span>
                    <div class="hash-display" style="font-size: 0.7rem; word-break: break-all">
                        {data}
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── CORDEL DIVIDER ────────────────────────────────────────────

#[component]
pub fn Divider(
    #[prop(optional)] symbol: Option<&'static str>,
) -> impl IntoView {
    let s = symbol.unwrap_or("◈");
    view! {
        <div class="divider-cordel">{s}</div>
    }
}

// ── TICKER BAR ────────────────────────────────────────────────

/// Scrolling price ticker — pass static items as (symbol, price, change)
#[component]
pub fn Ticker(
    items: Vec<(&'static str, &'static str, &'static str, bool)>,
) -> impl IntoView {
    let items_clone = items.clone();
    view! {
        <div class="ticker">
            <div class="ticker-track">
                // Duplicate for seamless loop
                {items.iter().chain(items_clone.iter()).map(|(sym, price, change, up)| {
                    let color = if *up { "var(--c-ink)" } else { "var(--c-red)" };
                    let arrow = if *up { "▲" } else { "▼" };
                    view! {
                        <span class="ticker-item">
                            <strong>{*sym}</strong>
                            {" "}{*price}
                            <span style=format!("color: {}", color)>{arrow}{" "}{*change}</span>
                            <span style="opacity:0.4; margin: 0 8px">{" | "}</span>
                        </span>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
