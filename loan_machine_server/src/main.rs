// src/main.rs


use tracing_subscriber;
use loan_machine_server::config::AppState;
use loan_machine_server::create_app;
use leptos_config::{get_configuration, LeptosOptions};
use leptos_axum::{generate_route_list, LeptosRoutes};
use loan_machine_server::app::App;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let conf = get_configuration(None).unwrap();
    let leptos_options: LeptosOptions = conf.leptos_options;
    let addr = leptos_options.site_addr;

    // 1. Initialize your state
    let state = AppState::new(leptos_options).await;
    let routes = generate_route_list(App);

    // 2. Build the complete router, THEN apply the state
    let router = create_app()
        .leptos_routes(&state, routes, App)
        .with_state(state.clone()); // <--- This is the magic! It turns Router<AppState> into Router<()>

    tracing::info!("Rust server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    // 3. In Axum 0.7+, you just pass the router directly
    axum::serve(listener, router)  
        .await
        .unwrap();
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