use leptos::prelude::*;

use crate::components::loan_requisition::loan_requisition::LoanRequisitionForm;
use crate::components::loan_requisition::my_requisitions::MyRequisitions;
use crate::components::loan_requisition::pending_requisitions::PendingRequisitions;
use crate::components::loan_requisition::my_payments::MyPayments;
use crate::components::gas_modal::use_gas_modal;

#[derive(Clone, Copy, PartialEq)]
enum LoanTab {
    NewLoan,
    MyRequisitions,
    OpenMarket,
    MyPayments,
}

#[component]
pub fn LoanPanel(
    #[prop(into)] coop_id: String,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (tab, set_tab) = signal(LoanTab::NewLoan);
    let set_gas_modal  = use_gas_modal();

    let tab_class = move |t: LoanTab| move || {
        if tab.get() == t { "tab-btn active" } else { "tab-btn" }
    };

    let switch = move |t: LoanTab| {
        set_gas_modal.set(None);
        set_tab.set(t);
    };

    let coop_new    = coop_id.clone();
    let coop_my     = coop_id.clone();
    let coop_market = coop_id.clone();
    let coop_pay    = coop_id.clone();

    view! {
        <div class="tab-bar">
            <button class=tab_class(LoanTab::NewLoan)
                on:click=move |_| switch(LoanTab::NewLoan)>
                "NEW LOAN"
            </button>
            <button class=tab_class(LoanTab::MyRequisitions)
                on:click=move |_| switch(LoanTab::MyRequisitions)>
                "MY LOANS"
            </button>
            <button class=tab_class(LoanTab::OpenMarket)
                on:click=move |_| switch(LoanTab::OpenMarket)>
                "OPEN MARKET"
            </button>
            <button class=tab_class(LoanTab::MyPayments)
                on:click=move |_| switch(LoanTab::MyPayments)>
                "MY PAYMENTS"
            </button>
        </div>

        {move || match tab.get() {
            LoanTab::NewLoan => view! {
                <LoanRequisitionForm
                    coop_id=coop_new.clone()
                    on_tx_success=move || on_tx_success.run(())
                />
            }.into_any(),
            LoanTab::MyRequisitions => view! {
                <MyRequisitions coop_id=coop_my.clone() />
            }.into_any(),
            LoanTab::OpenMarket => view! {
                <PendingRequisitions coop_id=coop_market.clone() />
            }.into_any(),
            LoanTab::MyPayments => view! {
                <MyPayments coop_id=coop_pay.clone() />
            }.into_any(),
        }}
    }
}
