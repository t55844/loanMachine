# Loan Machine — Solana

## What this project is

A decentralized credit cooperative (DeFi) platform. Members pool funds (USDT), request loans
covered by other members, and repay in installments. Reputation scores and moderator elections
govern trust within each cooperative. Multiple independent cooperatives can exist under one
platform registry.

This repository is a **migration** of the original Ethereum/Solidity implementation to Solana.

---

## Current State

The `hardhat/` directory contains the **original Ethereum contracts** — the source of truth for
business logic. They are kept as reference during the Solana port; they are not deployed.

The Solana port is in progress:
- On-chain layer: **not started** — Anchor programs to be built under `programs/`
- Rust server (`loan_machine_server/`): **carries over** mostly unchanged; blockchain service
  layer will be swapped from Alloy (EVM) to Solana RPC + Anchor client

See [`docs/ethereum-to-solana-migration.md`](docs/ethereum-to-solana-migration.md) for the
full mapping of Solidity concepts to Solana equivalents.

---

## Architecture

Four layers, each with a single clear job:

### 1. Anchor Programs (`programs/`) — on-chain truth
Stateless Rust programs on Solana. Hold no state themselves; all state lives in PDAs (Program
Derived Addresses) they own. Responsible for:
- Enforcing financial rules (coverage, repayment, withdrawal limits)
- Emitting events for off-chain indexing
- Managing membership approval (multisig proposals)

Key programs to build:
- `loan_machine` — requisitions, coverage, repayment, reputation, elections
- `coop_registry` — platform-level registry mapping coop IDs to program/accounts

### 2. Event Indexer — primary read source
Listens to Anchor events and builds a queryable store. On Solana, options are:
- Yellowstone gRPC (Triton / Helius)
- Helius webhooks
- Custom indexer reading parsed transaction logs

Replaces The Graph (used in the Ethereum version).

### 3. Rust Server (`loan_machine_server/`) — central hub
Full-stack Leptos SSR+WASM app. Builds and validates transactions, serves the UI, manages
user identity, and coordinates the full lifecycle of every action.

Three crates:
- `loan_machine_web` — Leptos components, server functions, wallet auth
- `loan_machine_core` — business logic, blockchain services, Privy integration, event indexer client
- `loan_machine_models` — shared types

### 4. Privy — custodial wallet
Signs and broadcasts transactions on behalf of users. Privy supports Solana. The JS bridge
(`loan_machine_server/loan_machine_web/public/privy-bridge.js`) is the only place that imports
the Privy SDK; it exposes functions on `window` and dispatches DOM events back to the
Rust/WASM layer.

---

## Data Flows

**Write (transaction):**
```
User action
  → Rust Server (build instruction, validate business rules)
    → Privy (sign + broadcast to Solana)
      → Anchor Program (execute, emit events)
        → confirmation back to Rust Server
          → update UI
```

**Read (query):**
```
User opens a page
  → Rust Server fetches from event indexer
    → process and serve to browser
```

---

## Key Design Decisions

- **Off-chain as much as possible.** Overdue detection, loan lists, vote tallying, and
  reputation averages are computed off-chain — not stored on-chain. Keeps compute costs low.
- **Privy for wallets.** Users never manage keys or seed phrases. The Rust server builds and
  validates the instruction; Privy's enclave handles signing.
- **memberId is opaque on-chain.** `memberId = keccak256(coop_salt || cpf_or_cnpj)` — the
  hash is computed by the server. The Anchor program treats it as an opaque `[u8; 32]`. CPF/CNPJ
  never touches the chain.
- **Per-coop salt.** The same CPF in two different cooperatives produces different `memberId`
  hashes. The salt is generated at cooperative creation and stored server-side only.
- **USDT via SPL Token.** Funds flow through SPL token accounts. The program holds a PDA-owned
  vault token account; it never holds SOL for user funds.
- **Rust + Leptos SSR+WASM.** The same Rust code compiles to a native server binary (Axum,
  Tokio, Solana RPC) and a WASM bundle (runs in the browser). No separate frontend codebase.

---

## Repository Layout

```
.
├── CLAUDE.md                        ← this file
├── docs/
│   ├── overview.md                  ← original Ethereum architecture overview
│   ├── ethereum-to-solana-migration.md  ← migration mapping (contracts + toolchain)
│   ├── auth.md                      ← Privy JWT auth, WalletSession, JS bridge
│   ├── money_handling.md            ← financial rules, coverage math
│   ├── cpf_cnpj_identity.md         ← identity hashing design
│   ├── rust_errors_and_types.md     ← error handling philosophy
│   └── rust_leptos_study_notes.md   ← Leptos SSR+WASM architecture notes
├── hardhat/                         ← REFERENCE ONLY — original Solidity contracts
│   └── contracts/
│       ├── LoanMachine.sol          ← core logic (port this to Anchor)
│       ├── CoopRegistry.sol         ← registry (port this to Anchor)
│       ├── ReputationLib.sol        ← reputation + elections library
│       ├── ILoanMachine.sol         ← interface / event definitions
│       └── IReputationSystem.sol    ← election structs
├── programs/                        ← (to be created) Anchor programs
└── loan_machine_server/             ← Rust full-stack server
    ├── loan_machine_web/            ← Leptos UI + server functions
    ├── loan_machine_core/           ← business logic + blockchain services
    └── loan_machine_models/         ← shared types
```

---

## Useful Commands

```bash
# Local Solana validator
solana-test-validator

# Build Anchor programs (once programs/ exists)
anchor build

# Deploy to localnet
anchor deploy

# Run Rust server (dev)
cargo leptos watch -p loan_machine_web
```

---

## Related Docs

- [`docs/ethereum-to-solana-migration.md`](docs/ethereum-to-solana-migration.md) — full
  concept mapping: Solidity → Anchor, storage model, token handling, multisig
- [`docs/overview.md`](docs/overview.md) — original Ethereum architecture (useful for
  understanding intended behavior)
- [`docs/auth.md`](docs/auth.md) — Privy auth subsystem detail
- [`docs/money_handling.md`](docs/money_handling.md) — financial rules and coverage math
- [`docs/cpf_cnpj_identity.md`](docs/cpf_cnpj_identity.md) — identity hashing design
