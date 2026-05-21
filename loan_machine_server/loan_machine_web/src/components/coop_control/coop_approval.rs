// loan_machine_web/src/components/coop_approval/request_approval.rs

use leptos::prelude::*;

use crate::components::ui::*;
use crate::components::gas_modal::{use_gas_modal, GasEstimate, GasModalRequest};
use crate::server_fns::coop_approval::prepare_request_approval;
use crate::wallet_auth::privy_bridge::{self, TxOutcome};

#[component]
pub fn RequestApproval(
    #[prop(into)] coop_id: String,
    on_tx_success: Callback<()>,
) -> impl IntoView {
    let (error, set_error) = signal(String::new());

    let request = Action::new(move |coop_id: &String| {
        let id = coop_id.clone();
        async move { prepare_request_approval(id).await }
    });
    let loading = request.pending();

    let gas_modal = use_gas_modal();
    Effect::new(move |_| {
        let Some(Ok(b)) = request.value().get() else { return };
        let to   = b.to.clone();
        let data = b.data.clone();
        let gas  = b.gas_hex.clone();
        gas_modal.set(Some(GasModalRequest {
            title: "CONFIRMAR PEDIDO DE APROVAÇÃO".into(),
            estimates: vec![GasEstimate {
                label:   "Propor aprovação da carteira".into(),
                gas_hex: gas.clone(),
            }],
            on_confirm: Callback::new(move |_| {
                privy_bridge::send_tx(&to, &data, Some(&gas));
            }),
        }));
    });

    Effect::new(move |_| {
        if let Some(Err(e)) = request.value().get() {
            set_error.set(e.to_string());
        }
    });

    privy_bridge::on_tx_outcome(move |outcome| match outcome {
        TxOutcome::Complete(_) => {
            set_error.set(String::new());
            on_tx_success.run(());
        }
        TxOutcome::Failed(err) => set_error.set(format!("Falha na transação: {err}")),
    });

    view! {
        <Card variant=CardVariant::Yellow tag="VISITANTE — PEDIDO DE APROVAÇÃO" hover=false>
            <div class="flex-col gap-6" style="margin-top: var(--sp-4)">

                <p class="t-mono-sm">
                    "Sua carteira ainda não pertence a esta cooperativa. Para entrar,
                    é preciso passar por um processo de aprovação coletivo."
                </p>

                <div class="flex-col gap-3" style="padding: var(--sp-4) 0">
                    <span class="t-mono-xs t-muted">"COMO FUNCIONA"</span>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"1"</Badge>
                        <p class="t-mono-xs">
                            "Você inicia um pedido de aprovação aqui. Isso registra
                            sua carteira como candidata, mas ainda não a torna membro."
                        </p>
                    </div>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"2"</Badge>
                        <p class="t-mono-xs">
                            "Os administradores da cooperativa analisam o pedido e
                            registram suas confirmações. É preciso atingir o limiar
                            de assinaturas definido pela cooperativa."
                        </p>
                    </div>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"3"</Badge>
                        <p class="t-mono-xs">
                            "Um moderador eleito co-assina o pedido — esta etapa
                            garante que a aprovação não dependa apenas dos admins."
                        </p>
                    </div>

                    <div style="display: flex; gap: var(--sp-3); align-items: flex-start">
                        <Badge color=BadgeColor::Gold>"4"</Badge>
                        <p class="t-mono-xs">
                            "Com confirmações + co-assinatura, sua carteira é aprovada.
                            Você poderá então vincular seu CPF/CNPJ à carteira e
                            tornar-se membro de fato."
                        </p>
                    </div>
                </div>

                <Alert kind=AlertKind::Info>
                    "Após enviar, esta tela mostrará o andamento do pedido
                    (confirmações de admin, co-assinatura do moderador) até a aprovação."
                </Alert>

                {move || {
                    let e = error.get();
                    (!e.is_empty()).then(|| view! {
                        <Alert kind=AlertKind::Error>{e}</Alert>
                    })
                }}

                <Button
                    variant=BtnVariant::Primary
                    full_width=true
                    loading=Signal::derive(move || loading.get())
                    on_click=Box::new(move || {
                        set_error.set(String::new());
                        request.dispatch(coop_id.clone());
                    })
                >
                    "INICIAR PEDIDO DE APROVAÇÃO"
                </Button>
            </div>
        </Card>
    }
}