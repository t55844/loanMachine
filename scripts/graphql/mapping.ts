import { BigInt } from "@graphprotocol/graph-ts"
import { Donated, Borrowed, Repaid,NewBorrower,NewDonor } from "./generated/LoanMachine/LoanMachine"
import { Donation, Borrow, Repayment, User } from "./generated/schema"


export function handleNewDonor(event: NewDonor): void {
  let user = User.load(event.params.donor.toHexString())
  if (!user) {
    user = new User(event.params.donor.toHexString())
    user.totalDonated = BigInt.fromI32(0)
    user.totalBorrowed = BigInt.fromI32(0)
    user.currentDebt = BigInt.fromI32(0)
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
  }
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleDonated(event: Donated): void {
  let donation = new Donation(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  donation.donor = event.params.donor.toHexString() // Convert Address → String
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
  }
  user.totalDonated = user.totalDonated.plus(event.params.amount)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleBorrowed(event: Borrowed): void {
  let borrow = new Borrow(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  borrow.borrower = event.params.borrower.toHexString() // Convert Address → String
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
  }
  user.totalBorrowed = user.totalBorrowed.plus(event.params.amount)
  user.currentDebt = user.currentDebt.plus(event.params.amount)
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleRepaid(event: Repaid): void {
  let repayment = new Repayment(event.transaction.hash.toHex() + "-" + event.logIndex.toString())
  repayment.borrower = event.params.borrower.toHexString() // Convert Address → String
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
