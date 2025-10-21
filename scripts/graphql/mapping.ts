import { BigInt, Bytes } from "@graphprotocol/graph-ts"
import {
  Donated,
  Borrowed,
  Repaid,
  TotalDonationsUpdated,
  TotalBorrowedUpdated,
  AvailableBalanceUpdated,
  NewDonor,
  NewBorrower,
  BorrowLimitReached,
  LoanRequisitionCreated,
  LoanCovered,
  LoanFunded,
  LoanContractGenerated,
  ParcelPaid,
  LenderRepaid,
  LoanCompleted,
  Withdrawn
} from "./generated/LoanMachine/LoanMachine"
import {
  MemberToWalletVinculation,
  ReputationChanged,
  AuthorizedCallerUpdated,
  ElectionOpened,
  CandidateAdded,
  VoteCast,
  ElectionClosed,
  UnbeatableMajorityReached,
  NewModerator
} from "./generated/ReputationSystem/ReputationSystem"
import {
  BorrowerStatusUpdated,
  DebtorAdded,
  DebtorRemoved,
  MonthlyUpdateTriggered
} from "./generated/DebtTracker/DebtTracker"
import {
  DonatedEvent,
  BorrowedEvent,
  RepaidEvent,
  TotalDonationsUpdatedEvent,
  TotalBorrowedUpdatedEvent,
  AvailableBalanceUpdatedEvent,
  NewDonorEvent,
  NewBorrowerEvent,
  BorrowLimitReachedEvent,
  LoanRequisitionCreatedEvent,
  LoanCoveredEvent,
  LoanFundedEvent,
  LoanContractGeneratedEvent,
  ParcelPaidEvent,
  LenderRepaidEvent,
  LoanCompletedEvent,
  MemberToWalletVinculationEvent,
  ReputationChangedEvent,
  AuthorizedCallerUpdatedEvent,
  ElectionOpenedEvent,
  CandidateAddedEvent,
  VoteCastEvent,
  ElectionClosedEvent,
  UnbeatableMajorityReachedEvent,
  BorrowerStatusUpdatedEvent,
  DebtorAddedEvent,
  DebtorRemovedEvent,
  MonthlyUpdateTriggeredEvent,
  WithdrawnEvent,
  NewModeratorEvent  
} from "./generated/schema"

function formatTimestamp(ts: BigInt): string {
  let ms = ts.times(BigInt.fromI32(1000)).toI64()
  let dt = new Date(ms)
  let dayStr = dt.getUTCDate().toString()
  let monthStr = (dt.getUTCMonth() + 1).toString()
  let yearStr = dt.getUTCFullYear().toString()
  let hourStr = dt.getUTCHours().toString()
  let minuteStr = dt.getUTCMinutes().toString()
  let secondStr = dt.getUTCSeconds().toString()

  if (dayStr.length == 1) {dayStr = "0" + dayStr}
  if (monthStr.length == 1) {monthStr = "0" + monthStr}
  if (hourStr.length == 1) {hourStr = "0" + hourStr}
  if (minuteStr.length == 1) {minuteStr = "0" + minuteStr}
  if (secondStr.length == 1) {secondStr = "0" + secondStr}
 

  return dayStr + "/" + monthStr + "/" + yearStr + " " + hourStr + ":" + minuteStr + ":" + secondStr
}

export function handleDonated(event: Donated): void {
  let entity = new DonatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.donor = event.params.donor
  entity.amount = event.params.amount
  entity.totalDonation = event.params.totalDonation
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowed(event: Borrowed): void {
  let entity = new BorrowedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.amount = event.params.amount
  entity.totalBorrowing = event.params.totalBorrowing
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleRepaid(event: Repaid): void {
  let entity = new RepaidEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.amount = event.params.amount
  entity.remainingDebt = event.params.remainingDebt
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleTotalDonationsUpdated(event: TotalDonationsUpdated): void {
  let entity = new TotalDonationsUpdatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.total = event.params.total
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleTotalBorrowedUpdated(event: TotalBorrowedUpdated): void {
  let entity = new TotalBorrowedUpdatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.total = event.params.total
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAvailableBalanceUpdated(event: AvailableBalanceUpdated): void {
  let entity = new AvailableBalanceUpdatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.total = event.params.total
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewDonor(event: NewDonor): void {
  let entity = new NewDonorEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.donor = event.params.donor
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewBorrower(event: NewBorrower): void {
  let entity = new NewBorrowerEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowLimitReached(event: BorrowLimitReached): void {
  let entity = new BorrowLimitReachedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanRequisitionCreated(event: LoanRequisitionCreated): void {
  let entity = new LoanRequisitionCreatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.requisitionId = event.params.requisitionId
  entity.borrower = event.params.borrower
  entity.amount = event.params.amount
  entity.parcelsCount = event.params.parcelsCount.toI32()
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanCovered(event: LoanCovered): void {
  let entity = new LoanCoveredEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.requisitionId = event.params.requisitionId
  entity.lender = event.params.lender
  entity.coverageAmount = event.params.coverageAmount
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanFunded(event: LoanFunded): void {
  let entity = new LoanFundedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.requisitionId = event.params.requisitionId
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanContractGenerated(event: LoanContractGenerated): void {
  let entity = new LoanContractGeneratedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.walletAddress = event.params.walletAddress
  entity.requisitionId = event.params.requisitionId
  entity.status = event.params.status
  entity.parcelsPending = event.params.parcelsPending.toI32()
  entity.parcelsValues = event.params.parcelsValues

 let paymentDatesArr = new Array<string>();
  for (let i = 0; i < event.params.paymentDates.length; i++) {
    paymentDatesArr.push(formatTimestamp(event.params.paymentDates[i]));
  }
  entity.paymentDates = paymentDatesArr;
  entity.creationTime = formatTimestamp(event.block.timestamp)

  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleParcelPaid(event: ParcelPaid): void {
  let entity = new ParcelPaidEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.requisitionId = event.params.requisitionId
  entity.parcelsRemaining = event.params.parcelsRemaining
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLenderRepaid(event: LenderRepaid): void {
  let entity = new LenderRepaidEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.requisitionId = event.params.requisitionId
  entity.lender = event.params.lender
  entity.amount = event.params.amount
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleLoanCompleted(event: LoanCompleted): void {
  let entity = new LoanCompletedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.requisitionId = event.params.requisitionId
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleMemberToWalletVinculation(event: MemberToWalletVinculation): void {
  let entity = new MemberToWalletVinculationEvent(
    event.transaction.hash.toHex() + "-" + event.logIndex.toString()
  )
  entity.memberId = event.params.memberId.toI32()
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
  let entity = new ReputationChangedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.memberId = event.params.memberId.toI32()
  entity.points = event.params.points
  entity.increase = event.params.increase
  entity.newReputation = event.params.newReputation
  entity.timestamp = formatTimestamp(event.params.timestamp)
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleAuthorizedCallerUpdated(event: AuthorizedCallerUpdated): void {
  let entity = new AuthorizedCallerUpdatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.caller = event.params.caller
  entity.authorized = event.params.authorized
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleElectionOpened(event: ElectionOpened): void {
  let entity = new ElectionOpenedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.electionId = event.params.electionId.toI32()
  entity.candidateId = event.params.candidateId.toI32()
  entity.startTime = formatTimestamp(event.params.startTime)
  entity.endTime = formatTimestamp(event.params.endTime)
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleCandidateAdded(event: CandidateAdded): void {
  let entity = new CandidateAddedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.electionId = event.params.electionId.toI32()
  entity.candidateId = event.params.candidateId.toI32()
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleVoteCast(event: VoteCast): void {
  let entity = new VoteCastEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.electionId = event.params.electionId.toI32()
  entity.candidateId = event.params.candidateId.toI32()
  entity.memberId = event.params.memberId.toI32()
  entity.voteWeight = event.params.voteWeight
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleElectionClosed(event: ElectionClosed): void {
  let entity = new ElectionClosedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.electionId = event.params.electionId.toI32()
  entity.winnerId = event.params.winnerId.toI32()
  entity.winningVotes = event.params.winningVotes
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleUnbeatableMajorityReached(event: UnbeatableMajorityReached): void {
  let entity = new UnbeatableMajorityReachedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.electionId = event.params.electionId.toI32()
  entity.winnerId = event.params.winnerId.toI32()
  entity.winningVotes = event.params.winningVotes
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleBorrowerStatusUpdated(event: BorrowerStatusUpdated): void {
  let entity = new BorrowerStatusUpdatedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.newStatus = event.params.newStatus
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleDebtorAdded(event: DebtorAdded): void {
  let entity = new DebtorAddedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleDebtorRemoved(event: DebtorRemoved): void {
  let entity = new DebtorRemovedEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.borrower = event.params.borrower
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleMonthlyUpdateTriggered(event: MonthlyUpdateTriggered): void {
  let entity = new MonthlyUpdateTriggeredEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.timestamp = formatTimestamp(event.params.timestamp)
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}


export function handleWithdrawn(event: Withdrawn): void {
  let entity = new WithdrawnEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.donor = event.params.donor
  entity.amount = event.params.amount
  entity.donations = event.params.donations
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}

export function handleNewModerator(event: NewModerator): void {
  let entity = new NewModeratorEvent(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  entity.memberId = event.params.memberId.toI32()
  entity.electionId = event.params.electionId.toI32()
  entity.blockTimestamp = formatTimestamp(event.block.timestamp)
  entity.transactionHash = event.transaction.hash
  entity.save()
}