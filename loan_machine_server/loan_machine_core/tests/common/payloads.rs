//! JSON shapes the subgraph returns. Centralized so a schema rename
//! touches one place.

use crate::common::deploy::DeployedEnv;
use serde_json::{json, Value};

// ── `cooperatives` query shape ───────────────────────────────

pub fn cooperative_row(
    id: &str,
    coop_id: &str,
    name: &str,
    loan_machine: &str,
    active: bool,
    registered_at: &str,
) -> Value {
    json!({
        "id": id,
        "coopId": coop_id,
        "name": name,
        "loanMachine": loan_machine,
        "active": active,
        "registeredAt": registered_at,
    })
}

/// Slimmer row without `registeredAt`. Used where the consuming query
/// doesn't read that field (e.g. memberRegisteredEvents.cooperative).
pub fn cooperative_minimal(id: &str, name: &str) -> Value {
    json!({
        "id":          id,
        "coopId":      id,
        "name":        name,
        "loanMachine": "0xdef0000000000000000000000000000000000000",
        "active":      true,
    })
}

pub fn cooperatives_with(rows: Vec<Value>) -> Value {
    json!({ "data": { "cooperatives": rows } })
}

/// Canonical "one cooperative" payload used by the cooperative-fetch tests.
pub fn one_cooperative_payload() -> Value {
    cooperatives_with(vec![cooperative_row(
        "0xaaa", "0xbbb", "Coop One", "0xccc", true, "1700000000",
    )])
}

pub fn empty_payload() -> Value {
    cooperatives_with(vec![])
}

pub fn graphql_error_payload() -> Value {
    json!({
        "errors": [{ "message": "Type Cooperative not found" }]
    })
}

// ── `memberRegisteredEvents` shapes ──────────────────────────

pub fn member_events_empty() -> Value {
    json!({ "data": { "memberRegisteredEvents": [] } })
}

pub fn member_events_with(events: Vec<Value>) -> Value {
    json!({ "data": { "memberRegisteredEvents": events } })
}

/// A `cooperative`-wrapped row that resolves to the env's deployed coop.
pub fn coop_entry_for(env: &DeployedEnv) -> Value {
    json!({
        "cooperative": {
            "id":          env.loan_machine_address.to_lowercase(),
            "coopId":      env.coop_id_hex,
            "name":        "Test Coop",
            "loanMachine": env.loan_machine_address.to_lowercase(),
            "active":      true,
        }
    })
}