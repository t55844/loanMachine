// loan_machine_web/src/components/coop_choice.rs

use leptos::prelude::*;
use crate::components::ui::*;

#[component]
pub fn CoopChoicePage() -> impl IntoView {
    view! {
        <Section>
            <div class="container-sm">
                <div class="t-center" style="margin-bottom: var(--sp-8)">
                    <SectionTitle>"WHAT DO YOU WANT TO DO?"</SectionTitle>
                    <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                        "Wallet connected. Choose an option to continue."
                    </p>
                </div>
                <div class="grid-2 gap-6">
                    <a href="/vinculation" style="text-decoration: none">
                        <Card hover=true>
                            <span class="card-tag">"MEMBER"</span>
                            <h2
                                class="t-display-md"
                                style="margin-top: var(--sp-4); color: var(--c-paper)"
                            >
                                "JOIN A "
                                <span class="t-yellow">"COOPERATIVE"</span>
                            </h2>
                            <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                                "Have an access code? Link your wallet and join an
                                existing cooperative."
                            </p>
                            <div style="margin-top: var(--sp-6)">
                                <Button variant=BtnVariant::Ghost full_width=true>
                                    "LINK WALLET →"
                                </Button>
                            </div>
                        </Card>
                    </a>
                    <a href="/create-coop" style="text-decoration: none">
                        <Card variant=CardVariant::Yellow hover=true>
                            <span class="card-tag">"FOUNDER"</span>
                            <h2
                                class="t-display-md"
                                style="margin-top: var(--sp-4); color: var(--c-paper)"
                            >
                                "CREATE A NEW "
                                <span class="t-yellow">"COOPERATIVE"</span>
                            </h2>
                            <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                                "Found a cooperative on-chain. You will be the main administrator
                                and will receive the exclusive access code."
                            </p>
                            <div style="margin-top: var(--sp-6)">
                                <Button variant=BtnVariant::Primary full_width=true>
                                    "CREATE COOPERATIVE →"
                                </Button>
                            </div>
                        </Card>
                    </a>
                </div>
            </div>
        </Section>
    }
}