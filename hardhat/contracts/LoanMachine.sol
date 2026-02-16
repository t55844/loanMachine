// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import "./ILoanMachine.sol";
import "./IReputationSystem.sol";

contract LoanMachine is ILoanMachine, IReputationSystem, ReentrancyGuard {


    struct DebtWatchItem {
            uint256 requisitionId;
            address borrower;
            uint256 nextDueDate;
            bool isOverdue;
        }

    struct LoanRequisition {
        uint256 requisitionId;
        address borrower;
        uint256 amount;
        uint32 minimumCoverage;
        uint32 currentCoverage;
        BorrowStatus status;
        uint256 creationTime;
        address[] coveringLenders;
        mapping(address => uint256) coverageAmounts;
        uint32 parcelsCount;
        uint32 daysIntervalOfPayment;
    }
    // =============================================================
    //                  REPUTATION SYSTEM CONSTANTS & STORAGE
    // =============================================================

    int32 public constant  REPUTATION_GAIN_BY_REPAYNG_DEBT = 1;
    int32 public constant  REPUTATION_GAIN_BY_COVERING_LOAN = 2;
    int32 public constant  REPUTATION_LOSS_BY_DEBT_NOT_PAYD = 3;

    mapping(address => bool) public  authorizedCallers;

    mapping(uint32 => address) private memberToWallet;
    mapping(address => uint32) public walletToMemberId;
    mapping(uint32 => address[]) public walletsOfMember;
    mapping(uint32 => int32) public memberReputation;

    int32 public totalPotentialVotes;
    mapping(uint32 => bool) private activeMembers;
    uint32[] private membersWithPositiveReputation;

    mapping(uint32 => bool) public  isModerator;
    mapping(uint32 => int32) public moderatorVotesReceived;
    mapping(uint32 => mapping(uint32 => bool)) public hasVotedInElection;

    uint32 private electionCounter;
    ElectionStatus[] public elections;

    // =============================================================
    //                  LOAN MACHINE STORAGE
    // =============================================================

    DebtWatchItem[] public debtWatchlist;
    mapping(uint256 => uint256) private watchlistIndex;

    uint256 public nextCheckIndex;
    uint256 public lastPeriodicCheckTimestamp;
    uint256 public constant CHECK_INTERVAL = 15 days;

    mapping(address => uint256) private donations;
    mapping(address => uint256) private borrowings;
    mapping(address => uint256) private lastBorrowTime;

    uint256 private totalDonations;
    uint256 private totalBorrowed;
    uint256 private availableBalance;

    address public immutable usdtToken;

    mapping(uint256 => LoanRequisition) private loanRequisitions;
    uint256 public requisitionCounter = 1;
    mapping(address => uint256[]) public borrowerRequisitions;
    mapping(address => uint256) private donationsInCoverage;
    mapping(uint32 => uint32) private loanRequisitionNumber;
    mapping(address => uint256) private lastContractPerWalletId;

    mapping(uint256 => LoanContract) public loanContracts;

    uint32 private constant BORROW_DURATION = 30 days;
    uint32 private constant MIN_DONATION_FOR_BORROW = 1e6; // 1 USDT (6 decimals)
    uint32 private constant MAX_DONATION = 5e6;           // 5 USDT (6 decimals)

    // =============================================================
    //                  CUSTOM ERRORS
    // =============================================================

    // LoanMachine errors
    error LoanMachine_InvalidAmount();
    error LoanMachine_InsufficientFunds();
    error LoanMachine_MinimumDonationRequired();
    error LoanMachine_BorrowNotExpired();
    error LoanMachine_InvalidCoveragePercentage();
    error LoanMachine_OverCoverage();
    error LoanMachine_LoanNotAvailable();
    error LoanMachine_MaxLoanRequisitionPendingReached();
    error LoanMachine_InsufficientDonationBalance();
    error LoanMachine_NoActiveBorrowing();
    error LoanMachine_RepaymentExceedsBorrowed();
    error LoanMachine_InvalidParcelsCount();
    error LoanMachine_ExcessiveDonation();
    error LoanMachine_TokenTransferFailed();
    error LoanMachine_MemberIdOrWalletInvalid();
    error LoanMachine_WalletAlreadyVinculated();
    error LoanMachine_ParcelAlreadyPaid();
    error LoanMachine_MinimumPercentageCover();
    error LoanMachine_InsufficientWithdrawableBalance();
    error LoanMachine_CheckIntervalNotYetPassed();
    error LoanMachine_NotLoanRequisitionCreator();
    error LoanMachine_RequisitionAlreadyActive();
    error LoanMachine_RequisitionNotFound();
    error LoanMachine_OnlyBorrowerCanCancelRequisition();
    error LoanMachine_RequisitionNotCancellable();
    error LoanMachine_RequisitionAlreadyFullyCovered();
    error LoanMachine_IntervalOfPaymentAboveLimit();

    // Reputation errors
    error ReputationSystem_MemberIdOrWalletInvalid();
    error ReputationSystem_WalletAlreadyVinculated();
    error ReputationSystem_ActiveElectionExists();
    error ReputationSystem_ElectionNotActive();
    error ReputationSystem_MemberAlreadyVoted();
    error ReputationSystem_InvalidCandidate();
    error ReputationSystem_NoCandidates();
    error ReputationSystem_ElectionExpired();
    error ReputationSystem_WalletAlreadyLinkedToAnotherMember();

    // =============================================================
    //                  MODIFIERS
    // =============================================================


    modifier registerMemberData(uint32 memberId, address wallet) {
        if (memberId == 0 || wallet == address(0)) revert ReputationSystem_MemberIdOrWalletInvalid();
        if (walletToMemberId[wallet] != 0 && walletToMemberId[wallet] != memberId)
            revert ReputationSystem_WalletAlreadyLinkedToAnotherMember();
        _;
    }

    modifier validMember(uint32 memberId, address wallet) {
        if (memberId == 0 || walletToMemberId[wallet] == 0)
            revert LoanMachine_MemberIdOrWalletInvalid();
        _;
    }

    modifier minimumPercentCoveragePermited(uint256 minimumCoverage) {
        if (minimumCoverage <= 70 || minimumCoverage > 100) revert LoanMachine_InvalidCoveragePercentage();
        _;
    }

    modifier validAmount(uint256 _amount) {
        if (_amount == 0) revert LoanMachine_InvalidAmount();
        _;
    }

    modifier canBorrow(uint256 _amount) {
        if (_amount > availableBalance) revert LoanMachine_InsufficientFunds();
        if (donations[msg.sender] < MIN_DONATION_FOR_BORROW && borrowings[msg.sender] != 0)
            revert LoanMachine_MinimumDonationRequired();
        if (lastBorrowTime[msg.sender] + BORROW_DURATION >= block.timestamp && borrowings[msg.sender] != 0)
            revert LoanMachine_BorrowNotExpired();
        _;
    }

    modifier validCoverage(uint32 coveragePercentage) {
        if (coveragePercentage == 0 || coveragePercentage > 100) revert LoanMachine_InvalidCoveragePercentage();
        if (coveragePercentage <= 9) revert LoanMachine_MinimumPercentageCover();
        _;
    }

    modifier maxParcelsCountAndInterval(uint32 parcelscount, uint32 daysIntervalOfPayment) {
        if (parcelscount < 1 || parcelscount > 12) revert LoanMachine_InvalidParcelsCount();
        if (daysIntervalOfPayment > 30) revert LoanMachine_IntervalOfPaymentAboveLimit();
        _;
    }

    modifier borrowingActive(uint256 requisitionId) {
        LoanContract storage loan = loanContracts[requisitionId];
        if (borrowings[msg.sender] == 0) revert LoanMachine_NoActiveBorrowing();
        if (loan.walletAddress != msg.sender) revert LoanMachine_NoActiveBorrowing();
        if (loan.status != ContractStatus.Active) revert LoanMachine_NoActiveBorrowing();
        if (loan.parcelsPending == 0) revert LoanMachine_NoActiveBorrowing();
        _;
    }

    modifier checkLoanRequisitionOpened(uint32 memberId){
        if(loanRequisitionNumber[memberId] >= 3) revert LoanMachine_MaxLoanRequisitionPendingReached();
        _;
    }

    modifier hasActiveElection() {
        if (elections.length > 0 && elections[elections.length - 1].active)
            revert ReputationSystem_ActiveElectionExists();
        _;
    }

    modifier electionExists(uint32 electionId) {
        if (electionId >= elections.length) revert ReputationSystem_ElectionNotActive();
        _;
    }

    modifier electionActive(uint32 electionId) {
        ElectionStatus storage election = elections[electionId];
        if (block.timestamp > election.endTime || !election.active)
            revert ReputationSystem_ElectionNotActive();
        _;
    }

    // =============================================================
    //                  CONSTRUCTOR
    // =============================================================

    constructor(address _usdtToken) {
        usdtToken = _usdtToken;
    }

    // =============================================================
    //                  REPUTATION SYSTEM FUNCTIONS
    // =============================================================

    function vinculationMemberToWallet(uint32 memberId, address wallet)
        external
        
        registerMemberData(memberId, wallet)
    {
        for (uint i = 0; i < walletsOfMember[memberId].length; i++) {
            if (walletsOfMember[memberId][i] == wallet) {
                revert ReputationSystem_WalletAlreadyVinculated();
            }
        }

        if (memberToWallet[memberId] == address(0)) {
            memberToWallet[memberId] = wallet;
        }
        walletToMemberId[wallet] = memberId;
        walletsOfMember[memberId].push(wallet);

        emit MemberToWalletVinculation(memberId, wallet, walletsOfMember[memberId], block.timestamp);
    }

    function _reputationChange(uint32 memberId, int32 points, bool increase)
        internal
    {
        int32 currentReputation = memberReputation[memberId];
        int32 newReputation = increase ? currentReputation + points : currentReputation - points;

        _updateTotalPotentialVotes(memberId, currentReputation, newReputation);

        memberReputation[memberId] = newReputation;
        emit ReputationChanged(memberId, points, increase, newReputation, block.timestamp);
    }

    function getReputation(uint32 memberId) external view  returns (int32) {
        return memberReputation[memberId];
    }

    function getMemberId(address wallet) external view  returns (uint32) {
        return walletToMemberId[wallet];
    }

    function isWalletVinculated(address wallet) external view  returns (bool) {
        return walletToMemberId[wallet] != 0;
    }

    function openElection(uint32 candidateId, uint32 oponent) external  hasActiveElection {
        if(memberToWallet[candidateId] == address(0) || memberToWallet[oponent] == address(0))
            revert ReputationSystem_InvalidCandidate();

        ElectionStatus memory newElection;
        newElection.id = electionCounter++;
        newElection.candidates = new uint32[](2);
        newElection.candidates[0] = candidateId;
        newElection.candidates[1] = oponent;
        newElection.startTime = block.timestamp;
        newElection.endTime = block.timestamp + 30 days;
        newElection.active = true;
        newElection.totalVotesCast = 0;
        newElection.potentialRemainingVotes = _calculateTotalPotentialVotes();

        elections.push(newElection);
        emit ElectionOpened(newElection.id, candidateId, newElection.startTime, newElection.endTime);
    }

    function addCandidate(uint32 electionId, uint32 candidateId)
        external
        
        electionExists(electionId)
        electionActive(electionId)
    {
        ElectionStatus storage election = elections[electionId];
        election.candidates.push(candidateId);
        emit CandidateAdded(electionId, candidateId);
    }

    function voteForModerator(uint32 electionId, uint32 candidateId, uint32 memberId)
        external
        
        validMember(memberId, msg.sender)
        electionExists(electionId)
        electionActive(electionId)
    {
        ElectionStatus storage election = elections[electionId];

        if (hasVotedInElection[electionId][memberId]) revert ReputationSystem_MemberAlreadyVoted();

        bool validCandidate = false;
        for (uint i = 0; i < election.candidates.length; i++) {
            if (election.candidates[i] == candidateId) {
                validCandidate = true;
                break;
            }
        }
        if (!validCandidate) revert ReputationSystem_InvalidCandidate();

        int32 voteWeight = memberReputation[memberId];
        if (voteWeight < 0) voteWeight = 0;

        hasVotedInElection[electionId][memberId] = true;
        moderatorVotesReceived[candidateId] += voteWeight;
        election.totalVotesCast += voteWeight;
        _updatePotentialRemainingVotes(electionId, memberId);

        emit VoteCast(electionId, candidateId, memberId, voteWeight);
        _checkAndCloseElectionIfUnbeatable(electionId);
    }

    function closeElection(uint32 electionId)
        external
        
        electionExists(electionId)
    {
        ElectionStatus storage election = elections[electionId];
        if (!election.active) revert ReputationSystem_ElectionNotActive();
        if (election.candidates.length == 0) revert ReputationSystem_NoCandidates();

        election.active = false;

        uint32 winnerId = election.candidates[0];
        int32 winningVotes = moderatorVotesReceived[winnerId];

        for (uint i = 1; i < election.candidates.length; i++) {
            uint32 candidateId = election.candidates[i];
            int32 candidateVotes = moderatorVotesReceived[candidateId];

            if (candidateVotes > winningVotes) {
                winnerId = candidateId;
                winningVotes = candidateVotes;
            }
        }

        isModerator[winnerId] = true;
        election.winnerId = winnerId;
        election.winningVotes = winningVotes;

        emit ElectionClosed(electionId, winnerId, winningVotes);
    }

    function setAuthorizedCaller(address caller, bool authorized) external  {
        authorizedCallers[caller] = authorized;
        emit AuthorizedCallerUpdated(caller, authorized);
    }

    function getCandidateVotes(uint32 candidateId) external view  returns (int32) {
        return moderatorVotesReceived[candidateId];
    }

    function hasMemberVoted(uint32 electionId, uint32 memberId) external view  returns (bool) {
        return hasVotedInElection[electionId][memberId];
    }

    function getCurrentElectionId() external view  returns (int32) {
        if (elections.length == 0) return -1;
        ElectionStatus storage lastElection = elections[elections.length - 1];
        return lastElection.active ? int32(lastElection.id) : -1;
    }

    function getElectionInfo(uint32 electionId)
        external
        view
        
        electionExists(electionId)
        returns (
            uint32 id,
            uint32[] memory candidates,
            uint256 startTime,
            uint256 endTime,
            bool active,
            uint32 winnerId,
            int32 winningVotes,
            int32 totalVotesCast
        )
    {
        ElectionStatus storage election = elections[electionId];
        return (
            election.id,
            election.candidates,
            election.startTime,
            election.endTime,
            election.active,
            election.winnerId,
            election.winningVotes,
            election.totalVotesCast
        );
    }

    function getActiveMemberCount() external view returns (uint32) {
        return uint32(membersWithPositiveReputation.length);
    }

    function getAverageReputation() external view returns (int32) {
        uint32 activeCount = uint32(membersWithPositiveReputation.length);
        if (activeCount == 0) return 0;
        return totalPotentialVotes / int32(activeCount);
    }

    function isMemberActive(uint32 memberId) external view returns (bool) {
        return activeMembers[memberId];
    }

    // =============================================================
    //                  REPUTATION INTERNAL HELPERS
    // =============================================================

    function _calculateTotalPotentialVotes() internal view returns (int32) {
        return totalPotentialVotes;
    }

    function _updatePotentialRemainingVotes(uint32 electionId, uint32 memberId) internal {
        ElectionStatus storage election = elections[electionId];
        int32 memberRep = memberReputation[memberId];
        if (memberRep > 0) {
            election.potentialRemainingVotes -= memberRep;
        }
    }

    function _updateTotalPotentialVotes(uint32 memberId, int32 oldRep, int32 newRep) internal {
        int32 oldPositive = oldRep > 0 ? oldRep : int32(0);
        int32 newPositive = newRep > 0 ? newRep : int32(0);
        int32 difference = newPositive - oldPositive;

        totalPotentialVotes += difference;

        if (oldRep <= 0 && newRep > 0) {
            activeMembers[memberId] = true;
            membersWithPositiveReputation.push(memberId);
        } else if (oldRep > 0 && newRep <= 0) {
            activeMembers[memberId] = false;
            _removeMemberFromActiveList(memberId);
        }
    }

    function _removeMemberFromActiveList(uint32 memberId) internal {
        for (uint i = 0; i < membersWithPositiveReputation.length; i++) {
            if (membersWithPositiveReputation[i] == memberId) {
                membersWithPositiveReputation[i] = membersWithPositiveReputation[membersWithPositiveReputation.length - 1];
                membersWithPositiveReputation.pop();
                break;
            }
        }
    }

    function _checkAndCloseElectionIfUnbeatable(uint32 electionId) internal {
        ElectionStatus storage election = elections[electionId];

        if (election.candidates.length < 2) return;

        (uint32 firstPlaceId, int32 firstPlaceVotes, , int32 secondPlaceVotes) = _getTopTwoCandidates(electionId);

        if (firstPlaceId == 0) return;

        if (firstPlaceVotes > secondPlaceVotes + election.potentialRemainingVotes) {
            election.active = false;
            election.winnerId = firstPlaceId;
            election.winningVotes = firstPlaceVotes;
            isModerator[firstPlaceId] = true;

            emit NewModerator(election.winnerId, electionId);
            emit UnbeatableMajorityReached(electionId, firstPlaceId, firstPlaceVotes);
            emit ElectionClosed(electionId, firstPlaceId, firstPlaceVotes);
        }
    }

    function _getTopTwoCandidates(uint32 electionId) internal view returns (
        uint32 firstPlaceId,
        int32 firstPlaceVotes,
        uint32 secondPlaceId,
        int32 secondPlaceVotes
    ) {
        ElectionStatus storage election = elections[electionId];

        if (election.candidates.length == 0) return (0, 0, 0, 0);

        firstPlaceId = election.candidates[0];
        firstPlaceVotes = moderatorVotesReceived[firstPlaceId];
        secondPlaceId = 0;
        secondPlaceVotes = 0;

        for (uint i = 1; i < election.candidates.length; i++) {
            uint32 candidateId = election.candidates[i];
            int32 candidateVotes = moderatorVotesReceived[candidateId];

            if (candidateVotes > firstPlaceVotes) {
                secondPlaceId = firstPlaceId;
                secondPlaceVotes = firstPlaceVotes;
                firstPlaceId = candidateId;
                firstPlaceVotes = candidateVotes;
            } else if (candidateVotes > secondPlaceVotes) {
                secondPlaceId = candidateId;
                secondPlaceVotes = candidateVotes;
            }
        }

        return (firstPlaceId, firstPlaceVotes, secondPlaceId, secondPlaceVotes);
    }

    function checkUnbeatableMajority(uint32 electionId)
        external
        view
        electionExists(electionId)
        returns (
            bool isUnbeatable,
            uint32 leadingCandidate,
            int32 leadingVotes,
            uint32 secondCandidate,
            int32 secondVotes
        )
    {
        ElectionStatus storage election = elections[electionId];
        if (election.candidates.length < 2) return (false, 0, 0, 0, 0);

        (leadingCandidate, leadingVotes, secondCandidate, secondVotes) = _getTopTwoCandidates(electionId);
        isUnbeatable = (leadingVotes > secondVotes + election.potentialRemainingVotes);

        return (isUnbeatable, leadingCandidate, leadingVotes, secondCandidate, secondVotes);
    }

    // =============================================================
    //                  LOAN MACHINE FUNCTIONS (FULL)
    // =============================================================

    function performPeriodicDebtCheck(uint256 batchSize) external nonReentrant {
        uint256 watchlistLength = debtWatchlist.length;
        if (watchlistLength == 0) return;

        bool intervalPassed = block.timestamp > lastPeriodicCheckTimestamp + CHECK_INTERVAL;

        if (nextCheckIndex == 0 && !intervalPassed) {
            revert LoanMachine_CheckIntervalNotYetPassed();
        }

        uint256 itemsChecked = 0;
        for (uint256 i = 0; i < batchSize && nextCheckIndex < watchlistLength; i++) {
            DebtWatchItem storage item = debtWatchlist[nextCheckIndex];

            if (!item.isOverdue && block.timestamp > item.nextDueDate) {
                item.isOverdue = true;
                emit BorrowerOverdue(item.requisitionId, item.borrower, item.nextDueDate);
            }

            nextCheckIndex++;
            itemsChecked++;
        }

        if (nextCheckIndex >= watchlistLength) {
            nextCheckIndex = 0;
            lastPeriodicCheckTimestamp = block.timestamp;
        }

        emit PeriodicCheckRun(itemsChecked, nextCheckIndex);
    }

    function cancelLoanRequisition(uint256 requisitionId, uint32 memberId)
        external
        nonReentrant
        validMember(memberId, msg.sender)
        returns (uint256 totalUncoveredAmount)
    {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        uint256 initialCoverage = req.currentCoverage;

        if (initialCoverage == 100) revert LoanMachine_RequisitionAlreadyFullyCovered();

        if (req.borrower != msg.sender) {
            revert LoanMachine_OnlyBorrowerCanCancelRequisition();
        }

        if (req.status == BorrowStatus.Active || req.status == BorrowStatus.Cancelled) {
            revert LoanMachine_RequisitionNotCancellable();
        }

        totalUncoveredAmount = 0;

        for (uint256 i = 0; i < req.coveringLenders.length; i++) {
            address lender = req.coveringLenders[i];
            uint256 coverageAmount = req.coverageAmounts[lender];

            if (coverageAmount > 0) {
                donationsInCoverage[lender] -= coverageAmount;
                donations[lender] += coverageAmount;

                totalUncoveredAmount += coverageAmount;

                delete req.coverageAmounts[lender];

                emit LoanUncovered(requisitionId, lender, coverageAmount);
            }
        }

        if (loanRequisitionNumber[memberId] > 0) {
            loanRequisitionNumber[memberId] -= 1;
        }

        req.status = BorrowStatus.Cancelled;
        req.currentCoverage = 0;

        emit LoanRequisitionCreatedCancelled(requisitionId, msg.sender, req.amount, req.parcelsCount, req.status);

        return totalUncoveredAmount;
    }

    function withdraw(uint256 amount, uint32 memberId)
        external
        validMember(memberId, msg.sender)
        validAmount(amount)
        nonReentrant
    {
        uint256 withdrawableBalance = getWithdrawableBalance(msg.sender);

        if (amount > withdrawableBalance) {
            revert LoanMachine_InsufficientWithdrawableBalance();
        }

        donations[msg.sender] -= amount;
        totalDonations -= amount;
        availableBalance -= amount;

        bool success = IERC20(usdtToken).transfer(msg.sender, amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        emit Withdrawn(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);
    }

    function repay(uint256 requisitionId, uint256 amount, uint32 memberId)
        validMember(memberId, msg.sender)
        borrowingActive(requisitionId)
        external
        nonReentrant
    {
        if (amount == 0) revert LoanMachine_InvalidAmount();

        LoanContract storage loan = loanContracts[requisitionId];
        address borrower = msg.sender;

        uint32 currentParcelIndex = loan.parcelsCount - loan.parcelsPending;
        uint256 currentParcelDueDate = loan.paymentDates[currentParcelIndex];

        uint256 itemIndex = watchlistIndex[requisitionId];
        DebtWatchItem storage item = debtWatchlist[itemIndex];

        if (block.timestamp > currentParcelDueDate) {
            _reputationChange(memberId, REPUTATION_LOSS_BY_DEBT_NOT_PAYD, false);

            if (!item.isOverdue) {
                item.isOverdue = true;
                emit BorrowerOverdue(requisitionId, borrower, currentParcelDueDate);
            }
        } else {
            _reputationChange(memberId, REPUTATION_GAIN_BY_REPAYNG_DEBT, true);

            if (item.isOverdue) {
                item.isOverdue = false;
                emit BorrowerDebtSettled(requisitionId, borrower);
            }
        }

        if (amount != loan.parcelsValues) revert LoanMachine_InvalidAmount();

        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        borrowings[borrower] -= amount;
        totalBorrowed -= amount;
        availableBalance += amount;

        _distributeRepaymentToLenders(requisitionId, amount);

        loan.parcelsPending -= 1;
        if (loan.parcelsPending == 0) {
            loan.status = ContractStatus.Closed;
            emit LoanCompleted(requisitionId);

            _removeFromWatchlist(requisitionId);
        } else {
            uint32 nextParcelIndex = loan.parcelsCount - loan.parcelsPending;
            item.nextDueDate = loan.paymentDates[nextParcelIndex];
            item.isOverdue = false;
        }

        emit Repaid(borrower, amount, borrowings[borrower]);
        emit ParcelPaid(requisitionId, loan.parcelsPending);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
    }

    function _removeFromWatchlist(uint256 requisitionId) internal {
        uint256 indexToRemove = watchlistIndex[requisitionId];
        uint256 lastIndex = debtWatchlist.length - 1;

        if (indexToRemove != lastIndex) {
            DebtWatchItem storage lastItem = debtWatchlist[lastIndex];
            debtWatchlist[indexToRemove] = lastItem;
            watchlistIndex[lastItem.requisitionId] = indexToRemove;
        }

        debtWatchlist.pop();
        delete watchlistIndex[requisitionId];
    }

    function donate(uint256 amount, uint32 memberId)
        external
        validMember(memberId, msg.sender)
        validAmount(amount)
        nonReentrant
    {
        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        bool isNewDonor = donations[msg.sender] == 0;

        donations[msg.sender] += amount;
        totalDonations += amount;
        availableBalance += amount;

        emit Donated(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);

        if (isNewDonor) {
            emit NewDonor(msg.sender);
        }
    }

    function createLoanRequisition(
        uint256 amount,
        uint32 minimumCoverage,
        uint32 parcelscount,
        uint32 memberId,
        uint32 daysIntervalOfPayment
    )
        external
        checkLoanRequisitionOpened(memberId)
        validMember(memberId, msg.sender)
        validAmount(amount)
        minimumPercentCoveragePermited(minimumCoverage)
        maxParcelsCountAndInterval(parcelscount, daysIntervalOfPayment)
        returns (uint256)
    {
        if (amount > availableBalance) revert LoanMachine_InsufficientFunds();

        uint256 lastContractId = lastContractPerWalletId[msg.sender];

        if (lastContractId != 0) {
            uint256 lastCreationTime = loanContracts[lastContractId].creationTime;
            if (lastCreationTime + BORROW_DURATION > block.timestamp) {
                revert LoanMachine_BorrowNotExpired();
            }
        }

        uint256 requisitionId = requisitionCounter++;

        LoanRequisition storage newReq = loanRequisitions[requisitionId];
        newReq.requisitionId = requisitionId;
        newReq.borrower = msg.sender;
        newReq.amount = amount;
        newReq.minimumCoverage = minimumCoverage;
        newReq.currentCoverage = 0;
        newReq.status = BorrowStatus.Pending;
        newReq.creationTime = block.timestamp;
        newReq.parcelsCount = parcelscount;
        newReq.daysIntervalOfPayment = daysIntervalOfPayment;

        borrowerRequisitions[msg.sender].push(requisitionId);
        loanRequisitionNumber[memberId] += 1;

        emit LoanRequisitionCreatedCancelled(requisitionId, msg.sender, amount, parcelscount, newReq.status);
        return requisitionId;
    }

    function coverLoan(uint256 requisitionId, uint32 coveragePercentage, uint32 memberId)
        external
        validMember(memberId, msg.sender)
        validCoverage(coveragePercentage)
        nonReentrant
    {
        LoanRequisition storage req = loanRequisitions[requisitionId];

        if (req.status != BorrowStatus.Pending && req.status != BorrowStatus.PartiallyCovered) {
            revert LoanMachine_LoanNotAvailable();
        }

        if (uint256(req.currentCoverage) + uint256(coveragePercentage) > 100) {
            revert LoanMachine_OverCoverage();
        }

        uint256 lastContractId = lastContractPerWalletId[req.borrower];
        if (lastContractId != 0) {
            uint256 lastCreationTime = loanContracts[lastContractId].creationTime;
            if (lastCreationTime + BORROW_DURATION > block.timestamp) {
                revert LoanMachine_BorrowNotExpired();
            }
        }

        uint256 PRECISION = 1e18;
        uint256 coverageAmount = ((req.amount * coveragePercentage * PRECISION) + (100 * PRECISION - 1)) / (100 * PRECISION);

        if (donations[msg.sender] < coverageAmount) {
            revert LoanMachine_InsufficientDonationBalance();
        }

        donations[msg.sender] -= coverageAmount;
        donationsInCoverage[msg.sender] += coverageAmount;

        req.currentCoverage += coveragePercentage;

        if (req.coverageAmounts[msg.sender] == 0) {
            req.coveringLenders.push(msg.sender);
        }
        req.coverageAmounts[msg.sender] += coverageAmount;

        if (req.currentCoverage >= req.minimumCoverage) {
            req.status = BorrowStatus.FullyCovered;

            uint32 borrowerId = walletToMemberId[req.borrower];
            loanRequisitionNumber[borrowerId] = 0;
            _generateLoanContract(requisitionId);
            _fundLoan(requisitionId);
        } else {
            req.status = BorrowStatus.PartiallyCovered;
        }

        int32 reputationGain = (REPUTATION_GAIN_BY_COVERING_LOAN * int32(coveragePercentage)) / 10;
        _reputationChange(memberId, reputationGain, true);

        emit LoanCovered(requisitionId, msg.sender, coverageAmount);
    }

    function _generateLoanContract(uint256 requisitionId) internal {
        LoanContract storage loan = loanContracts[requisitionId];
        LoanRequisition storage req = loanRequisitions[requisitionId];

        loan.walletAddress = req.borrower;
        loan.requisitionId = requisitionId;
        loan.status = ContractStatus.Active;
        loan.parcelsCount = req.parcelsCount;
        loan.parcelsPending = req.parcelsCount;
        loan.creationTime = block.timestamp;

        uint256 totalAmount = req.amount;
        uint32 count = req.parcelsCount;

        uint256 base = totalAmount / count;
        uint256 remainder = totalAmount % count;

        uint256[] memory parcels = new uint256[](count);
        for (uint256 i = 0; i < count; i++) {
            parcels[i] = (i < remainder) ? base + 1 : base;
        }

        loan.parcelsValues = base;

        _generatePaymentDates(loan, count, req.daysIntervalOfPayment);

        lastContractPerWalletId[loan.walletAddress] = requisitionId;

        for (uint256 i = 0; i < count; i++) {
            loan.parcelsAmounts.push(parcels[i]);
        }

        emit LoanContractGenerated(
            loan.walletAddress,
            requisitionId,
            loan.status,
            loan.parcelsPending,
            loan.parcelsValues,
            loan.paymentDates,
            loan.creationTime
        );
    }

    function _generatePaymentDates(LoanContract storage loan, uint32 parcelsCount, uint32 daysIntervalOfPayment) internal {
        uint256 startDate = block.timestamp;
        uint256 interval = uint256(daysIntervalOfPayment) * 1 days;

        for (uint32 i = 0; i < parcelsCount; i++) {
            uint256 paymentDate = startDate + (interval * (i + 1));
            loan.paymentDates.push(paymentDate);
        }
    }

    function _fundLoan(uint256 requisitionId) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        address borrower = req.borrower;

        borrowings[borrower] += req.amount;
        totalBorrowed += req.amount;
        availableBalance -= req.amount;
        lastBorrowTime[borrower] = block.timestamp;
        req.status = BorrowStatus.Active;

        bool success = IERC20(usdtToken).transfer(borrower, req.amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        LoanContract storage loan = loanContracts[requisitionId];
        uint256 firstDueDate = loan.paymentDates[0];

        watchlistIndex[requisitionId] = debtWatchlist.length;
        debtWatchlist.push(DebtWatchItem({
            requisitionId: requisitionId,
            borrower: borrower,
            nextDueDate: firstDueDate,
            isOverdue: false
        }));

        emit Borrowed(borrower, req.amount, borrowings[borrower]);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
        emit LoanFunded(requisitionId);
    }

    function _distributeRepaymentToLenders(uint256 requisitionId, uint256 repaymentAmount) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        uint256 totalCoverageAmount = req.amount;

        for (uint256 i = 0; i < req.coveringLenders.length; i++) {
            address lender = req.coveringLenders[i];
            uint256 lenderCoverage = req.coverageAmounts[lender];
            uint256 lenderShare = (repaymentAmount * lenderCoverage) / totalCoverageAmount;

            if (lenderShare > 0) {
                if (lenderShare > donationsInCoverage[lender]) {
                    lenderShare = donationsInCoverage[lender];
                }
                donationsInCoverage[lender] -= lenderShare;
                donations[lender] += lenderShare;
                emit LenderRepaid(requisitionId, lender, lenderShare);
            }
        }
    }

    function getWithdrawableBalance(address user) public view returns (uint256) {
        uint256 availableDonations = donations[user];
        uint256 lockedInCoverage = donationsInCoverage[user];

        return availableDonations > lockedInCoverage ? availableDonations - lockedInCoverage : 0;
    }

    function getActiveLoans(address borrower) public view returns (LoanContract[] memory activeLoans, uint256[] memory requisitionIds) {
        uint256[] storage allRequisitionIds = borrowerRequisitions[borrower];
        uint256 activeCount = 0;

        for (uint256 i = 0; i < allRequisitionIds.length; i++) {
            LoanContract storage loan = loanContracts[allRequisitionIds[i]];
            if (loan.walletAddress == borrower && loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
                activeCount++;
            }
        }

        activeLoans = new LoanContract[](activeCount);
        requisitionIds = new uint256[](activeCount);
        uint256 index = 0;

        for (uint256 i = 0; i < allRequisitionIds.length; i++) {
            LoanContract storage loan = loanContracts[allRequisitionIds[i]];
            if (loan.walletAddress == borrower && loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
                activeLoans[index] = loan;
                requisitionIds[index] = allRequisitionIds[i];
                index++;
            }
        }

        return (activeLoans, requisitionIds);
    }

    function getDebtWatchlist() external view returns (DebtWatchItem[] memory) {
        return debtWatchlist;
    }

    function isBorrowerOverdue(uint256 requisitionId) external view returns (bool) {
        if (debtWatchlist.length == 0 || (watchlistIndex[requisitionId] == 0 && debtWatchlist[0].requisitionId != requisitionId)) {
            return false;
        }
        return debtWatchlist[watchlistIndex[requisitionId]].isOverdue;
    }

    function getNextPaymentAmount(uint256 requisitionId) external view returns (uint256 paymentAmount, bool canPay) {
        LoanContract storage loan = loanContracts[requisitionId];

        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
            return (loan.parcelsValues, true);
        }

        return (0, false);
    }

    function getRepaymentSummary(uint256 requisitionId) external view returns (
        uint256 totalRemainingDebt,
        uint256 nextPaymentAmount,
        uint256 parcelsRemaining,
        uint256 totalParcels,
        bool isActive
    ) {
        LoanContract storage loan = loanContracts[requisitionId];
        LoanRequisition storage req = loanRequisitions[requisitionId];

        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
            totalRemainingDebt = loan.parcelsPending * loan.parcelsValues;
            nextPaymentAmount = loan.parcelsValues;
            parcelsRemaining = loan.parcelsPending;
            totalParcels = req.parcelsCount;
            isActive = true;
        } else {
            totalRemainingDebt = 0;
            nextPaymentAmount = 0;
            parcelsRemaining = 0;
            totalParcels = req.parcelsCount;
            isActive = false;
        }

        return (totalRemainingDebt, nextPaymentAmount, parcelsRemaining, totalParcels, isActive);
    }

    function canPayRequisition(uint256 requisitionId, address borrower) external view returns (bool) {
        LoanContract storage loan = loanContracts[requisitionId];

        return (
            loan.walletAddress == borrower &&
            loan.status == ContractStatus.Active &&
            loan.parcelsPending > 0
        );
    }

    function getPaymentDates(uint256 requisitionId) external view returns (uint256[] memory) {
        return loanContracts[requisitionId].paymentDates;
    }

    function getDonationsInCoverage(address lender) external view returns (uint256) {
        return donationsInCoverage[lender];
    }

    function getUSDTBalance() external view returns (uint256) {
        return IERC20(usdtToken).balanceOf(address(this));
    }

    function getAllowance(address user) external view returns (uint256) {
        return IERC20(usdtToken).allowance(user, address(this));
    }

    function getAvailableBorrowAmount() external view returns (uint256) {
        return availableBalance;
    }

    function getTotalDonations() external view returns (uint256) {
        return totalDonations;
    }

    function getTotalBorrowed() external view returns (uint256) {
        return totalBorrowed;
    }

    function getAvailableBalance() external view returns (uint256) {
        return availableBalance;
    }

    function getContractBalance() external view returns (uint256) {
        return IERC20(usdtToken).balanceOf(address(this));
    }

    function getDonation(address _user) external view returns (uint256) {
        return donations[_user];
    }

    function getBorrowing(address _user) external view returns (uint256) {
        return borrowings[_user];
    }

    function getLastBorrowTime(address _user) external view returns (uint256) {
        return lastBorrowTime[_user];
    }

    function getCoveringLenders(uint256 requisitionId) external view returns (address[] memory) {
        return loanRequisitions[requisitionId].coveringLenders;
    }

    function getLenderCoverage(uint256 requisitionId, address lender) external view returns (uint256) {
        return loanRequisitions[requisitionId].coverageAmounts[lender];
    }

    function getBorrowerRequisitions(address borrower) external view returns (uint256[] memory) {
        return borrowerRequisitions[borrower];
    }

    function getRequisitionInfo(uint256 requisitionId) external view returns (RequisitionInfo memory) {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        return RequisitionInfo({
            requisitionId: req.requisitionId,
            borrower: req.borrower,
            amount: req.amount,
            minimumCoverage: req.minimumCoverage,
            currentCoverage: req.currentCoverage,
            status: req.status,
            creationTime: req.creationTime,
            coveringLenders: req.coveringLenders,
            parcelsCount: req.parcelsCount
        });
    }

    function getLoanContract(uint256 requisitionId) external view returns (LoanContract memory) {
        return loanContracts[requisitionId];
    }

    function canUserBorrow(address _user, uint256 _amount) external view returns (bool) {
        return (
            _amount > 0 &&
            _amount <= availableBalance &&
            (donations[_user] >= MIN_DONATION_FOR_BORROW || borrowings[_user] == 0) &&
            (lastBorrowTime[_user] + BORROW_DURATION < block.timestamp || borrowings[_user] == 0)
        );
    }
}