// src/components/home.rs
use leptos::prelude::*;
use crate::components::ui::*;


#[component]
pub fn HomePage(#[prop(into)] on_login: Callback<()>) -> impl IntoView {
    view! {
        <HeroSection on_login=on_login />
        <HowItWorksSection />
        <TransparencySection />
        <CtaSection on_login=on_login />
        <HomeFooter />
    }
}


#[component]
pub(crate) fn HeroSection(#[prop(into)] on_login: Callback<()>) -> impl IntoView {
    view! {
        <section class="hero">
            <div class="container">
                <div style="max-width: 680px; display:flex; flex-direction:column; gap: var(--sp-8)">

                    <div>
                        <Badge color=BadgeColor::Yellow live=true>
                            "Active Community"
                        </Badge>
                    </div>

                    <h1 class="t-display-xl t-bright" style="line-height:0.92">
                        "CREDIT"
                        <br/>
                        <span class="t-yellow">"SOLIDARITY"</span>
                        <br/>
                        "ON THE BLOCKCHAIN"
                    </h1>

                    <p class="t-mono-md" style="color: var(--c-gray-5); max-width: 480px; line-height:1.8">
                        "A collective and transparent lending machine. "
                        "Every transaction recorded immutably. "
                        "Governed by its own members."
                    </p>

                    <div class="flex gap-4">
                        <button
                            class="btn btn-primary btn-lg"
                            on:click=move |_| on_login.run(())
                        >
                            "JOIN THE COOPERATIVE"
                        </button>
                        <a href="#how-it-works" class="btn btn-ghost btn-lg">
                            "HOW IT WORKS"
                        </a>
                    </div>

                    // Stats row
                    <div class="flex gap-8 mt-4" style="border-top: 1px solid var(--c-gray-2); padding-top: var(--sp-8)">
                        <StatBlock label="Active Members" value="0+".to_string() />
                        <StatBlock label="Total Donated"  value="$ 0".to_string() />
                        <StatBlock label="Loans"          value="0".to_string() />
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
pub(crate) fn HowItWorksSection() -> impl IntoView {
    view! {
        <section class="section" id="how-it-works">
            <div class="container">
                <div class="t-cordel-rule t-display-md t-yellow mb-12">
                    "HOW IT WORKS"
                </div>

                <div class="grid-3">
                    <Card variant=CardVariant::Default hover=true>
                        <div class="flex-col gap-4">
                            <span class="t-display-xl t-yellow" style="opacity:0.3">"01"</span>
                            <h3 class="t-display-md t-bright">"JOIN THE COOP"</h3>
                            <p class="t-mono-sm t-muted">
                                "Log in with your email. Your digital wallet is created automatically — "
                                "no seed phrase, no technical complexity."
                            </p>
                        </div>
                    </Card>

                    <Card variant=CardVariant::Yellow hover=true>
                        <div class="flex-col gap-4">
                            <span class="t-display-xl t-yellow" style="opacity:0.3">"02"</span>
                            <h3 class="t-display-md t-bright">"CONTRIBUTE"</h3>
                            <p class="t-mono-sm t-bright" >
                                "Make a donation in USDT to the collective fund. "
                                "Each contribution increases your reputation in the network."
                            </p>
                        </div>
                    </Card>

                    <Card variant=CardVariant::Default hover=true>
                        <div class="flex-col gap-4">
                            <span class="t-display-xl t-yellow" style="opacity:0.3">"03"</span>
                            <h3 class="t-display-md t-bright">"ACCESS CREDIT"</h3>
                            <p class="t-mono-sm t-muted">
                                "Request loans covered by your cooperative's members. "
                                "Terms decided collectively."
                            </p>
                        </div>
                    </Card>
                </div>
            </div>
        </section>
    }
}


#[component]
pub(crate) fn TransparencySection() -> impl IntoView {
    view! {
        <section class="section-sm" style="background: var(--c-surface); border-top: 1px solid var(--c-gray-2); border-bottom: 1px solid var(--c-gray-2)">
            <div class="container">
                <div class="grid-4">
                    <div class="stat-block t-center">
                        <span class="stat-label">"On-Chain Contracts"</span>
                        <span class="stat-value">"100%"</span>
                    </div>
                    <div class="stat-block t-center">
                        <span class="stat-label">"Auditable Code"</span>
                        <span class="stat-value">"✓"</span>
                    </div>
                    <div class="stat-block t-center">
                        <span class="stat-label">"No Central Custody"</span>
                        <span class="stat-value">"✓"</span>
                    </div>
                    <div class="stat-block t-center">
                        <span class="stat-label">"Member Governance"</span>
                        <span class="stat-value">"✓"</span>
                    </div>
                </div>
            </div>
        </section>
    }
}
#[component]
pub(crate) fn CtaSection(#[prop(into)] on_login: Callback<()>) -> impl IntoView{
    view!{
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

#[component]
pub(crate) fn HomeFooter() -> impl IntoView {
    view!{
        <footer style="border-top: 1px solid var(--c-gray-2); padding: var(--sp-8) 0">
            <div class="container flex-between">
                <span class="t-display-md t-yellow" style="opacity:0.6">"LOAN MACHINE"</span>
                <span class="t-mono-xs t-muted">"Immutable contracts. Real community."</span>
            </div>
        </footer>
    }
}



