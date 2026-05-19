import { BigInt, Bytes, ethereum } from "@graphprotocol/graph-ts"

import {
  ProposalCreated, ProposalConfirmed, ProposalExecuted,
  AdminAdded, AdminRemoved, AdminTransferred, ThresholdChanged,
  ProposalCosigned, WalletApproved, WalletRevoked,
  MemberRegistered, AccessCodeRotated, CoopDeactivated, CoopReactivated,
  WithdrawalRequested, WithdrawalExecuted, WithdrawalBlocked, WithdrawalCancelled,
  Donated, Withdrawn, Borrowed, Repaid,
  TotalDonationsUpdated, TotalBorrowedUpdated, AvailableBalanceUpdated, NewDonor,
  LoanRequisitionCreatedCancelled, LoanCovered, LoanFunded, LoanContractGenerated,
  ParcelPaid, LenderRepaid, LoanCompleted, LoanUncovered,
  BorrowerOverdue, BorrowerDebtSettled,
  ReputationChanged, AuthorizedCallerUpdated,
  ElectionOpened, CandidateAdded, VoteCast, ElectionClosed,
  UnbeatableMajorityReached, NewModerator
} from "../generated/templates/LoanMachine/LoanMachine"

import {
  Cooperative,
  ProposalCreatedEvent, ProposalConfirmedEvent, ProposalExecutedEvent,
  AdminAddedEvent, AdminRemovedEvent, AdminTransferredEvent, ThresholdChangedEvent,
  ProposalCosignedEvent, WalletApprovedEvent, WalletRevokedEvent,
  MemberRegisteredEvent, AccessCodeRotatedEvent, CoopDeactivatedEvent, CoopReactivatedEvent,
  WithdrawalRequestedEvent, WithdrawalExecutedEvent, WithdrawalBlockedEvent, WithdrawalCancelledEvent,
  DonatedEvent, WithdrawnEvent, BorrowedEvent, RepaidEvent,
  TotalDonationsUpdatedEvent, TotalBorrowedUpdatedEvent, AvailableBalanceUpdatedEvent, NewDonorEvent,
  LoanRequisitionCreatedCancelledEvent, LoanCoveredEvent, LoanFundedEvent,
  LoanContractGeneratedEvent, ParcelPaidEvent, LenderRepaidEvent,
  LoanCompletedEvent, LoanUncoveredEvent, BorrowerOverdueEvent, BorrowerDebtSettledEvent,
  ReputationChangedEvent, AuthorizedCallerUpdatedEvent,
  ElectionOpenedEvent, CandidateAddedEvent, VoteCastEvent, ElectionClosedEvent,
  UnbeatableMajorityReachedEvent, NewModeratorEvent
} from "../generated/schema"

// ── HELPERS ──────────────────────────────────────────────────

function makeId(event: ethereum.Event): string {
  return event.transaction.hash.toHex() + "-" + event.logIndex.toString()
}

function coopId(event: ethereum.Event): string {
  // Cooperative.id == lowercased LoanMachine address (set in coop-registry.ts)
  return event.address.toHexString()
}

function formatTimestamp(ts: BigInt): string {
  let ms = ts.times(BigInt.fromI32(1000)).toI64()
  let dt = new Date(ms)
  let d  = dt.getUTCDate().toString();        if (d.length  == 1) d  = "0" + d
  let m  = (dt.getUTCMonth() + 1).toString(); if (m.length  == 1) m  = "0" + m
  let h  = dt.getUTCHours().toString();       if (h.length  == 1) h  = "0" + h
  let mi = dt.getUTCMinutes().toString();     if (mi.length == 1) mi = "0" + mi
  let s  = dt.getUTCSeconds().toString();     if (s.length  == 1) s  = "0" + s
  return d + "/" + m + "/" + dt.getUTCFullYear().toString() + " " + h + ":" + mi + ":" + s
}

// ═════════════════════════════════════════════════════════════
//                     GOVERNANCE
// ═════════════════════════════════════════════════════════════

export function handleProposalCreated(event: ProposalCreated): void {
  let entity = new ProposalCreatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.proposalId      = event.params.proposalId
  entity.pType           = event.params.pType
  entity.proposer        = event.params.proposer
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleProposalConfirmed(event: ProposalConfirmed): void {
  let entity = new ProposalConfirmedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.proposalId      = event.params.proposalId
  entity.admin           = event.params.admin
  entity.confirmations   = event.params.confirmations
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleProposalExecuted(event: ProposalExecuted): void {
  let entity = new ProposalExecutedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.proposalId      = event.params.proposalId
  entity.pType           = event.params.pType
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAdminAdded(event: AdminAdded): void {
  let entity = new AdminAddedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.admin           = event.params.admin
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAdminRemoved(event: AdminRemoved): void {
  let entity = new AdminRemovedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.admin           = event.params.admin
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAdminTransferred(event: AdminTransferred): void {
  let entity = new AdminTransferredEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.oldAdmin        = event.params.oldAdmin
  entity.newAdmin        = event.params.newAdmin
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleThresholdChanged(event: ThresholdChanged): void {
  let entity = new ThresholdChangedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.oldThreshold    = event.params.oldThreshold
  entity.newThreshold    = event.params.newThreshold
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                  WALLET APPROVAL
// ═════════════════════════════════════════════════════════════

export function handleProposalCosigned(event: ProposalCosigned): void {
  let entity = new ProposalCosignedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.proposalId      = event.params.proposalId
  entity.moderatorId     = event.params.moderatorMemberId
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWalletApproved(event: WalletApproved): void {
  let entity = new WalletApprovedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.wallet          = event.params.wallet
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWalletRevoked(event: WalletRevoked): void {
  let entity = new WalletRevokedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.wallet          = event.params.wallet
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                     MEMBERSHIP
// ═════════════════════════════════════════════════════════════

export function handleMemberRegistered(event: MemberRegistered): void {
  let entity = new MemberRegisteredEvent(makeId(event))
  entity.cooperative   = coopId(event)
  entity.memberId      = event.params.memberId
  entity.wallet        = event.params.wallet
  entity.isFirstWallet = event.params.isFirstWallet

  let wallets: Bytes[] = []
  for (let i = 0; i < event.params.allWallets.length; i++) {
    wallets.push(event.params.allWallets[i] as Bytes)
  }
  entity.allWallets = wallets

  entity.timestamp       = formatTimestamp(event.params.timestamp)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAccessCodeRotated(event: AccessCodeRotated): void {
  let entity = new AccessCodeRotatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleCoopDeactivated(event: CoopDeactivated): void {
  let entity = new CoopDeactivatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()

  // Update the live Cooperative status flag
  let coop = Cooperative.load(coopId(event))
  if (coop != null) {
    coop.active = false
    coop.save()
  }
}

export function handleCoopReactivated(event: CoopReactivated): void {
  let entity = new CoopReactivatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()

  let coop = Cooperative.load(coopId(event))
  if (coop != null) {
    coop.active = true
    coop.save()
  }
}

// ═════════════════════════════════════════════════════════════
//                  WITHDRAWAL DELAY
// ═════════════════════════════════════════════════════════════

export function handleWithdrawalRequested(event: WithdrawalRequested): void {
  let entity = new WithdrawalRequestedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requestId       = event.params.requestId
  entity.requester       = event.params.requester
  entity.amount          = event.params.amount
  entity.executableAfter = event.params.executableAfter
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawalExecuted(event: WithdrawalExecuted): void {
  let entity = new WithdrawalExecutedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requestId       = event.params.requestId
  entity.requester       = event.params.requester
  entity.amount          = event.params.amount
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawalBlocked(event: WithdrawalBlocked): void {
  let entity = new WithdrawalBlockedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requestId       = event.params.requestId
  entity.blocker         = event.params.blocker
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawalCancelled(event: WithdrawalCancelled): void {
  let entity = new WithdrawalCancelledEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requestId       = event.params.requestId
  entity.requester       = event.params.requester
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                     FINANCIAL
// ═════════════════════════════════════════════════════════════

export function handleDonated(event: Donated): void {
  let entity = new DonatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.donor           = event.params.donor
  entity.amount          = event.params.amount
  entity.totalDonation   = event.params.totalDonation
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawn(event: Withdrawn): void {
  let entity = new WithdrawnEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.donor           = event.params.donor
  entity.amount          = event.params.amount
  entity.donations       = event.params.donations
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowed(event: Borrowed): void {
  let entity = new BorrowedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.borrower        = event.params.borrower
  entity.amount          = event.params.amount
  entity.totalBorrowing  = event.params.totalBorrowing
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleRepaid(event: Repaid): void {
  let entity = new RepaidEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.borrower        = event.params.borrower
  entity.amount          = event.params.amount
  entity.remainingDebt   = event.params.remainingDebt
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleTotalDonationsUpdated(event: TotalDonationsUpdated): void {
  let entity = new TotalDonationsUpdatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.total           = event.params.total
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleTotalBorrowedUpdated(event: TotalBorrowedUpdated): void {
  let entity = new TotalBorrowedUpdatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.total           = event.params.total
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAvailableBalanceUpdated(event: AvailableBalanceUpdated): void {
  let entity = new AvailableBalanceUpdatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.total           = event.params.total
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewDonor(event: NewDonor): void {
  let entity = new NewDonorEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.donor           = event.params.donor
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                   LOAN LIFECYCLE
// ═════════════════════════════════════════════════════════════

export function handleLoanRequisitionCreatedCancelled(event: LoanRequisitionCreatedCancelled): void {
  // ID must be namespaced — different coops both have requisitionId=1.
  let id = coopId(event) + "-" + event.params.requisitionId.toString()
  let entity = LoanRequisitionCreatedCancelledEvent.load(id)

  if (entity == null) {
    entity = new LoanRequisitionCreatedCancelledEvent(id)
    entity.cooperative     = coopId(event)
    entity.requisitionId   = event.params.requisitionId
    entity.borrower        = event.params.borrower
    entity.amount          = event.params.amount
    entity.parcelsCount    = event.params.parcelsCount.toI32()
  }
  entity.status          = event.params.status
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanCovered(event: LoanCovered): void {
  let entity = new LoanCoveredEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requisitionId   = event.params.requisitionId
  entity.lender          = event.params.lender
  entity.coverageAmount  = event.params.coverageAmount
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanFunded(event: LoanFunded): void {
  let entity = new LoanFundedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requisitionId   = event.params.requisitionId
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanContractGenerated(event: LoanContractGenerated): void {
  let entity = new LoanContractGeneratedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.walletAddress   = event.params.walletAddress
  entity.requisitionId   = event.params.requisitionId
  entity.status          = event.params.status
  entity.parcelsPending  = event.params.parcelsPending.toI32()
  entity.parcelsValues   = event.params.parcelsValues

  let paymentDatesArr = new Array<string>()
  for (let i = 0; i < event.params.paymentDates.length; i++) {
    paymentDatesArr.push(formatTimestamp(event.params.paymentDates[i]))
  }
  entity.paymentDates    = paymentDatesArr
  entity.creationTime    = formatTimestamp(event.params.creationTime)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleParcelPaid(event: ParcelPaid): void {
  let entity = new ParcelPaidEvent(makeId(event))
  entity.cooperative      = coopId(event)
  entity.requisitionId    = event.params.requisitionId
  entity.parcelsRemaining = event.params.parcelsRemaining
  entity.blockTimestamp   = formatTimestamp(event.block.timestamp)
  entity.transactionHash  = event.transaction.hash
  entity.save()
}

export function handleLenderRepaid(event: LenderRepaid): void {
  let entity = new LenderRepaidEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requisitionId   = event.params.requisitionId
  entity.lender          = event.params.lender
  entity.amount          = event.params.amount
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanCompleted(event: LoanCompleted): void {
  let entity = new LoanCompletedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requisitionId   = event.params.requisitionId
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanUncovered(event: LoanUncovered): void {
  let entity = new LoanUncoveredEvent(makeId(event))
  entity.cooperative            = coopId(event)
  entity.requisitionId          = event.params.requisitionId
  entity.lender                 = event.params.lender
  entity.amountReturnedToLender = event.params.amountReturnedToLender
  entity.blockTimestamp         = formatTimestamp(event.block.timestamp)
  entity.transactionHash        = event.transaction.hash
  entity.save()
}

export function handleBorrowerOverdue(event: BorrowerOverdue): void {
  let entity = new BorrowerOverdueEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requisitionId   = event.params.requisitionId
  entity.borrower        = event.params.borrower
  entity.dueDate         = event.params.dueDate
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowerDebtSettled(event: BorrowerDebtSettled): void {
  let entity = new BorrowerDebtSettledEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.requisitionId   = event.params.requisitionId
  entity.borrower        = event.params.borrower
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                     REPUTATION
// ═════════════════════════════════════════════════════════════

export function handleReputationChanged(event: ReputationChanged): void {
  let entity = new ReputationChangedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.memberId        = event.params.memberId
  entity.points          = event.params.points
  entity.increase        = event.params.increase
  entity.newReputation   = event.params.newReputation
  entity.timestamp       = formatTimestamp(event.params.timestamp)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAuthorizedCallerUpdated(event: AuthorizedCallerUpdated): void {
  let entity = new AuthorizedCallerUpdatedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.caller          = event.params.caller
  entity.authorized      = event.params.authorized
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                     ELECTIONS
// ═════════════════════════════════════════════════════════════

export function handleElectionOpened(event: ElectionOpened): void {
  let entity = new ElectionOpenedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.electionId      = event.params.electionId.toI32()
  entity.candidateId     = event.params.candidateId
  entity.startTime       = formatTimestamp(event.params.startTime)
  entity.endTime         = formatTimestamp(event.params.endTime)
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleCandidateAdded(event: CandidateAdded): void {
  let entity = new CandidateAddedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.electionId      = event.params.electionId.toI32()
  entity.candidateId     = event.params.candidateId
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleVoteCast(event: VoteCast): void {
  let entity = new VoteCastEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.electionId      = event.params.electionId.toI32()
  entity.candidateId     = event.params.candidateId
  entity.memberId        = event.params.memberId
  entity.voteWeight      = event.params.voteWeight
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleElectionClosed(event: ElectionClosed): void {
  let entity = new ElectionClosedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.electionId      = event.params.electionId.toI32()
  entity.winnerId        = event.params.winnerId
  entity.winningVotes    = event.params.winningVotes
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleUnbeatableMajorityReached(event: UnbeatableMajorityReached): void {
  let entity = new UnbeatableMajorityReachedEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.electionId      = event.params.electionId.toI32()
  entity.winnerId        = event.params.winnerId
  entity.winningVotes    = event.params.winningVotes
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewModerator(event: NewModerator): void {
  let entity = new NewModeratorEvent(makeId(event))
  entity.cooperative     = coopId(event)
  entity.memberId        = event.params.memberId
  entity.electionId      = event.params.electionId.toI32()
  entity.blockTimestamp  = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}