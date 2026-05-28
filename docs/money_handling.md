# Money Handling

This document covers how financial data flows through the system. It will be expanded as new features are added. Current sections reflect what is implemented; future sections are stubs.

---

## 1. Member Financials

A member's financial snapshot combines two data sources: the **subgraph** (event history) and the **chain** (live contract state). Neither source alone is sufficient — the subgraph is fast and aggregatable but slightly delayed; the chain is authoritative but expensive to query for multiple fields at once.

### 1a. Data model — `MemberFinancials`

**File:** `loan_machine_models/src/responses.rs`

| Field                | Source      | Description                                                   |
|----------------------|-------------|---------------------------------------------------------------|
| `wallet`             | auth        | Display address of the requesting wallet                      |
| `member_id`          | chain/graph | `bytes32` hex — the on-chain identity hash                    |
| `reputation`         | subgraph → chain fallback | Signed integer; positive = good standing, negative = overdue |
| `donation`           | subgraph → chain fallback | Current donation balance, raw token units (string)            |
| `borrowing`          | subgraph → chain fallback | Outstanding borrowed amount, raw token units (string)         |
| `last_borrow_time`   | subgraph → chain fallback | Unix timestamp of latest borrow; `0` if never borrowed        |
| `in_coverage`        | chain only  | Portion of donation locked in active loan coverage            |
| `withdrawable`       | chain only  | `donation − in_coverage`, free to withdraw                    |
| `allowance`          | chain only  | USDT allowance granted to the LoanMachine contract            |
| `loan_machine_address` | chain     | Address of the coop's LoanMachine contract                    |

Token amounts are raw `uint256` values as decimal strings (USDT with 6 decimals). The UI formats them via `fmt_usdt` (e.g. `1000000` → `"1 USDT"`).

### 1b. Fetch logic

**File:** `loan_machine_core/src/server_logic/member_financials.rs`

```
get_member_financials_logic(subgraph, blockchain, coop_id_hex, wallet)

1. Parse coop_id_hex → B256
2. Resolve LoanMachine address from CoopRegistry contract
3. Security gate: resolve_member_id
     subgraph first → chain fallback
     returns None if wallet is not a member → WalletNotMember error
4. resolve_member_reputation
     subgraph first → chain fallback
5. getUserFinancials(wallet) on-chain call
     always executed — provides in_coverage, withdrawable, allowance
6. fetch_member_financials from subgraph
     queries latest DonatedEvent, WithdrawnEvent, BorrowedEvent, RepaidEvent
     most-recent timestamp wins when two events cover the same field
     on subgraph error: silently falls back to chain values from step 5
7. Assemble and return MemberFinancials
```

The subgraph query is **best-effort**: if it fails (node down, timeout, schema mismatch) the chain values from `getUserFinancials` are used for donation and borrowing as well. This makes the endpoint resilient to subgraph lag without blocking the user.

### 1c. Security gate

**File:** `loan_machine_web/src/server_fns/member_financials.rs`

The server function checks `auth.is_member(&coop_id)` before calling the logic layer. A non-member wallet receives a generic 404-style error, not the actual financial data. The logic layer also independently verifies membership and returns `WalletNotMember` if the wallet has no on-chain member ID in this coop — this double-gate prevents a misconfigured auth check from leaking data.

### 1d. UI — `MemberStatusPanel`

**File:** `loan_machine_web/src/components/user/member_status.rs`

Shown in two places:
- `CoopRow` (user_coops.rs) — summary row on the user's coop list
- `RoleSection` (coop_control_panel.rs) — displayed for Member, Admin, and Moderator roles

The panel is a `LocalResource` that fetches on mount. It renders a `FinancialsCard` with:
- Member ID (truncated hash)
- Reputation badge (ATIVO / SEM HISTÓRICO / INADIMPLENTE)
- Donation / Em cobertura / Disponível p/ saque grid
- Empréstimos ativos + último empréstimo grid
- USDT allowance row (hidden when zero)
- Status badges: DOADOR, DEVEDOR, SEM MOVIMENTAÇÃO

---

## 2. Donations — (to be documented)

Members donate USDT to the cooperative pool. The contract tracks each donor's balance separately. A portion is locked as coverage for active loans (`in_coverage`); the remainder is free to withdraw.

---

## 3. Loans — (to be documented)

Members can borrow from the cooperative pool up to their allowance. Loans accrue reputation changes on repayment. Overdue loans push reputation negative.

---

## 4. Reputation — (to be documented)

An on-chain signed integer per member per coop. Increases on repayment, decreases on late payment. Used as input to borrow limits and governance weight.
