# Rust Server — Solana Port Plan

This document is the authoritative plan for migrating `loan_machine_server/` from
the Ethereum/Alloy stack to Solana/Anchor. It is derived from a full audit of the
current codebase and should be updated as work progresses.

---

## Read strategy: `getProgramAccounts` replaces The Graph

On Ethereum, contract storage is opaque — you cannot ask a node "give me all
loans with status = Pending." The Graph exists to solve that: it indexes events
into a queryable database on your behalf.

On Solana, **all state lives in accounts (PDAs) that are readable at any time
via the RPC**. The `getProgramAccounts` RPC call accepts field-level filters
(offset + bytes to match), so every query that previously hit The Graph can be
answered directly against the chain.

The strategy chosen is **Path A: status-tracked PDAs, no external database**.

- PDAs are never deleted. A closed election keeps its `ElectionResult` PDA with
  `status = Closed`. A repaid loan keeps its `RequisitionState` with
  `status = Repaid`.
- The server queries on-chain accounts directly via the Helius RPC endpoint.
- No indexer infrastructure, no webhook server, no managed database to operate.
- Data is always perfectly consistent with on-chain state — no indexer lag.

The Anchor program must be designed with this in mind: every entity that the
server needs to query must have a filterable `status` or `coop_id` field at a
known byte offset in its account layout.

---

## Mental model shift: transaction assembly

The single most important conceptual change in the whole migration.

**Current EVM flow:**

```
server                                     browser (WASM)
  │                                           │
  ├─ encode calldata (alloy ABI)              │
  ├─ estimate gas (eth_estimateGas RPC)       │
  ├─ return { to, data, gas_hex }  ──────────►│
  │                                           ├─ show GasModal
  │                                           ├─ user confirms
  │                                           ├─ privy_bridge.send_tx(to, data, gas)
  │                                           ├─ Privy signs EIP-1559 tx
  │                                           └─ Privy broadcasts eth_sendRawTransaction
```

**New Solana flow:**

```
server                                     browser (WASM)
  │                                           │
  ├─ fetch recent blockhash (RPC)             │
  ├─ build Anchor instruction (Borsh)         │
  ├─ build VersionedTransaction:              │
  │    fee_payer  = user's pubkey             │
  │    instruction = ix                       │
  │    blockhash  = fetched above             │
  ├─ serialize to base64  ───────────────────►│
  │                                           ├─ NO GasModal
  │                                           ├─ privy_bridge.send_solana_tx(base64_tx)
  │                                           ├─ Privy deserializes + signs
  │                                           └─ Privy broadcasts sendTransaction RPC
```

On EVM the server sends **calldata** and Privy wraps it into a transaction.
On Solana the server must send a **fully-built serialized transaction** because
the blockhash (90-second expiry) must be baked in before signing.

---

## Cargo.toml changes (`loan_machine_core`)

```toml
# REMOVE
alloy = { ... }

# ADD
solana-client  = "^2"
solana-sdk     = "^2"
anchor-client  = "0.31"   # must match anchor-lang version in programs/
borsh          = "1"
```

---

## Layer-by-layer work items

### `services/blockchain/` — full rewrite

Every file in this module is EVM-specific. The struct shape
(`BlockchainService` holding sub-services, wired in `AppState`) survives; the
implementations do not.

---

#### `provider.rs` — rewrite

**Current:** builds an `alloy` RPC provider.

```rust
pub type Provider = RootProvider<Http<Client>>;

pub fn build_provider(rpc_url: &str) -> Result<Provider, BlockchainError> {
    ProviderBuilder::new().on_http(url)
}
```

**After:** builds a `solana-client` async RPC client.

```rust
use solana_client::nonblocking::rpc_client::RpcClient;

pub fn build_rpc_client(rpc_url: &str) -> RpcClient {
    RpcClient::new(rpc_url.to_string())
}
```

---

#### `abis.rs` — delete

The `sol!` macro generates Rust types from Solidity ABI. On Solana there is no
ABI — instructions use Borsh. Typed Rust builders come from `anchor-client`,
which reads the IDL that `anchor build` produces. Delete this file.

---

#### `coop_registry.rs` — rewrite

**Current:** calls the `CoopRegistry` EVM contract via `alloy`.

**After:** reads `GlobalState` and `CoopState` PDAs directly from the RPC.

```rust
// GlobalState PDA: seeds = [b"global"]
// CoopState   PDA: seeds = [b"coop", coop_id_bytes]

let data = rpc_client.get_account_data(&global_state_pda).await?;
let state = GlobalState::try_deserialize(&mut data.as_ref())?;
```

Borsh deserialization uses the same struct definitions as the Anchor program
(import the types from a shared crate or re-declare them here with matching
`#[derive(AnchorDeserialize)]`).

---

#### `loan_machine_fns_helpers.rs` — rewrite

**Current:** two kinds of functions per action — `encode_*` (ABI calldata) and
`estimate_*_gas` (RPC gas estimate).

**After:** one function per action — `build_ix_*` — that returns a Solana
`Instruction`. Gas estimation disappears entirely (Solana fees are always
`n_signatures × 5000 lamports`, no estimation needed).

```rust
pub fn build_ix_join_coop(
    program_id: &Pubkey,
    member_id:  [u8; 32],
    wallet:     Pubkey,
    payer:      Pubkey,
) -> Instruction {
    // anchor discriminator + borsh-encoded args
}
```

The server then wraps the instruction in a `VersionedTransaction` with a fresh
blockhash before sending it to the browser.

---

#### `contract_errors.rs` + `translate_revert` — rewrite

**Current:** decodes ABI-encoded EVM revert data and maps selectors to messages.

**After:** reads Anchor's error code from the transaction logs and maps it to
the `#[error_code]` enum in the Anchor program.

```rust
// Anchor errors appear in logs as:
// "Program log: AnchorError ... Error Code: SomeVariant. ..."
// Parse the log string or match the u32 error code returned by the RPC.
```

---

#### `deployable.rs` — delete

**Current:** deploys a new `LoanMachine.sol` bytecode instance per cooperative.

**After:** there is nothing to deploy. On Solana one program handles all
cooperatives. Creating a new cooperative means sending a `create_coop`
instruction to the `loan_machine` program, which initializes a `CoopState` PDA.
No bytecode is uploaded. Delete this file and the `deployable` Cargo feature.

---

#### `mod.rs` / `BlockchainService` struct — update

Remove `CoopRegistryService` (it becomes a direct PDA-reader pattern).
The struct simplifies to holding the `RpcClient` and the program ID.

```rust
pub struct BlockchainService {
    pub rpc:        Arc<RpcClient>,
    pub program_id: Pubkey,
}
```

---

### `services/subgraph/` — replace with `AccountQueryService`

**Current:** GraphQL client for The Graph. Sends POST requests with query
strings, deserializes JSON responses into typed structs.

**After:** delete this service entirely. There is no external indexer to call.
Queries go directly to the Solana RPC via `getProgramAccounts`.

Create `services/account_query/mod.rs`:

```rust
pub struct AccountQueryService {
    client:     Arc<RpcClient>,
    program_id: Pubkey,
}

impl AccountQueryService {
    // Example — replaces the open_market subgraph query:
    pub async fn get_open_requisitions(&self, coop_id: [u8; 32])
        -> Result<Vec<RequisitionState>, AccountQueryError>
    {
        let filters = vec![
            // 8-byte Anchor discriminator identifies RequisitionState accounts
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                0,
                RequisitionState::discriminator().to_vec(),
            )),
            // coop_id field at its byte offset in the account layout
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                REQUISITION_COOP_ID_OFFSET,
                coop_id.to_vec(),
            )),
            // status byte == 0 (Pending)
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                REQUISITION_STATUS_OFFSET,
                vec![0u8],
            )),
        ];
        // deserialize each returned account into RequisitionState
        ...
    }
}
```

The field offsets (`REQUISITION_COOP_ID_OFFSET`, etc.) are constants derived
from the Anchor account layout. Compute them once and document them in the
service file.

---

### `server_logic/subgraph_queries/` — rename and rewrite

Rename the directory to `server_logic/account_queries/`. Each file keeps the
same function signature and return type; only the internals change from
GraphQL string + JSON parse to `AccountQueryService` calls + Borsh deserialization.

| File | Query becomes |
|---|---|
| `cooperative_by_id.rs` | fetch `CoopState` PDA by seeds `[b"coop", coop_id]` |
| `cooperatives.rs` | `getProgramAccounts` filter on `CoopState` discriminator |
| `membership.rs` | `getProgramAccounts` filter on `MemberState.wallet == pubkey` |
| `member_financials.rs` | fetch `MemberState` PDA + SPL token account balance via RPC |
| `my_requisitions.rs` | `getProgramAccounts` filter on `RequisitionState.borrower == pubkey` |
| `open_market.rs` | `getProgramAccounts` filter on `RequisitionState.status == Pending` |
| `pending_approval.rs` | `getProgramAccounts` filter on `MembershipProposal.resolved == false` |
| `pending_proposals_count.rs` | same filter as above, return `.len()` |
| `user_coops.rs` | `getProgramAccounts` filter on `MemberState.wallet == pubkey` |
| `user_related_coops.rs` | same, different post-filter in business logic |
| `approve_wallet_proposals.rs` | `getProgramAccounts` filter on `WalletProposal` discriminator |
| `last_closed_election.rs` | `getProgramAccounts` filter on `ElectionResult.coop_id`, sort by slot |

---

### `server_logic/` — mostly unchanged

Business logic is chain-agnostic: coverage math, loan lifecycle, election
tallying, reputation averaging. These files call blockchain/account query
services through typed interfaces. As long as the service return types match,
this layer needs no changes beyond removing `gas` fields from return structs.

---

### `wallet_auth/privy_bridge.rs` — update send path

Privy supports Solana natively. Authentication (JWT verification in
`privy_auth/`) is chain-agnostic and unchanged.

The send side changes from sending calldata to sending a serialized transaction:

```rust
// REMOVE
pub fn send_tx(to: &str, data: &str, gas: Option<&str>) { ... }
pub fn deploy_contract(data: &str, gas_hex: &str) { ... }

// ADD
pub fn send_solana_tx(base64_tx: &str) {
    // base64-encoded VersionedTransaction, ready for Privy to sign
    call_window_fn_with_arg("loan_machine_send_solana_tx", base64_tx);
}
```

`privy-bridge.js` replaces the `eth_sendRawTransaction` call with Privy's
Solana `sendTransaction()` method.

---

### `components/gas_modal.rs` — delete

Solana transaction fees are always `n_signatures × 5000 lamports` (~$0.001).
There is nothing to estimate and nothing to confirm with the user.

Files to delete:
- `components/gas_modal.rs`
- `components/gas_modal_test.rs`

Remove from every component:
- `use_gas_modal()` / `provide_gas_modal()` calls
- `GasEstimate` struct usage
- `GasModalRequest` construction

The `get_wallet_balance_wei` server function (used only to show the ETH balance
in the modal) is deleted as well.

---

### `loan_machine_models/wallet_address.rs` — update

**Current:** wraps an EVM hex address — `0x` prefix, 20 bytes, checksummed.

**After:** wraps a Solana base58 pubkey — 32 bytes, no prefix.

```rust
// From:
pub struct WalletAddress(String); // "0x1234...abcd"
// To:
pub struct WalletAddress(String); // "4Zn9...base58"
```

Every place the server compares or stores wallet addresses adapts automatically
since they go through this newtype.

---

### `services/identity/` — unchanged

`memberId = keccak256(coop_salt || cpf_or_cnpj)` → `[u8; 32]`.
The hash algorithm, salt generation, and server-side computation are
chain-agnostic. The Anchor program treats `memberId` as an opaque `[u8; 32]`.
No changes needed.

---

### `services/privy/` — unchanged

Holds the Privy `app_id` and generates the `window.APP_CONFIG` JS blob.
Privy's app ID is the same regardless of chain. No changes needed.

### `services/privy_auth/` — unchanged (one field check)

JWT verification logic is identical. The only thing to verify: Privy's JWT
payload contains a wallet address. After migrating to a Solana-linked wallet,
that address will be a base58 pubkey. The JWT parsing code does not care about
the format — it is stored as a string and passed into `WalletAddress::parse()`,
which is the type that changes.

### `services/cache/` — unchanged

### `services/chain_config/` — update

`ChainConfig` currently holds EVM chain ID, RPC URL, contract addresses. After:

- Remove: `chain_id`, `coop_registry_address`
- Keep: `rpc_url`
- Add: `program_id` (the `loan_machine` Anchor program pubkey)

---

## Summary table

```
services/blockchain/
  provider.rs                  rewrite  → RpcClient (solana-client)
  abis.rs                      DELETE
  coop_registry.rs             rewrite  → read GlobalState / CoopState PDAs
  loan_machine_fns_helpers.rs  rewrite  → build_ix_* functions, no gas estimate
  contract_errors.rs           rewrite  → Anchor error code parsing
  deployable.rs                DELETE

services/subgraph/             DELETE   → replaced by services/account_query/
services/account_query/        CREATE   → getProgramAccounts wrapper

services/privy/                UNCHANGED
services/privy_auth/           UNCHANGED
services/cache/                UNCHANGED
services/chain_config/         update   → remove chain_id/contract_addr, add program_id
services/identity/             UNCHANGED

server_logic/
  subgraph_queries/ (dir)      RENAME   → account_queries/
  account_queries/*.rs         rewrite  → AccountQueryService calls + Borsh deserialization
  *.rs (all others)            UNCHANGED (remove gas fields from return structs)

wallet_auth/
  privy_bridge.rs              update   → send_tx → send_solana_tx (base64 tx payload)
  privy-bridge.js              update   → Privy Solana sendTransaction()
  session.rs                   UNCHANGED
  auth_header.rs               UNCHANGED

components/
  gas_modal.rs                 DELETE
  gas_modal_test.rs            DELETE
  all others                   remove GasModal/GasEstimate usage

models/
  wallet_address.rs            update   → base58 pubkey (32 bytes) instead of EVM hex

Cargo.toml (core)
  alloy = ...                  DELETE
  + solana-client, solana-sdk, anchor-client, borsh
```

---

## What is NOT in scope here

- The Anchor program itself (`programs/loan_machine/`) — that is a separate
  build pipeline. The PDA layouts defined there determine the byte offsets used
  in `AccountQueryService` filters, so the program must be designed first.
- The TypeScript tests (`tests/`) — those already target the Anchor program
  directly and do not go through the Rust server.
