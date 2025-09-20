import { BigInt } from "@graphprotocol/graph-ts"
import { 
  Donated, 
  Borrowed, 
  Repaid,
  NewBorrower,
  NewDonor,
  TotalDonationsUpdated,
  TotalBorrowedUpdated,
  AvailableBalanceUpdated
} from "./generated/LoanMachine/LoanMachine"
import { Donation, Borrow, Repayment, User, Stats } from "./generated/schema"

export function handleNewDonor(event: NewDonor): void {
  let user = User.load(event.params.donor.toHexString())
  if (!user) {
    user = new User(event.params.donor.toHexString())
    user.totalDonated = BigInt.fromI32(0)
    user.totalBorrowed = BigInt.fromI32(0)
    user.currentDebt = BigInt.fromI32(0)
    user.lastActivity = BigInt.fromI32(0)
  }
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleNewBorrower(event: NewBorrower): void {
  let user = User.load(event.params.borrower.toHexString())
  if (!user) {
    user = new User(event.params.borrower.toHexString())
    user.totalDonated = BigInt.fromI32(0)
    user.totalBorrowed = BigInt.fromI32(0)
    user.currentDebt = BigInt.fromI32(0)
    user.lastActivity = BigInt.fromI32(0)
  }
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleDonated(event: Donated): void {
  let donation = new Donation(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  donation.donor = event.params.donor.toHexString()
  donation.amount = event.params.amount
  donation.timestamp = event.block.timestamp
  donation.totalDonation = event.params.totalDonation
  donation.save()

  let user = User.load(event.params.donor.toHexString())
  if (!user) {
    user = new User(event.params.donor.toHexString())
    user.totalDonated = BigInt.fromI32(0)
    user.totalBorrowed = BigInt.fromI32(0)
    user.currentDebt = BigInt.fromI32(0)
    user.lastActivity = BigInt.fromI32(0)
  }
  user.totalDonated = user.totalDonated.plus(event.params.amount)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleBorrowed(event: Borrowed): void {
  let borrow = new Borrow(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  borrow.borrower = event.params.borrower.toHexString()
  borrow.amount = event.params.amount
  borrow.timestamp = event.block.timestamp
  borrow.totalBorrowing = event.params.totalBorrowing
  borrow.save()

  let user = User.load(event.params.borrower.toHexString())
  if (!user) {
    user = new User(event.params.borrower.toHexString())
    user.totalDonated = BigInt.fromI32(0)
    user.totalBorrowed = BigInt.fromI32(0)
    user.currentDebt = BigInt.fromI32(0)
    user.lastActivity = BigInt.fromI32(0)
  }
  user.totalBorrowed = user.totalBorrowed.plus(event.params.amount)
  user.currentDebt = user.currentDebt.plus(event.params.amount)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleRepaid(event: Repaid): void {
  let repayment = new Repayment(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  repayment.borrower = event.params.borrower.toHexString()
  repayment.amount = event.params.amount
  repayment.timestamp = event.block.timestamp
  repayment.remainingDebt = event.params.remainingDebt
  repayment.save()

  let user = User.load(event.params.borrower.toHexString())
  if (user) {
    user.currentDebt = event.params.remainingDebt
    user.lastActivity = event.block.timestamp
    user.save()
  }
}

function getOrCreateStats(): Stats {
  let stats = Stats.load("singleton")
  if (!stats) {
    stats = new Stats("singleton")
    stats.totalDonations = BigInt.fromI32(0)
    stats.totalBorrowed = BigInt.fromI32(0)
    stats.availableBalance = BigInt.fromI32(0)
    stats.save()
  }
  return stats
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