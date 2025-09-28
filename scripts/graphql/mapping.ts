import { BigInt } from "@graphprotocol/graph-ts"
import {
  Donated,
  Borrowed,
  Repaid,
  NewBorrower,
  NewDonor,
  TotalDonationsUpdated,
  TotalBorrowedUpdated,
  AvailableBalanceUpdated,
  LoanRequisitionCreated,
  LoanCovered,
  LoanFunded,
  LoanContractGenerated,
  ParcelPaid,
  LenderRepaid,
  LoanCompleted
} from "./generated/LoanMachine/LoanMachine"

import {
  Donation,
  Borrow,
  Repayment,
  User,
  Stats,
  LoanRequest,
  LoanCoverage,
  LoanContract
} from "./generated/schema"

// -------------------
// Utility to create/load User entity
// -------------------
function getOrCreateUser(id: string): User {
  let user = User.load(id)
  if (!user) {
    user = new User(id)
    user.totalDonated = BigInt.fromI32(0)
    user.totalBorrowed = BigInt.fromI32(0)
    user.currentDebt = BigInt.fromI32(0)
    user.lastActivity = BigInt.fromI32(0)
  }
  return user as User
}

// -------------------
// Utility to create/load Stats entity
// -------------------
function getOrCreateStats(): Stats {
  let stats = Stats.load("singleton")
  if (!stats) {
    stats = new Stats("singleton")
    stats.totalDonations = BigInt.fromI32(0)
    stats.totalBorrowed = BigInt.fromI32(0)
    stats.availableBalance = BigInt.fromI32(0)
    stats.save()
  }
  return stats as Stats
}

// -------------------
// Event Handlers
// -------------------
export function handleNewDonor(event: NewDonor): void {
  let id = event.params.donor.toHexString()
  let user = getOrCreateUser(id)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleNewBorrower(event: NewBorrower): void {
  let id = event.params.borrower.toHexString()
  let user = getOrCreateUser(id)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleDonated(event: Donated): void {
  let donation = new Donation(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  let donorId = event.params.donor.toHexString()
  donation.donor = donorId
  donation.amount = event.params.amount
  donation.timestamp = event.block.timestamp
  donation.totalDonation = event.params.totalDonation
  donation.save()

  let user = getOrCreateUser(donorId)
  user.totalDonated = user.totalDonated.plus(event.params.amount)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleBorrowed(event: Borrowed): void {
  let borrow = new Borrow(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  let borrowerId = event.params.borrower.toHexString()
  borrow.borrower = borrowerId
  borrow.amount = event.params.amount
  borrow.timestamp = event.block.timestamp
  borrow.totalBorrowing = event.params.totalBorrowing
  borrow.save()

  let user = getOrCreateUser(borrowerId)
  user.totalBorrowed = user.totalBorrowed.plus(event.params.amount)
  user.currentDebt = user.currentDebt.plus(event.params.amount)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleRepaid(event: Repaid): void {
  let repayment = new Repayment(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  let borrowerId = event.params.borrower.toHexString()
  repayment.borrower = borrowerId
  repayment.amount = event.params.amount
  repayment.timestamp = event.block.timestamp
  repayment.remainingDebt = event.params.remainingDebt
  repayment.save()

  let user = User.load(borrowerId)
  if (user) {
    user.currentDebt = event.params.remainingDebt
    user.lastActivity = event.block.timestamp
    user.save()
  }
}

export function handleTotalDonationsUpdated(event: TotalDonationsUpdated): void {
  let stats = getOrCreateStats()
  stats.totalDonations = event.params.total
  stats.save()
}

export function handleTotalBorrowedUpdated(event: TotalBorrowedUpdated): void {
  let stats = getOrCreateStats()
  stats.totalBorrowed = event.params.total
  stats.save()
}

export function handleAvailableBalanceUpdated(event: AvailableBalanceUpdated): void {
  let stats = getOrCreateStats()
  stats.availableBalance = event.params.total
  stats.save()
}

export function handleLoanRequisitionCreated(event: LoanRequisitionCreated): void {
  let id = event.params.requisitionId.toString()
  let loan = new LoanRequest(id)
  loan.requisitionId = event.params.requisitionId
  loan.borrower = event.params.borrower.toHexString()
  loan.amount = event.params.amount
  loan.timestamp = event.block.timestamp
  loan.currentCoverageAmount = BigInt.fromI32(0)
  loan.coveringLendersCount = 0
  loan.funded = false
  loan.fundedAt = null
  loan.parcelsCount = event.params.parcelsCount.toI32()
  loan.save()

  let user = getOrCreateUser(event.params.borrower.toHexString())
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleLoanCovered(event: LoanCovered): void {
  let reqId = event.params.requisitionId.toString()
  let loan = LoanRequest.load(reqId)

  if (!loan) {
    loan = new LoanRequest(reqId)
    loan.requisitionId = event.params.requisitionId
    loan.borrower = "0x0000000000000000000000000000000000000000"
    loan.amount = BigInt.fromI32(0)
    loan.timestamp = event.block.timestamp
    loan.currentCoverageAmount = BigInt.fromI32(0)
    loan.coveringLendersCount = 0
    loan.funded = false
    loan.fundedAt = null
  }

  let lenderId = event.params.lender.toHexString()
  let lenderUser = getOrCreateUser(lenderId)
  lenderUser.lastActivity = event.block.timestamp
  lenderUser.save()

  let coverageId = reqId + "-" + lenderId
  let coverage = LoanCoverage.load(coverageId)
  if (!coverage) {
    coverage = new LoanCoverage(coverageId)
    coverage.loanRequest = reqId
    coverage.lender = lenderId
    coverage.amount = event.params.coverageAmount
    coverage.timestamp = event.block.timestamp
    loan.coveringLendersCount = loan.coveringLendersCount + 1
  } else {
    coverage.amount = coverage.amount.plus(event.params.coverageAmount)
    coverage.timestamp = event.block.timestamp
  }

  coverage.save()
  loan.currentCoverageAmount = loan.currentCoverageAmount.plus(event.params.coverageAmount)
  loan.save()
}

export function handleLoanFunded(event: LoanFunded): void {
  let reqId = event.params.requisitionId.toString()
  let loan = LoanRequest.load(reqId)
  if (!loan) {
    loan = new LoanRequest(reqId)
    loan.requisitionId = event.params.requisitionId
    loan.borrower = "0x0000000000000000000000000000000000000000"
    loan.amount = BigInt.fromI32(0)
    loan.timestamp = event.block.timestamp
    loan.currentCoverageAmount = BigInt.fromI32(0)
    loan.coveringLendersCount = 0
  }
  loan.funded = true
  loan.fundedAt = event.block.timestamp
  loan.save()
}

export function handleLoanContractGenerated(event: LoanContractGenerated): void {
  let id = event.params.requisitionId.toString()
  let lc = new LoanContract(id)
  lc.walletAddress = event.params.walletAddress.toHexString()
  lc.requisitionId = event.params.requisitionId
  lc.status = event.params.status
  lc.parcelsPending = event.params.parcelsPending.toI32()
  lc.createdAt = event.block.timestamp
  lc.save()

  let loanReq = LoanRequest.load(id)
  if (loanReq) {
    loanReq.parcelsCount = lc.parcelsPending
    loanReq.save()
  }
}

// -------------------
// NEW: ParcelPaid, LenderRepaid, LoanCompleted
// -------------------
export function handleParcelPaid(event: ParcelPaid): void {
  let id = event.params.requisitionId.toString()
  let loan = LoanRequest.load(id)
  if (!loan) return

  if (loan.parcelsCount > 0) {
    loan.parcelsCount = loan.parcelsCount - 1
  }
  loan.save()
}

export function handleLenderRepaid(event: LenderRepaid): void {
  let id = event.params.requisitionId.toString()
  let loanContract = LoanContract.load(id)
  if (!loanContract) return

  if (loanContract.parcelsPending > 0) {
    loanContract.parcelsPending = loanContract.parcelsPending - 1
  }
  loanContract.save()
}

export function handleLoanCompleted(event: LoanCompleted): void {
  let id = event.params.requisitionId.toString()
  let loan = LoanRequest.load(id)
  if (!loan) return

  // mark loan as completed
  loan.funded = false
  loan.save()
}
