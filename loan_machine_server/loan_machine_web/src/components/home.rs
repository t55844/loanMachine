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
                            "Comunidade Ativa"
                        </Badge>
                    </div>

                    <h1 class="t-display-xl t-bright" style="line-height:0.92">
                        "CRÉDITO"
                        <br/>
                        <span class="t-yellow">"SOLIDÁRIO"</span>
                        <br/>
                        "NA BLOCKCHAIN"
                    </h1>

                    <p class="t-mono-md" style="color: var(--c-gray-5); max-width: 480px; line-height:1.8">
                        "Uma máquina de empréstimos coletiva e transparente. "
                        "Cada transação registrada imutavelmente. "
                        "Governada pelos próprios membros."
                    </p>

                    <div class="flex gap-4">
                        <button
                            class="btn btn-primary btn-lg"
                            on:click=move |_| on_login.run(())
                        >
                            "ENTRAR NA COOPERATIVA"
                        </button>
                        <a href="#como-funciona" class="btn btn-ghost btn-lg">
                            "COMO FUNCIONA"
                        </a>
                    </div>

                    // Stats row
                    <div class="flex gap-8 mt-4" style="border-top: 1px solid var(--c-gray-2); padding-top: var(--sp-8)">
                        <StatBlock label="Membros Ativos" value="0+".to_string() />
                        <StatBlock label="Total Doado"    value="R$ 0".to_string() />
                        <StatBlock label="Empréstimos"    value="0".to_string() />
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
pub(crate) fn HowItWorksSection() -> impl IntoView {
    view! {
        <section class="section" id="como-funciona">
            <div class="container">
                <div class="t-cordel-rule t-display-md t-yellow mb-12">
                    "COMO FUNCIONA"
                </div>

                <div class="grid-3">
                    <Card variant=CardVariant::Default hover=true>
                        <div class="flex-col gap-4">
                            <span class="t-display-xl t-yellow" style="opacity:0.3">"01"</span>
                            <h3 class="t-display-md t-bright">"ENTRE NA COOP"</h3>
                            <p class="t-mono-sm t-muted">
                                "Faça login com seu email. Sua carteira digital é criada automaticamente — "
                                "sem seed phrase, sem complexidade técnica."
                            </p>
                        </div>
                    </Card>

                    <Card variant=CardVariant::Yellow hover=true>
                        <div class="flex-col gap-4">
                            <span class="t-display-xl t-yellow" style="opacity:0.3">"02"</span>
                            <h3 class="t-display-md t-bright">"CONTRIBUA"</h3>
                            <p class="t-mono-sm t-bright" >
                                "Faça uma doação em USDT para o fundo coletivo. "
                                "Cada contribuição aumenta sua reputação na rede."
                            </p>
                        </div>
                    </Card>

                    <Card variant=CardVariant::Default hover=true>
                        <div class="flex-col gap-4">
                            <span class="t-display-xl t-yellow" style="opacity:0.3">"03"</span>
                            <h3 class="t-display-md t-bright">"ACESSE CRÉDITO"</h3>
                            <p class="t-mono-sm t-muted">
                                "Solicite empréstimos cobertos pelos membros da sua cooperativa. "
                                "Condições decididas coletivamente."
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
                        <span class="stat-label">"Contratos na Chain"</span>
                        <span class="stat-value">"100%"</span>
                    </div>
                    <div class="stat-block t-center">
                        <span class="stat-label">"Código Auditável"</span>
                        <span class="stat-value">"✓"</span>
                    </div>
                    <div class="stat-block t-center">
                        <span class="stat-label">"Sem custódia central"</span>
                        <span class="stat-value">"✓"</span>
                    </div>
                    <div class="stat-block t-center">
                        <span class="stat-label">"Governança dos membros"</span>
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
                        "PRONTO PARA FAZER"
                        <br/>
                        <span class="t-yellow">"PARTE?"</span>
                    </h2>
                    <p class="t-mono-md t-muted">
                        "Você precisará do código de acesso da sua cooperativa para se vincular."
                    </p>
                    <button
                        class="btn btn-primary btn-lg"
                        on:click=move |_| on_login.run(())
                    >
                        "COMEÇAR AGORA"
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
                <span class="t-mono-xs t-muted">"Contratos imutáveis. Comunidade real."</span>
            </div>
        </footer>
    }
}

 
    
