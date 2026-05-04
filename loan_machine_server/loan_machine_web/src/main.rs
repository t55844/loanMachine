#![recursion_limit = "256"]

use axum::Router;
use leptos::prelude::*;
use leptos::hydration::HydrationScripts;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_config::{get_configuration, LeptosOptions};
use loan_machine_core::config::AppState;
use loan_machine_web::app::App;
use tower_http::services::ServeDir;

#[component]
fn Shell(
    options: LeptosOptions,
    privy_app_id: String,
    rpc_url:      String,
    chain_id:     u64,
    ) -> impl IntoView {

    let config_js = format!(
        "window.APP_CONFIG = {{ privyAppId: '{}', rpcUrl: '{}', chainId: {} }};",
        privy_app_id, rpc_url, chain_id
    );

    view! {
        <!DOCTYPE html>
        <html lang="pt-BR">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"Loan Machine"</title>
                <link rel="stylesheet" href="/style/design-system.css"/>
                <link rel="stylesheet" href="/style/auth-bar.css"/>
                <script inner_html=config_js />
                <HydrationScripts options=options.clone()/>
            </head>
            <body>
                <App/>
                <script type="module" src="/privy-bridge.js"></script>
            </body>
        </html>
    }
}

#[tokio::main]
async fn main() {
    dotenvy::from_filename(".env").ok();
        tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("loan_machine_web=debug".parse().unwrap())
                .add_directive("loan_machine_core=debug".parse().unwrap())
        )
        .init();

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let site_root = leptos_options.site_root.clone();

    let state = AppState::new(leptos_options.clone()).await;
    let routes = generate_route_list(App);

    let shell_app_id   = state.privy.app_id().to_string();
    let shell_rpc      = state.chain_config.rpc_url.clone();
    let shell_chain    = state.chain_config.chain_id;

    let router = Router::new()
        .leptos_routes(
            &state,
            routes,
            {
                let opts = leptos_options.clone();
                move || view!{
                    <Shell
                        options=opts.clone()
                        privy_app_id=shell_app_id.clone()
                        rpc_url=shell_rpc.clone()
                        chain_id=shell_chain
                    />
                }
            },
        )
        .fallback_service(ServeDir::new(&*site_root))
        .with_state(state);

    tracing::info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

/*
Symbol	Name	Meaning
!	Bang	Macro: Code that writes code at compile-time.
?	Question Mark	Error Handling: "Give me the value or return the error."
#	Hash/Attribute	Metadata: Instructions for the compiler.
&	Ampersand	Borrowing: Accessing data without taking ownership.
::	Double Colon	Path: Navigating modules (like crate::models).

variable that i will use but don't want to trigger "unused variable" warnings.
    let _subgraph_url = "https://api.thegraph.com/...";
    struct Config {
        #[allow(dead_code)] // This silences the warning just for this field
        subgraph_url: String,
        api_key: String,
    }
    fn fetch_data(url: &str) {
        todo!("Implement the GraphQL query for {}", url);
    }
 */