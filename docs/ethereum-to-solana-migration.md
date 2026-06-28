# Ethereum → Solana Migration Overview

This document covers the principal changes required when porting the Loan Machine from the
Ethereum/Solidity/Hardhat stack to Solana/Anchor/Rust. It focuses exclusively on the
on-chain layer (contracts + toolchain). The Rust server migration is tracked separately.

---

## 1. Toolchain (Hardhat → Anchor)

| Ethereum | Solana |
|---|---|
| `hardhat.config.js` | `Anchor.toml` + `Cargo.toml` |
| `npx hardhat node` | `solana-test-validator` |
| `npx hardhat run scripts/deploy.js` | `anchor deploy` |
| Tests: Mocha + `ethers.js` | Tests: Anchor TypeScript (`@coral-xyz/anchor`) or Rust |
| OpenZeppelin (`ReentrancyGuard`, `IERC20`) | No equivalent; written from scratch or community crates |
| `package.json` with hardhat deps | `Cargo.toml` with `anchor-lang`, `spl-token` |
| GraphQL / The Graph for event indexing | Yellowstone gRPC or Helius webhooks |

---

## 2. File Structure

All `.sol` files are replaced by Rust Anchor programs:

| Solidity file | Solana equivalent |
|---|---|
| `LoanMachine.sol` | `programs/loan_machine/src/lib.rs` |
| `CoopRegistry.sol` | Same program or `programs/coop_registry/src/lib.rs` |
| `ILoanMachine.sol`, `IReputationSystem.sol` | **Deleted** — Anchor instructions define the API |
| `ReputationLib.sol` | Rust module `src/reputation.rs` |
| `libraries/LoanCalculations.sol` | Rust module `src/loan_calc.rs` |
| `structs/LoanStructs.sol` | Rust structs in `src/state.rs` |
| `MockUSDT.sol` (ERC-20) | Create a test SPL token mint via CLI / test setup |

---

## 3. Language / Concept Mapping

| Solidity | Solana/Rust |
|---|---|
| `msg.sender` | `ctx.accounts.signer.key()` |
| `address` (20 bytes) | `Pubkey` (32 bytes) |
| `bytes32 memberId` | `[u8; 32]` — same semantic, same keccak256 hash from the server |
| `uint256` | `u64` (Solana has no native 256-bit int) |
| `mapping(K => V)` | PDA (Program Derived Address) per key |
| `event Foo(...)` + `emit Foo(...)` | `#[event] struct Foo` + `emit!(Foo {...})` |
| `modifier onlyAdmin()` | `require!(is_admin, LoanError::NotAdmin)` |
| `nonReentrant` | **Not needed** — Solana is non-reentrant by design |
| `block.timestamp` | `Clock::get()?.unix_timestamp` |
| `error LoanMachine_NotAdmin()` | `#[error_code] enum LoanError { NotAdmin }` |
| `constructor(address _usdtToken)` | `initialize` instruction writing to a PDA |
| `abi.encode/decode` | Borsh serialization (Anchor handles automatically) |
| `keccak256(...)` | `solana_program::keccak::hashv(...)` (still available) |
| `IERC20(usdtToken).transferFrom(...)` | SPL Token CPI `transfer(...)` |
| `IERC20(usdtToken).balanceOf(address(this))` | Read token account balance directly |

---

## 4. Storage Model (Biggest Redesign)

Solidity stores all state inside the contract (mappings, arrays). Solana programs are
**stateless** — all state lives in separate **accounts** owned by the program.

### Current Solidity state → PDAs required

| Solidity storage | Solana PDA (seeds) |
|---|---|
| `donations[wallet]`, `borrowings[wallet]` | `UserAccount` PDA: `[b"user", wallet_pubkey]` |
| `loanRequisitions[id]` | `RequisitionAccount` PDA: `[b"requisition", id.to_le_bytes()]` |
| `loanContracts[id]` | `LoanContractAccount` PDA: `[b"loan", id.to_le_bytes()]` |
| `debtWatchlist` (dynamic array) | Iterated off-chain; individual `DebtWatchAccount` PDAs if needed |
| `proposals[id]` (multisig) | `ProposalAccount` PDA: `[b"proposal", id.to_le_bytes()]` |
| `coops[coopId]` (CoopRegistry) | `CoopRecord` PDA: `[b"coop", coop_id]` |
| Global counters (`requisitionCounter`, etc.) | `GlobalState` PDA: `[b"global"]` |

### Inner mapping in LoanRequisition

`LoanRequisition` contains `mapping(address => uint256) coverageAmounts` — impossible in
Anchor's account model. Must be split into:

- `RequisitionAccount` — fixed-size requisition fields, no inner map.
- `CoverageAccount` PDA per lender: `[b"coverage", requisition_id, lender_pubkey]` — stores
  that lender's coverage amount.
- `coveringLenders: address[]` — either fixed-size array (e.g., `[Pubkey; 20]`) or tracked
  off-chain via events.

### Dynamic arrays

`paymentDates` and `parcelsAmounts` must be **fixed-size**. Max 12 parcels:
`payment_dates: [i64; 12]`, `parcel_amounts: [u64; 12]`.

---

## 5. Token Handling (ERC-20 → SPL Token)

- USDT exists on Solana as an **SPL token** (same concept, different standard).
- The program does **not** hold tokens directly — it owns a **vault token account**
  (a regular SPL token account whose authority is a PDA).
- `IERC20(usdtToken).transferFrom(user, contract, amount)` → SPL Token CPI from user's
  Associated Token Account (ATA) to the program vault.
- `IERC20(usdtToken).transfer(to, amount)` → SPL Token CPI from vault to recipient's ATA,
  signed by the vault PDA.
- `MockUSDT.sol` is replaced by `spl-token create-token` in test setup.

---

## 6. Multisig

The current inline multisig (proposals, confirmations, threshold) must be either:

- **Rewritten in Rust** — same proposal/confirmation pattern, each `ProposalAccount` is a PDA.
- **Integrated with Squads Protocol** — the standard Solana multisig. Program admin authority
  becomes a Squads vault; admin actions become Squads transactions. Simpler but adds an
  external dependency.

---

## 7. CoopRegistry

Straightforward to port. The `coops` mapping becomes one `CoopRecord` PDA per coop.
`platformAdmin` becomes a `Pubkey` stored in a `GlobalState` PDA.

---

## 8. What Does NOT Change

- **Business logic** — all cooperative rules, coverage math, parcel repayment, reputation
  scoring, and election logic are fully portable to Rust.
- **`memberId` hashing** — `keccak256(salt || cpf)` continues to work the same way. The
  server still generates it; the on-chain program treats it as an opaque `[u8; 32]`.
- **Off-chain-first design** — overdue detection, loan lists, vote tallying, and reputation
  averages stay off-chain (server + event indexing).
- **Privy wallet layer** — Privy supports Solana; the bridge JS changes but the pattern
  (server builds TX → Privy signs → broadcasts) is the same.
