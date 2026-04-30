// loan_machine_core/src/services/subgraph/mod.rs
//
// SubgraphService — minimal GraphQL client for our indexed events.
//
// Holds the subgraph URL privately (same encapsulation pattern as
// BlockchainService). Server logic asks this service for typed data;
// it never sees the raw URL or constructs queries directly.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
pub enum SubgraphError {
    Network(reqwest::Error),
    Graphql(String),
    Decode(serde_json::Error)
}

impl fmt::Display for SubgraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubgraphError::Network(e) => write!(f, "Subgraph Network error: {}", e),
            SubgraphError::Graphql(e) => write!(f, "Subgraph GraphQL error: {}", e),
            SubgraphError::Decode(e) => write!(f, "Subgraph Decode error: {}", e),
        }
    }
}

impl std::error::Error for SubgraphError {}

pub struct SubgraphService {
    client: Client,
    url:    String,
}

impl SubgraphService{
    pub fn new(url: String) -> Self{
        Self {
            client: Client::new(),
            url,
        }
    }

    pub async fn query<T, V>(&self, query: &str, variables: V) -> Result<T, SubgraphError>
    where 
        T: for<'de> Deserialize<'de>,
        V: Serialize,
    {
        #[derive(Serialize)]
        struct Request<'a, V> {
            query:     &'a str,
            variables: V,
        }

        let body = Request{query, variables};
        let resp = self.client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .map_err(SubgraphError::Network)?
            .error_for_status()
            .map_err(SubgraphError::Network)?;

        let envelope: Value = resp.json().await.map_err(SubgraphError::Network)?;

        if let Some(errors) = envelope.get("errors"){
            if !errors.is_null(){
                return Err(SubgraphError::Graphql(errors.to_string()))
            }
        }

        let data = envelope
            .get("data")
            .cloned()
            .unwrap_or(Value::Null);

        serde_json::from_value(data).map_err(SubgraphError::Decode)
    }
}

impl fmt::Debug for SubgraphService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubgraphService")
            .field("url", &"[REDACTED]")
            .finish()
    }
}