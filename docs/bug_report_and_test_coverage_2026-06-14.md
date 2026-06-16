# Bug Report & Test Coverage Summary (2026-06-14)

This report was produced by a whole-codebase review (Solidity contracts, Rust
backend core, Rust frontend web) plus a survey of existing automated tests.
Two of the most severe findings from the initial automated review were
re-verified against the actual contract code before inclusion — one
("Critical" accounting double-count) did not hold up, and another ("High" —
cancel an already-funded loan) is blocked by existing code. Both are listed
below as **ruled out** so they aren't re-investigated later.

## Summary by severity (after verification)

- **Critical**: none (the one candidate didn't hold up)
- **High**: none (the one candidate is blocked by existing code)
- **Medium**: 5 — debtWatchlist OOB risk, proposeWalletApproval no-expiry,
  subgraph ordering, contract_errors fragile match, stale-gas-bundle-on-retry
  cluster (donation/withdrawal retry + tab-switch GasModal reset)
- **Low**: 7 — rounding dust, proposeWalletApproval caller check, threshold
  edge case, financials overflow-to-zero, missing client-side amount>0 check,
  redundant approve, fmt_timestamp date math
- **Informational**: 5 — dead code, int32 cast, and three "checked, no bug"
  items

Most actionable if pursuing fixes: **Solidity #3/#4 (Medium)** are the only
items with real architectural weight; everything else is Small/Trivial. The
web "stale gas bundle on retry" cluster (3 findings) likely shares one
root-cause fix in how `GasModal`/`Action` retry is wired.

---

## Solidity (`hardhat/contracts/LoanMachine.sol`)

### Ruled out during review

**~~Critical~~ — `donationsInCoverage`/`availableBalance`/`totalDonations` double-counting**
- Claimed accounting inconsistency across `coverLoan`/`repay`/`withdraw`.
- Verified the invariant `availableBalance == totalDonations - totalBorrowed`
  holds across `donate`, `withdraw`, `coverLoan`, `_fundLoan`, `repay`.
  No double-counting found.

**~~High~~ — `cancelLoanRequisition` on an already-funded loan**
- Location: `cancelLoanRequisition` (L576-610) vs `_fundLoan` (L835-846)
- Claimed a borrower could cancel an already-funded `FullyCovered` loan
  (coverage < 100%) and keep the funds.
- But `_fundLoan` sets `req.status = BorrowStatus.Active` immediately
  (L843), and `cancelLoanRequisition` rejects `status == Active` (L587).
  Cancellation of a funded loan is blocked.

### Medium / Moderate complexity

**`debtWatchlist` / `_removeFromWatchlist` — possible out-of-bounds**
- Location: L624, L881-891
- Potential out-of-bounds on `debtWatchlist.length - 1` if the list is empty
  when removal is attempted. Needs a path-trace to confirm reachability, but
  worth a defensive guard.

**`proposeWalletApproval` — no expiry/cancellation**
- Location: L343-354, L424-428
- No expiry or cancellation mechanism for pending wallet-approval proposals —
  a stale/abandoned proposal can block or confuse later approval flows
  (DoS-ish, not exploitable for fund loss).

### Low

**Rounding dust in `coverLoan` / repay distribution** (Trivial)
- Location: L766-769 (coverLoan), L863-879 (repay distribution)
- `coverAmount` rounds **up** (ceiling division), so the sum of lenders'
  locked `donationsInCoverage` can exceed `req.amount` by a few wei of
  "dust" that's never reclaimed. Cosmetic, no fund-safety impact given the
  pool-level invariant holds.

**`proposeWalletApproval` callable by anyone?** (Small, needs confirmation)
- Location: ~L340
- Function doesn't appear to restrict the caller to existing admins —
  confirm whether `validMember`/admin modifier is actually missing or just
  not visible in the excerpt reviewed.

**Threshold-change edge case** (Small, needs confirmation)
- Location: L391-401, L439-444
- Possible edge case when the approval threshold changes mid-flight with
  pending proposals; needs a concrete repro.

### Informational

- **`joinCoop` (L525)** — dead/leftover comment. (Trivial)
- **`getElectionInfo` (L948)** — `int32` cast of election id; fine while ids
  stay small, but worth a comment noting the bound. (Trivial)
- **`LoanCalculations.sol` / `LoanStructs.sol`** — appear to be dead code
  (unused by `LoanMachine.sol`); candidates for deletion to reduce surface
  area, not a bug per se.

---

## Rust backend core (`loan_machine_core/src/`)

### Medium

**Subgraph query missing ordering** (Small, needs confirmation)
- Location: `helpers/resolve_member_id.rs` (L30-55)
- Query has no `orderBy`/`orderDirection`, so if a member ID has multiple
  historical registration events, the "current" one returned could be stale
  rather than the latest.

**Fragile revert-data extraction** (Moderate)
- Location: `services/blockchain/contract_errors.rs` (L85-101)
- `extract_revert_data` scrapes the `Debug` string for a literal
  `"RawValue(\"0x"` substring — fragile, and may never match real `alloy`
  error shapes, potentially making `translate_revert`'s nice error messages
  dead code in practice. Worth a unit test against a real revert to confirm
  it actually fires.

### Low

**Overflow silently mapped to zero** (Trivial)
- Location: `member_financials.rs` (L67, L72)
- `try_into().unwrap_or(0)` silently maps an overflow to `0` instead of
  erroring — could mask a real overflow as "zero balance".

**Missing amount > 0 validation before calldata** (Trivial)
- Location: `donation.rs` (L40), `withdrawal.rs` (L36)
- No `amount > 0` validation before building calldata — relies on the
  contract's `validAmount` revert. Fine functionally, but means a `0` amount
  round-trips to the chain just to fail there (extra RPC + gas estimate).

**Always re-approves ERC-20 allowance** (Small)
- Location: `donation.rs` (L67-70)
- Always issues a fresh `approve` rather than checking existing allowance
  first — extra gas/tx for repeat donors.

**O(n) registry scan fallback** (Moderate, needs confirmation)
- Location: `helpers/resolve_wallet_coop.rs` (L85-111)
- Fallback path does a sequential scan over the registry when the primary
  lookup misses — fine at small scale, could become slow as coops grow.
  Confirm this path is actually hit in practice before prioritizing.

### Informational

**TOCTOU between balance check and on-chain execution** (architectural, no fix)
- Location: `withdrawal.rs` (L52-65)
- Inherent gap between the server-side balance check and on-chain execution
  — the chain re-validates via `validAmount`/`getWithdrawableBalance`, so
  this is a UX issue (stale quote) not a fund-safety issue.

---

## Rust frontend web (`loan_machine_web/src/`)

### Medium

**Stale prepared bundle on re-trigger** (Small, needs confirmation)
- Location: `server_fns/donation.rs` (L59-77, 121-126)
- If a user re-triggers donation after a prior prepare, the UI may reuse a
  stale prepared bundle (old amount/allowance) instead of re-preparing —
  could show the wrong gas estimate. Needs a repro to confirm whether
  `Action` re-invocation actually re-prepares.

**Stale bundle on donation retry** (Small)
- Location: `server_fns/donation.rs` (L59-77, L168)
- `on_retry` reopens `GasModal` with the previously prepared bundle without
  a fresh `prepare` call — same stale-bundle risk as above, specifically on
  the retry path.

**Stale bundle on withdrawal retry** (Small)
- Location: `server_fns/withdrawal.rs` (L56-74, L141)
- Same stale-bundle-on-retry issue as donation, withdrawal side.

**GasModal not reset on tab switch** (Moderate)
- Location: `components/transfer_panel.rs` (L45-52)
- Switching tabs mid-transaction doesn't close/reset the global `GasModal`
  singleton — could leave stale signal references bound to the previous
  tab's action, risking a panic or wrong-bundle submission if the modal is
  then confirmed.

### Low

**Non-calendar date math in `fmt_timestamp`** (Small)
- Location: `components/user/member_status.rs` (L128-138)
- Assumes fixed days/month — cosmetic display drift for dates far from
  epoch, no functional impact.

### Informational (checked, no bug)

- **`exceeds_max` edge cases** (e.g. `"1000000."`) — handled correctly.
- **`usdt_to_raw` zero-amount path** — submit button correctly disabled for
  zero.
- **All fund-touching `server_fns`** — correctly enforce
  `Authenticated::require()` + `is_member` + server-resolved wallet (never
  trust client-supplied wallet/coop id).
- **`coop_control_panel.rs` (L48-65)** — `coop_id` from route params is
  validated server-side downstream.

---

# Test Coverage Résumé

## Solidity (`hardhat/`)

- **No test suite exists.** `hardhat/test/` contains only ad-hoc
  event-listener scripts, not actual tests (no Hardhat/Foundry assertions).
  All contract-level correctness is currently validated only indirectly,
  through the Rust integration tests below.

## `loan_machine_core/tests/` — 19 integration test files, full anvil-backed fixture

Shared infra: `deploy.rs` (per-binary `OnceCell<Deployed>` anvil + contract
deployment), `elections.rs`, `proposals.rs`, `financials.rs` (new),
`payloads.rs`, `subgraph_mock.rs`, with `platform_admin_lock` /
`second_admin_lock` for serializing on-chain mutations within a binary.

**Recently added**:
- `server_logic_coop_financials_test.rs` — coop-wide financial totals
- `server_logic_donation_test.rs` — donation logic happy/error paths
- `server_logic_withdrawal_test.rs` — 8 tests covering member gating, invalid
  amount/coop id, no-donation withdrawal, full donate→withdraw cycle with
  before/after financial assertions, cross-wallet isolation (admin1 donates,
  admin2 can't see/withdraw it), and extreme values (`0` → on-chain revert
  "Invalid amount.", `withdrawable+1` → `InsufficientBalance`)

**Other existing files cover**: elections/moderator voting, member
vinculation, wallet approval proposals, identity resolution, reputation, coop
view/approval flows, create-coop, member financials.

**Gaps**:
- `coop_view.rs`, `coop_approval.rs`, `create_coop.rs` (direct),
  `wallet_balance.rs`, several `subgraph_queries/*` modules — no dedicated
  test file (only indirect coverage via other tests)
- `helpers/resolve_*` modules — only indirectly exercised

## `loan_machine_web/src/` — 16 test files/modules

Includes the new `ui.rs::amount_input_tests` (16 tests for
`is_valid_amount_input` / `usdt_to_raw` / `exceeds_max`: empty input, plain
integers, decimals up to 6 places, >6 fraction digits, multiple dots, letters,
symbols, exactly-1,000,000, above-1,000,000, fraction overflow, huge numbers —
all passing), plus `donation/withdrawal_test.rs`, `coop_financials_test.rs`,
etc.

**Gaps**:
- `server_fns/coop_financials.rs` and `server_fns/donation.rs` have **no
  web-side tests**
- `GasModal` open/active states untested
- `coop_financials.rs` populated-data rendering untested (only loading state
  covered)
- `donation_test` / `withdrawal_test` (web) only cover static initial render,
  not interaction/submission flows

## Totals

- Web test suite: **154 tests passing** (up from 138 before this session's
  additions)
- Core integration tests: 19 files, all passing (verified serially and in
  parallel)
- Solidity: **0** automated tests
