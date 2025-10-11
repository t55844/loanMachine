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
  LoanCompleted,
  MemberToWalletVinculation,
  ReputationChanged,
  BorrowerStatusUpdated,
  DebtorAdded,
  DebtorRemoved,
  MonthlyUpdateTriggered,
  AuthorizedCallerUpdated,
  ElectionOpened,
  CandidateAdded,
  VoteCast,
  ElectionClosed,
  UnbeatableMajorityReached
} from "./generated/LoanMachine/LoanMachine"

import {
  Donation,
  Borrow,
  Repayment,
  User,
  Stats,
  LoanRequest,
  LoanCoverage,
  LoanContract,
  Member,
  ReputationChange,
  DebtStatus,
  DebtorList,
  AuthorizedCaller,
  Election,
  Candidate,
  Vote
} from "./generated/schema"

// -------------------
// Utility Functions
// -------------------
function getOrCreateUser(id: string): User {
  let user = User.load(id);
  if (!user) {
    user = new User(id);
    user.totalDonated = BigInt.zero();
    user.totalBorrowed = BigInt.zero();
    user.currentDebt = BigInt.zero();
    user.lastActivity = BigInt.zero();
    user.currentDebtStatus = "NoDebt";
    user.hasOpenDebt = false;
    user.isInDebtorList = false;
    user.isAuthorizedCaller = false;
    user.lastStatusUpdate = BigInt.zero();
    user.save();
  }
  return user as User;
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
  return stats as Stats
}

function getOrCreateDebtorList(): DebtorList {
  let debtorList = DebtorList.load("singleton")
  if (!debtorList) {
    debtorList = new DebtorList("singleton")
    debtorList.lastMonthlyUpdate = BigInt.zero()
    debtorList.nextMonthlyUpdate = BigInt.zero()
    debtorList.shouldTriggerUpdate = false
    debtorList.save()
  }
  return debtorList as DebtorList
}

function getOrCreateMember(memberId: BigInt): Member {
  let memberIdStr = memberId.toString();
  let member = Member.load(memberIdStr);
  if (!member) {
    member = new Member(memberIdStr);
    member.memberId = memberId;
    member.linkedAt = BigInt.zero();
    member.currentReputation = 0;
    member.save();
  }
  return member as Member;
}

function getDebtStatusString(status: i32): string {
  switch (status) {
    case 0:
      return "NoDebt"
    case 1:
      return "HasActiveDebt"
    case 2:
      return "HasOverdueDebt"
    default:
      return "NoDebt"
  }
}

// -------------------
// Core Lending Event Handlers
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
  user.currentDebtStatus = "HasActiveDebt"
  user.hasOpenDebt = true
  user.save()

  updateDebtorList(borrowerId, true)
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
    
    if (user.currentDebt.equals(BigInt.zero())) {
      user.currentDebtStatus = "NoDebt"
      user.hasOpenDebt = false
      updateDebtorList(borrowerId, false)
    } else {
      user.currentDebtStatus = "HasActiveDebt"
      user.hasOpenDebt = true
    }
    
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

export function handleParcelPaid(event: ParcelPaid): void {
  let id = event.params.requisitionId.toString();
  let loan = LoanRequest.load(id);
  if (!loan) return;
  if (loan.parcelsCount > 0) {
    loan.parcelsCount = loan.parcelsCount - 1;
  }
  loan.save();
}

export function handleLenderRepaid(event: LenderRepaid): void {
  let id = event.params.requisitionId.toString();
  let loanContract = LoanContract.load(id);
  if (!loanContract) return;
  if (loanContract.parcelsPending > 0) {
    loanContract.parcelsPending = loanContract.parcelsPending - 1;
  }
  loanContract.save();
}

export function handleLoanCompleted(event: LoanCompleted): void {
  let id = event.params.requisitionId.toString();
  let loan = LoanRequest.load(id);
  if (!loan) return;
  loan.funded = false;
  loan.save();
}

// -------------------
// Member System Event Handlers
// -------------------
export function handleMemberToWalletVinculation(event: MemberToWalletVinculation): void {
  let memberIdStr = event.params.memberId.toString();
  let member = getOrCreateMember(event.params.memberId);

  let wallet = event.params.wallet.toHexString();
  member.wallet = wallet;
  member.linkedAt = event.block.timestamp;
  member.save();

  let user = getOrCreateUser(wallet);
  user.lastActivity = event.block.timestamp;
  user.save();
}

export function handleReputationChanged(event: ReputationChanged): void {
  let id = event.transaction.hash.toHex() + "-" + event.logIndex.toString();
  let rc = new ReputationChange(id);

  let memberIdStr = event.params.memberId.toString();
  rc.memberId = event.params.memberId;
  rc.member = memberIdStr;
  rc.points = event.params.points;
  rc.increase = event.params.increase;
  rc.newReputation = event.params.newReputation;
  rc.timestamp = event.block.timestamp;
  rc.save();

  let member = Member.load(memberIdStr);
  if (member) {
    member.currentReputation = event.params.newReputation;
    member.save();

    if (member.wallet) {
      let user = User.load(member.wallet as string);
      if (user) {
        user.lastActivity = event.block.timestamp;
        user.save();
      }
    }
  }
}

// -------------------
// DebtTracker Event Handlers
// -------------------
export function handleBorrowerStatusUpdated(event: BorrowerStatusUpdated): void {
  let borrowerId = event.params.borrower.toHexString()
  let user = getOrCreateUser(borrowerId)
  
  let debtStatus = new DebtStatus(
    event.transaction.hash.toHex() + "-" + event.logIndex.toString()
  )
  debtStatus.borrower = borrowerId
  debtStatus.status = getDebtStatusString(event.params.newStatus)
  debtStatus.timestamp = event.block.timestamp
  debtStatus.save()

  user.currentDebtStatus = getDebtStatusString(event.params.newStatus)
  user.hasOpenDebt = user.currentDebtStatus !== "NoDebt"
  user.lastStatusUpdate = event.block.timestamp
  user.lastActivity = event.block.timestamp
  user.save()
}

export function handleDebtorAdded(event: DebtorAdded): void {
  let borrowerId = event.params.borrower.toHexString()
  let user = getOrCreateUser(borrowerId)
  
  user.isInDebtorList = true
  user.lastActivity = event.block.timestamp
  user.save()

  let debtorList = getOrCreateDebtorList()
  let currentDebtors = debtorList.borrowersWithOpenDebt
  if (!currentDebtors.includes(borrowerId)) {
    currentDebtors.push(borrowerId)
    debtorList.borrowersWithOpenDebt = currentDebtors
    debtorList.save()
  }
}

export function handleDebtorRemoved(event: DebtorRemoved): void {
  let borrowerId = event.params.borrower.toHexString()
  let user = getOrCreateUser(borrowerId)
  
  user.isInDebtorList = false
  user.lastActivity = event.block.timestamp
  user.save()

  let debtorList = getOrCreateDebtorList()
  let currentDebtors = debtorList.borrowersWithOpenDebt
  let updatedDebtors: string[] = []
  
  for (let i = 0; i < currentDebtors.length; i++) {
    if (currentDebtors[i] !== borrowerId) {
      updatedDebtors.push(currentDebtors[i])
    }
  }
  
  debtorList.borrowersWithOpenDebt = updatedDebtors
  debtorList.save()
}

export function handleMonthlyUpdateTriggered(event: MonthlyUpdateTriggered): void {
  let debtorList = getOrCreateDebtorList()
  
  debtorList.lastMonthlyUpdate = event.params.timestamp
  debtorList.nextMonthlyUpdate = event.params.timestamp.plus(BigInt.fromI32(2592000))
  debtorList.shouldTriggerUpdate = false
  debtorList.save()
}

// -------------------
// Authorization Event Handlers
// -------------------
export function handleAuthorizedCallerUpdated(event: AuthorizedCallerUpdated): void {
  let callerId = event.params.caller.toHexString()
  let authorizedCaller = AuthorizedCaller.load(callerId)
  
  if (!authorizedCaller) {
    authorizedCaller = new AuthorizedCaller(callerId)
    authorizedCaller.caller = callerId
  }
  
  authorizedCaller.authorized = event.params.authorized
  authorizedCaller.lastUpdated = event.block.timestamp
  authorizedCaller.save()

  let user = getOrCreateUser(callerId)
  user.isAuthorizedCaller = event.params.authorized
  user.lastActivity = event.block.timestamp
  user.save()
}

// -------------------
// Election System Event Handlers
// -------------------
export function handleElectionOpened(event: ElectionOpened): void {
  let electionId = event.params.electionId.toString() + "-" + event.params.candidateId.toString()
  let election = new Election(electionId)
  
  election.electionId = event.params.electionId
  election.candidateId = event.params.candidateId
  election.startTime = event.params.startTime
  election.endTime = event.params.endTime
  election.isOpen = true
  election.createdAt = event.block.timestamp
  election.save()

  // Create the initial candidate
  let candidateId = electionId + "-candidate-" + event.params.candidateId.toString()
  let candidate = new Candidate(candidateId)
  candidate.election = electionId
  candidate.candidateId = event.params.candidateId
  candidate.totalVotes = 0
  candidate.addedAt = event.block.timestamp
  
  // Link candidate to member if exists
  let member = Member.load(event.params.candidateId.toString())
  if (member) {
    candidate.member = member.id
  }
  
  candidate.save()
}

export function handleCandidateAdded(event: CandidateAdded): void {
  let electionId = event.params.electionId.toString() + "-" + event.params.candidateId.toString()
  let election = Election.load(electionId)
  
  if (election) {
    let candidateId = electionId + "-candidate-" + event.params.candidateId.toString()
    let candidate = new Candidate(candidateId)
    candidate.election = electionId
    candidate.candidateId = event.params.candidateId
    candidate.totalVotes = 0
    candidate.addedAt = event.block.timestamp
    
    let member = Member.load(event.params.candidateId.toString())
    if (member) {
      candidate.member = member.id
    }
    
    candidate.save()
  }
}

export function handleVoteCast(event: VoteCast): void {
  let electionId = event.params.electionId.toString() + "-" + event.params.candidateId.toString()
  let election = Election.load(electionId)
  
  if (election) {
    let candidateId = electionId + "-candidate-" + event.params.candidateId.toString()
    let candidate = Candidate.load(candidateId)
    
    if (candidate) {
      // Update candidate vote total
      candidate.totalVotes = candidate.totalVotes + event.params.voteWeight
      candidate.save()

      // Create vote record
      let voteId = electionId + "-vote-" + event.params.memberId.toString()
      let vote = new Vote(voteId)
      vote.election = electionId
      vote.candidate = candidateId
      vote.voter = event.params.memberId.toString()
      vote.voteWeight = event.params.voteWeight
      vote.timestamp = event.block.timestamp
      vote.save()
    }
  }
}

export function handleElectionClosed(event: ElectionClosed): void {
  let electionId = event.params.electionId.toString() + "-" + event.params.winnerId.toString()
  let election = Election.load(electionId)
  
  if (election) {
    election.isOpen = false
    election.winnerId = event.params.winnerId
    election.winningVotes = event.params.winningVotes
    election.save()
  }
}

export function handleUnbeatableMajorityReached(event: UnbeatableMajorityReached): void {
  let electionId = event.params.electionId.toString() + "-" + event.params.winnerId.toString()
  let election = Election.load(electionId)
  
  if (election) {
    election.isOpen = false
    election.winnerId = event.params.winnerId
    election.winningVotes = event.params.winningVotes
    election.save()
  }
}

// -------------------
// Helper Functions
// -------------------
function updateDebtorList(borrowerId: string, add: boolean): void {
  let debtorList = getOrCreateDebtorList()
  let currentDebtors = debtorList.borrowersWithOpenDebt
  
  if (add) {
    if (!currentDebtors.includes(borrowerId)) {
      currentDebtors.push(borrowerId)
      debtorList.borrowersWithOpenDebt = currentDebtors
    }
  } else {
    let updatedDebtors: string[] = []
    for (let i = 0; i < currentDebtors.length; i++) {
      if (currentDebtors[i] !== borrowerId) {
        updatedDebtors.push(currentDebtors[i])
      }
    }
    debtorList.borrowersWithOpenDebt = updatedDebtors
  }
  
  debtorList.save()
}