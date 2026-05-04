import { BigInt, Bytes } from "@graphprotocol/graph-ts"

// ── All events now come from a single LoanMachine contract ───
// (ReputationLib is inlined — no separate data source needed)
import {
  // Governance
  ProposalCreated,
  ProposalConfirmed,
  ProposalExecuted,
  AdminAdded,
  AdminRemoved,
  AdminTransferred,
  ThresholdChanged,
  // Wallet approval
  ProposalCosigned,
  WalletApproved,
  WalletRevoked,
  // Membership
  MemberJoined,
  AccessCodeRotated,
  CoopDeactivated,
  CoopReactivated,
  // Withdrawal
  WithdrawalRequested,
  WithdrawalExecuted,
  WithdrawalBlocked,
  WithdrawalCancelled,
  // Financial
  Donated,
  Withdrawn,
  Borrowed,
  Repaid,
  TotalDonationsUpdated,
  TotalBorrowedUpdated,
  AvailableBalanceUpdated,
  NewDonor,
  // Loan lifecycle
  LoanRequisitionCreatedCancelled,
  LoanCovered,
  LoanFunded,
  LoanContractGenerated,
  ParcelPaid,
  LenderRepaid,
  LoanCompleted,
  LoanUncovered,
  BorrowerOverdue,
  BorrowerDebtSettled,
  // Reputation (inlined from ReputationLib)
  MemberToWalletVinculation,
  ReputationChanged,
  AuthorizedCallerUpdated,
  // Elections (inlined from ReputationLib)
  ElectionOpened,
  CandidateAdded,
  VoteCast,
  ElectionClosed,
  UnbeatableMajorityReached,
  NewModerator
} from "./generated/LoanMachine/LoanMachine"

import {
  // Governance
  ProposalCreatedEvent,
  ProposalConfirmedEvent,
  ProposalExecutedEvent,
  AdminAddedEvent,
  AdminRemovedEvent,
  AdminTransferredEvent,
  ThresholdChangedEvent,
  // Wallet approval
  ProposalCosignedEvent,
  WalletApprovedEvent,
  WalletRevokedEvent,
  // Membership
  MemberJoinedEvent,
  AccessCodeRotatedEvent,
  CoopDeactivatedEvent,
  CoopReactivatedEvent,
  // Withdrawal
  WithdrawalRequestedEvent,
  WithdrawalExecutedEvent,
  WithdrawalBlockedEvent,
  WithdrawalCancelledEvent,
  // Financial
  DonatedEvent,
  WithdrawnEvent,
  BorrowedEvent,
  RepaidEvent,
  TotalDonationsUpdatedEvent,
  TotalBorrowedUpdatedEvent,
  AvailableBalanceUpdatedEvent,
  NewDonorEvent,
  // Loan lifecycle
  LoanRequisitionCreatedCancelledEvent,
  LoanCoveredEvent,
  LoanFundedEvent,
  LoanContractGeneratedEvent,
  ParcelPaidEvent,
  LenderRepaidEvent,
  LoanCompletedEvent,
  LoanUncoveredEvent,
  BorrowerOverdueEvent,
  BorrowerDebtSettledEvent,
  // Reputation
  MemberToWalletVinculationEvent,
  ReputationChangedEvent,
  AuthorizedCallerUpdatedEvent,
  // Elections
  ElectionOpenedEvent,
  CandidateAddedEvent,
  VoteCastEvent,
  ElectionClosedEvent,
  UnbeatableMajorityReachedEvent,
  NewModeratorEvent
} from "./generated/schema"

// ── HELPERS ──────────────────────────────────────────────────
import { ethereum } from "@graphprotocol/graph-ts"
function makeId(event: ethereum.Event): string {
  return event.transaction.hash.toHex() + "-" + event.logIndex.toString()
}

function formatTimestamp(ts: BigInt): string {
  let ms = ts.times(BigInt.fromI32(1000)).toI64()
  let dt = new Date(ms)
  let d = dt.getUTCDate().toString()
  let m = (dt.getUTCMonth() + 1).toString()
  let y = dt.getUTCFullYear().toString()
  let h = dt.getUTCHours().toString()
  let mi = dt.getUTCMinutes().toString()
  let s = dt.getUTCSeconds().toString()
  if (d.length == 1) d = "0" + d
  if (m.length == 1) m = "0" + m
  if (h.length == 1) h = "0" + h
  if (mi.length == 1) mi = "0" + mi
  if (s.length == 1) s = "0" + s
  return d + "/" + m + "/" + y + " " + h + ":" + mi + ":" + s
}

// ═════════════════════════════════════════════════════════════
//                     GOVERNANCE HANDLERS
// ═════════════════════════════════════════════════════════════

export function handleProposalCreated(event: ProposalCreated): void {
  let entity = new ProposalCreatedEvent(makeId(event))
  entity.proposalId = event.params.proposalId
  entity.pType = event.params.pType
  entity.proposer = event.params.proposer
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleProposalConfirmed(event: ProposalConfirmed): void {
  let entity = new ProposalConfirmedEvent(makeId(event))
  entity.proposalId = event.params.proposalId
  entity.admin = event.params.admin
  entity.confirmations = event.params.confirmations
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleProposalExecuted(event: ProposalExecuted): void {
  let entity = new ProposalExecutedEvent(makeId(event))
  entity.proposalId = event.params.proposalId
  entity.pType = event.params.pType
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAdminAdded(event: AdminAdded): void {
  let entity = new AdminAddedEvent(makeId(event))
  entity.admin = event.params.admin
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAdminRemoved(event: AdminRemoved): void {
  let entity = new AdminRemovedEvent(makeId(event))
  entity.admin = event.params.admin
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAdminTransferred(event: AdminTransferred): void {
  let entity = new AdminTransferredEvent(makeId(event))
  entity.oldAdmin = event.params.oldAdmin
  entity.newAdmin = event.params.newAdmin
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleThresholdChanged(event: ThresholdChanged): void {
  let entity = new ThresholdChangedEvent(makeId(event))
  entity.oldThreshold = event.params.oldThreshold
  entity.newThreshold = event.params.newThreshold
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                  WALLET APPROVAL HANDLERS
// ═════════════════════════════════════════════════════════════

export function handleProposalCosigned(event: ProposalCosigned): void {
  let entity = new ProposalCosignedEvent(makeId(event))
  entity.proposalId = event.params.proposalId
  entity.moderatorId = event.params.moderatorMemberId
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWalletApproved(event: WalletApproved): void {
  let entity = new WalletApprovedEvent(makeId(event))
  entity.wallet = event.params.wallet
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWalletRevoked(event: WalletRevoked): void {
  let entity = new WalletRevokedEvent(makeId(event))
  entity.wallet = event.params.wallet
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                     MEMBERSHIP HANDLERS
// ═════════════════════════════════════════════════════════════

export function handleMemberJoined(event: MemberJoined): void {
  let entity = new MemberJoinedEvent(makeId(event))
  entity.wallet = event.params.wallet
  entity.memberId = event.params.memberId
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAccessCodeRotated(event: AccessCodeRotated): void {
  let entity = new AccessCodeRotatedEvent(makeId(event))
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleCoopDeactivated(event: CoopDeactivated): void {
  let entity = new CoopDeactivatedEvent(makeId(event))
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleCoopReactivated(event: CoopReactivated): void {
  let entity = new CoopReactivatedEvent(makeId(event))
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                  WITHDRAWAL DELAY HANDLERS
// ═════════════════════════════════════════════════════════════

export function handleWithdrawalRequested(event: WithdrawalRequested): void {
  let entity = new WithdrawalRequestedEvent(makeId(event))
  entity.requestId = event.params.requestId
  entity.requester = event.params.requester
  entity.amount = event.params.amount
  entity.executableAfter = event.params.executableAfter
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawalExecuted(event: WithdrawalExecuted): void {
  let entity = new WithdrawalExecutedEvent(makeId(event))
  entity.requestId = event.params.requestId
  entity.requester = event.params.requester
  entity.amount = event.params.amount
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawalBlocked(event: WithdrawalBlocked): void {
  let entity = new WithdrawalBlockedEvent(makeId(event))
  entity.requestId = event.params.requestId
  entity.blocker = event.params.blocker
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawalCancelled(event: WithdrawalCancelled): void {
  let entity = new WithdrawalCancelledEvent(makeId(event))
  entity.requestId = event.params.requestId
  entity.requester = event.params.requester
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                     FINANCIAL HANDLERS
// ═════════════════════════════════════════════════════════════

export function handleDonated(event: Donated): void {
  let entity = new DonatedEvent(makeId(event))
  entity.donor = event.params.donor
  entity.amount = event.params.amount
  entity.totalDonation = event.params.totalDonation
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleWithdrawn(event: Withdrawn): void {
  let entity = new WithdrawnEvent(makeId(event))
  entity.donor = event.params.donor
  entity.amount = event.params.amount
  entity.donations = event.params.donations
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowed(event: Borrowed): void {
  let entity = new BorrowedEvent(makeId(event))
  entity.borrower = event.params.borrower
  entity.amount = event.params.amount
  entity.totalBorrowing = event.params.totalBorrowing
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleRepaid(event: Repaid): void {
  let entity = new RepaidEvent(makeId(event))
  entity.borrower = event.params.borrower
  entity.amount = event.params.amount
  entity.remainingDebt = event.params.remainingDebt
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleTotalDonationsUpdated(event: TotalDonationsUpdated): void {
  let entity = new TotalDonationsUpdatedEvent(makeId(event))
  entity.total = event.params.total
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleTotalBorrowedUpdated(event: TotalBorrowedUpdated): void {
  let entity = new TotalBorrowedUpdatedEvent(makeId(event))
  entity.total = event.params.total
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAvailableBalanceUpdated(event: AvailableBalanceUpdated): void {
  let entity = new AvailableBalanceUpdatedEvent(makeId(event))
  entity.total = event.params.total
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewDonor(event: NewDonor): void {
  let entity = new NewDonorEvent(makeId(event))
  entity.donor = event.params.donor
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//                   LOAN LIFECYCLE HANDLERS
// ═════════════════════════════════════════════════════════════

export function handleLoanRequisitionCreatedCancelled(event: LoanRequisitionCreatedCancelled): void {
  let statusId = event.params.requisitionId.toString()
  let entity = LoanRequisitionCreatedCancelledEvent.load(statusId)

  if (entity == null) {
    entity = new LoanRequisitionCreatedCancelledEvent(statusId)
    entity.requisitionId = event.params.requisitionId
    entity.borrower = event.params.borrower
    entity.amount = event.params.amount
    entity.parcelsCount = event.params.parcelsCount.toI32()
    entity.status = event.params.status
    entity.blockTimestamp = formatTimestamp(event.block.timestamp)
    entity.transactionHash = event.transaction.hash
  } else {
    entity.status = event.params.status
    entity.blockTimestamp = formatTimestamp(event.block.timestamp)
    entity.transactionHash = event.transaction.hash
  }

  entity.save()
}

export function handleLoanCovered(event: LoanCovered): void {
  let entity = new LoanCoveredEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.lender = event.params.lender
  entity.coverageAmount = event.params.coverageAmount
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanFunded(event: LoanFunded): void {
  let entity = new LoanFundedEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanContractGenerated(event: LoanContractGenerated): void {
  let entity = new LoanContractGeneratedEvent(makeId(event))
  entity.walletAddress = event.params.walletAddress
  entity.requisitionId = event.params.requisitionId
  entity.status = event.params.status
  entity.parcelsPending = event.params.parcelsPending.toI32()
  entity.parcelsValues = event.params.parcelsValues

  let paymentDatesArr = new Array<string>()
  for (let i = 0; i < event.params.paymentDates.length; i++) {
    paymentDatesArr.push(formatTimestamp(event.params.paymentDates[i]))
  }
  entity.paymentDates = paymentDatesArr
  entity.creationTime = formatTimestamp(event.params.creationTime)

  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleParcelPaid(event: ParcelPaid): void {
  let entity = new ParcelPaidEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.parcelsRemaining = event.params.parcelsRemaining
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLenderRepaid(event: LenderRepaid): void {
  let entity = new LenderRepaidEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.lender = event.params.lender
  entity.amount = event.params.amount
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanCompleted(event: LoanCompleted): void {
  let entity = new LoanCompletedEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanUncovered(event: LoanUncovered): void {
  let entity = new LoanUncoveredEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.lender = event.params.lender
  entity.amountReturnedToLender = event.params.amountReturnedToLender
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowerOverdue(event: BorrowerOverdue): void {
  let entity = new BorrowerOverdueEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.borrower = event.params.borrower
  entity.dueDate = event.params.dueDate
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowerDebtSettled(event: BorrowerDebtSettled): void {
  let entity = new BorrowerDebtSettledEvent(makeId(event))
  entity.requisitionId = event.params.requisitionId
  entity.borrower = event.params.borrower
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//               REPUTATION HANDLERS (bytes32 memberId)
// ═════════════════════════════════════════════════════════════

export function handleMemberToWalletVinculation(event: MemberToWalletVinculation): void {
  let entity = new MemberToWalletVinculationEvent(makeId(event))
  entity.memberId = event.params.memberId       // bytes32 now
  entity.wallet = event.params.wallet

  let walletVinculatedBytes: Bytes[] = []
  for (let i = 0; i < event.params.walletVinculated.length; i++) {
    walletVinculatedBytes.push(event.params.walletVinculated[i] as Bytes)
  }
  entity.walletVinculated = walletVinculatedBytes

  entity.timestamp = formatTimestamp(event.params.timestamp)
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleReputationChanged(event: ReputationChanged): void {
  let entity = new ReputationChangedEvent(makeId(event))
  entity.memberId = event.params.memberId        // bytes32 now
  entity.points = event.params.points
  entity.increase = event.params.increase
  entity.newReputation = event.params.newReputation
  entity.timestamp = formatTimestamp(event.params.timestamp)
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAuthorizedCallerUpdated(event: AuthorizedCallerUpdated): void {
  let entity = new AuthorizedCallerUpdatedEvent(makeId(event))
  entity.caller = event.params.caller
  entity.authorized = event.params.authorized
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

// ═════════════════════════════════════════════════════════════
//              ELECTION HANDLERS (bytes32 candidateId/memberId)
// ═════════════════════════════════════════════════════════════

export function handleElectionOpened(event: ElectionOpened): void {
  let entity = new ElectionOpenedEvent(makeId(event))
  entity.electionId = event.params.electionId.toI32()
  entity.candidateId = event.params.candidateId   // bytes32 now
  entity.startTime = formatTimestamp(event.params.startTime)
  entity.endTime = formatTimestamp(event.params.endTime)
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleCandidateAdded(event: CandidateAdded): void {
  let entity = new CandidateAddedEvent(makeId(event))
  entity.electionId = event.params.electionId.toI32()
  entity.candidateId = event.params.candidateId   // bytes32 now
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleVoteCast(event: VoteCast): void {
  let entity = new VoteCastEvent(makeId(event))
  entity.electionId = event.params.electionId.toI32()
  entity.candidateId = event.params.candidateId   // bytes32 now
  entity.memberId = event.params.memberId          // bytes32 now
  entity.voteWeight = event.params.voteWeight
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleElectionClosed(event: ElectionClosed): void {
  let entity = new ElectionClosedEvent(makeId(event))
  entity.electionId = event.params.electionId.toI32()
  entity.winnerId = event.params.winnerId          // bytes32 now
  entity.winningVotes = event.params.winningVotes
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleUnbeatableMajorityReached(event: UnbeatableMajorityReached): void {
  let entity = new UnbeatableMajorityReachedEvent(makeId(event))
  entity.electionId = event.params.electionId.toI32()
  entity.winnerId = event.params.winnerId          // bytes32 now
  entity.winningVotes = event.params.winningVotes
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewModerator(event: NewModerator): void {
  let entity = new NewModeratorEvent(makeId(event))
  entity.memberId = event.params.memberId          // bytes32 now
  entity.electionId = event.params.electionId.toI32()
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}