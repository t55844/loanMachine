use leptos::prelude::*;
use crate::components::ui::*;

#[component]
pub fn PendingRequisitions(#[prop(into)] coop_id: String) -> impl IntoView {
    let _ = coop_id;
    view! {
        <Card hover=false tag="OPEN MARKET">
            <p class="t-mono-sm t-muted" style="padding: var(--sp-8) 0; text-align: center">
                "Open market view coming soon."
            </p>
        </Card>
    }
}
