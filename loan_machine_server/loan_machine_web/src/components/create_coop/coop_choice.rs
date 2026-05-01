// loan_machine_web/src/components/coop_choice.rs

use leptos::prelude::*;
use crate::components::ui::*;

#[component]
pub fn CoopChoicePage() -> impl IntoView {
    view! {
        <Section>
            <div class="container-sm">
                <div class="t-center" style="margin-bottom: var(--sp-8)">
                    <SectionTitle>"O QUE DESEJA FAZER?"</SectionTitle>
                    <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                        "Carteira conectada. Escolha uma opção para continuar."
                    </p>
                </div>
                <div class="grid-2 gap-6">
                    <a href="/vinculate" style="text-decoration: none">
                        <Card hover=true>
                            <span class="card-tag">"MEMBRO"</span>
                            <h2
                                class="t-display-md"
                                style="margin-top: var(--sp-4); color: var(--c-paper)"
                            >
                                "ENTRAR EM UMA "
                                <span class="t-yellow">"COOPERATIVA"</span>
                            </h2>
                            <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                                "Possui um código de acesso? Vincule seu CPF/CNPJ e
                                participe de uma cooperativa já existente."
                            </p>
                            <div style="margin-top: var(--sp-6)">
                                <Button variant=BtnVariant::Ghost full_width=true>
                                    "VINCULAR DOCUMENTO →"
                                </Button>
                            </div>
                        </Card>
                    </a>
                    <a href="/create-coop" style="text-decoration: none">
                        <Card variant=CardVariant::Yellow hover=true>
                            <span class="card-tag">"FUNDADOR"</span>
                            <h2
                                class="t-display-md"
                                style="margin-top: var(--sp-4); color: var(--c-paper)"
                            >
                                "CRIAR UMA NOVA "
                                <span class="t-yellow">"COOPERATIVA"</span>
                            </h2>
                            <p class="t-mono-sm t-muted" style="margin-top: var(--sp-4)">
                                "Funde uma cooperativa on-chain. Você será o administrador
                                principal e receberá o código de acesso exclusivo."
                            </p>
                            <div style="margin-top: var(--sp-6)">
                                <Button variant=BtnVariant::Primary full_width=true>
                                    "CRIAR COOPERATIVA →"
                                </Button>
                            </div>
                        </Card>
                    </a>
                </div>
            </div>
        </Section>
    }
}