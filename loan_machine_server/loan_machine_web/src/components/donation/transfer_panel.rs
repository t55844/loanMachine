// src/components/donation/transfer_panel.rs
//
// Tab switcher between `<DonationForm />` and `<WithdrawalForm />`.

use leptos::prelude::*;

use crate::components::donation::donation::DonationForm;
use crate::components::donation::withdrawal::WithdrawalForm;
use crate::components::gas_modal::use_gas_modal;

#[derive(Clone, Copy, PartialEq)]
enum TransferTab {
    Donate,
    Withdraw,
}

#[component]
pub fn TransferPanel(
    #[prop(into)] coop_id: String,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (tab, set_tab) = signal(TransferTab::Donate);
    let set_gas_modal  = use_gas_modal();

    let donate_class = move || match tab.get() {
        TransferTab::Donate => "tab-btn active",
        _ => "tab-btn",
    };
    let withdraw_class = move || match tab.get() {
        TransferTab::Withdraw => "tab-btn active",
        _ => "tab-btn",
    };

    let coop_id_donate   = coop_id.clone();
    let coop_id_withdraw = coop_id.clone();

    view! {
        <div class="tab-bar">
            <button class=donate_class on:click=move |_| {
                set_gas_modal.set(None);
                set_tab.set(TransferTab::Donate);
            }>
                "DONATE"
            </button>
            <button class=withdraw_class on:click=move |_| {
                set_gas_modal.set(None);
                set_tab.set(TransferTab::Withdraw);
            }>
                "WITHDRAW"
            </button>
        </div>

        {move || match tab.get() {
            TransferTab::Donate => view! {
                <DonationForm coop_id=coop_id_donate.clone() on_tx_success=move || on_tx_success.run(()) />
            }.into_any(),
            TransferTab::Withdraw => view! {
                <WithdrawalForm coop_id=coop_id_withdraw.clone() on_tx_success=move || on_tx_success.run(()) />
            }.into_any(),
        }}
    }
}
