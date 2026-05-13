// loan_machine_core/src/services/subgraph/mod.rs
//
// SubgraphService — minimal GraphQL client for our indexed events.
//
// Holds the subgraph URL privately (same encapsulation pattern as
// BlockchainService). Server logic asks this service for typed data;
// it never sees the raw URL or constructs queries directly.


use std::fmt;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubgraphError{
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("graphql error: {0}")]
    Graphql(String),
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("subgraph response missing `data` field")]
    MissingData,
}


#[derive(Clone)]
pub struct SubgraphService {
    client: Client,
    url: String,
}

impl SubgraphService{
    /// Build a service with sane HTTP defaults (10s timeout).
    pub fn new(url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client with default settings should not fail to build");

        Self { client, url }
    }


    pub async fn query<T, V>(&self, query: &str, variables: V) -> Result<T, SubgraphError>
        where
            T: for<'de> Deserialize<'de>,
            V: Serialize,
        {
            #[derive(Serialize)]
            struct Request<'a, V> {
                query: &'a str,
                variables: V,
            }
    
            #[derive(Deserialize)]
            struct GraphqlError {
                message: String,
            }
    
            #[derive(Deserialize)]
            struct Envelope<T> {
                data: Option<T>,
                #[serde(default)]
                errors: Option<Vec<GraphqlError>>,
            }
    
            let body = Request { query, variables };
    
            // Pull the body as text first so JSON parse failures surface as
            // SubgraphError::Decode (not Network), which is what they actually are.
            let response_text = self
                .client
                .post(&self.url)
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?;
    
            let envelope: Envelope<T> = serde_json::from_str(&response_text)?;
    
            if let Some(errors) = envelope.errors {
                if !errors.is_empty() {
                    let joined = errors
                        .into_iter()
                        .map(|e| e.message)
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Err(SubgraphError::Graphql(joined));
                }
            }
    
        envelope.data.ok_or(SubgraphError::MissingData)
    }
}
    
    impl fmt::Debug for SubgraphService {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("SubgraphService")
                .field("url", &"[REDACTED]")
                .finish()
        }
}