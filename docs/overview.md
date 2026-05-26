# Loan Machine — Project Overview

A decentralized credit cooperative (DeFi) platform that lets users borrow and lend funds peer-to-peer.
The system is designed to keep gas costs low by pushing as much as possible off-chain, and to remove wallet-management friction from end users via Privy.

---

## Architecture

The system has four layers that each have a clear, distinct job.

### 1. Smart Contract (Polygon / Ethereum)
**Job:** Financial truth and hard validation.

Deployed on Polygon or Ethereum. Responsible for the "value" of the project — the canonical on-chain record of loans, repayments, and cooperative membership. Enforces critical rules that prevent bad actors and protect funds. Kept as lean as possible to minimize gas costs; heavy computation and complex queries are offloaded to the layers below.

Contracts live in `hardhat/contracts/`:
- `LoanMachine.sol` — core: loan requisitions, lender coverage, repayment parcels, overdue tracking
- `CoopRegistry.sol` / `CoopAccount.sol` — cooperative membership and wallet approval
- `ReputationLib.sol` / `IReputationSystem.sol` — on-chain reputation scores
- `MockUSDT.sol` — ERC-20 token used in local/test environments
- Multisig governance with proposal types: `TransferAdmin`, `ApproveWallet`, `RotateAccessCode`, etc.

---

### 2. The Graph (Subgraph)
**Job:** Event indexing and primary read source.

Listens to events emitted by the smart contracts and indexes them into a queryable GraphQL API. Orchestrates data across multiple LoanMachine instances. This is the **primary source for all queries** because:
- Queries are free (no gas)
- Data from multiple events and contracts can be correlated easily
- Much faster than on-chain RPC calls

---

### 3. Rust Server (`loan_machine_server/`) — the central hub
**Job:** Connect everything. Build, validate, and route transactions. Manage users.

This is where the system's logic lives. It:
- Builds and validates transactions before forwarding them to Privy for signing
- Fetches data from The Graph, processes it, and serves it to the user
- Manages users: identity, access control, what they can see and do
- Coordinates the full lifecycle of every user action

Built as a Leptos full-stack SSR+WASM app (Rust on both server and browser). Three crates:

| Crate | Purpose |
|---|---|
| `loan_machine_web` | Leptos components, server functions, wallet auth layer |
| `loan_machine_core` | Business logic, blockchain services (Alloy), subgraph client, Privy integration |
| `loan_machine_models` | Shared types: requests, responses, `WalletAddress` |

Services inside `loan_machine_core/src/services/`:
- `blockchain/` — Alloy-based contract calls and transaction building
- `privy/` — transaction signing and broadcast via Privy
- `privy_auth/` — JWT verification and wallet lookup (JWKS-based, cached)
- `subgraph/` — GraphQL queries against The Graph
- `identity/` — user identity management
- `cache/` — caching layer
- `chain_config/` — chain and contract configuration
- `coop_deployment/` — cooperative deployment logic

---

### 4. Privy
**Job:** Custodial wallet — sign transactions, broadcast to chain.

Privy holds embedded wallets for users, completely removing wallet-management friction (no seed phrases, no MetaMask setup). Its role in the flow:
1. Receives a prepared, validated transaction from the Rust server
2. Signs it using the user's embedded wallet (private key stays in Privy's enclave)
3. Broadcasts the signed transaction to the chain
4. Waits for on-chain confirmation
5. Returns the result to the Rust server

The JS bridge (`public/privy-bridge.js`) is the only file that imports the Privy SDK directly. It exposes functions on `window` for the Rust/WASM layer to call, and dispatches custom DOM events back.

---

## Data Flows

### Write (transaction)
```
User action
  → Rust Server (build TX, validate business rules)
    → Privy (sign + broadcast to chain)
      → Smart Contract (execute, emit events)
        → confirmation back to Rust Server
          → update UI
```

### Read (query)
```
User opens a page
  → Rust Server fetches from The Graph (GraphQL)
    → process and serve to browser
```

---

## Key Design Decisions

- **Off-chain as much as possible:** overdue detection, loan lists, vote tallying, and reputation averages are computed off-chain by the Rust server or The Graph — not stored on-chain. This keeps gas costs low.
- **Privy for wallets:** users never manage keys or seed phrases. The Rust server builds and validates the TX; Privy's enclave handles signing.
- **The Graph as primary read source:** reading from a GraphQL index is free and fast. Direct on-chain RPC calls are a fallback, not the default.
- **Rust + Leptos SSR+WASM:** the same Rust code compiles to a native server binary (uses Axum, Alloy, Tokio) and a WASM bundle (runs in the browser). No separate frontend codebase.

---

## Related Docs

- [auth.md](auth.md) — auth subsystem in detail: Privy JWT verification, `Authenticated` type-witness, `WalletSession` state machine, JS bridge, server fn patterns
- [rust_errors_and_types.md](rust_errors_and_types.md) — error handling philosophy: `thiserror`, `#[from]`/`#[source]`, layered errors, stringify-at-the-exit rule
- [rust_leptos_study_notes.md](rust_leptos_study_notes.md) — Leptos architecture: SSR+WASM dual compilation, `AppState`/`Arc`, `#[server]` bridge, `Resource`/`Action`/signals, full lifecycle walkthrough
