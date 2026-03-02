// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import "./ILoanMachine.sol";
import "./IReputationSystem.sol";
import "./ReputationLib.sol";

/**
 * @title  LoanMachine
 * @notice Single-contract credit cooperative.  Reputation and election logic
 *         is organised in ReputationLib (an internal library — no separate
 *         deployment, no external calls, no new trust boundaries).
 *
 * Size reduction
 * ──────────────
 * • Optimizer enabled in hardhat.config.js (runs = 200).
 * • ReputationLib inlined by the compiler: duplicated bytecode sequences are
 *   deduplicated automatically, reducing the total size.
 * • Shadow warning fixed: getElectionInfo return param renamed to `isActive`.
 */
contract LoanMachine is ILoanMachine, IReputationSystem, ReentrancyGuard {

    using ReputationLib for ReputationLib.ReputationStorage;

    // =============================================================
    //                          STRUCTS
    // =============================================================

    struct DebtWatchItem {
        uint256 requisitionId;
        address borrower;
        uint256 nextDueDate;
        bool    isOverdue;
    }

    struct LoanRequisition {
        uint256 requisitionId;
        address borrower;
        uint256 amount;
        uint32  minimumCoverage;
        uint32  currentCoverage;
        BorrowStatus status;
        uint256 creationTime;
        address[] coveringLenders;
        mapping(address => uint256) coverageAmounts;
        uint32  parcelsCount;
        uint32  daysIntervalOfPayment;
    }

    // =============================================================
    //                       ADMIN STORAGE
    // =============================================================

    address public coopAdmin;
    bool    private _initialized;

    bytes32 private accessCodeHash;
    bool    public  active;

    mapping(address => bool) private approvedWallets;
    mapping(address => bool) public  isMember;
    address[] private memberWallets;

    // ── ADMIN EVENTS ─────────────────────────────────────────────

    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);
    event AccessCodeRotated();
    event WalletApproved(address indexed wallet);
    event WalletRevoked(address indexed wallet);
    event MemberJoined(address indexed wallet, uint32 memberId);
    event CoopDeactivated();
    event CoopReactivated();

    // =============================================================
    //               REPUTATION STORAGE (via library struct)
    // All reputation/election state lives in this single struct.
    // ReputationLib functions receive it as a storage pointer.
    // =============================================================

    ReputationLib.ReputationStorage private _rs;

    // kept for IReputationSystem interface compatibility
    mapping(address => bool) public authorizedCallers;

    // =============================================================
    //                      LOAN STORAGE
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
    mapping(address => uint256)   private donationsInCoverage;
    mapping(uint32  => uint32)    private loanRequisitionNumber;
    mapping(address => uint256)   private lastContractPerWalletId;

    mapping(uint256 => LoanContract) public loanContracts;

    uint32 private constant BORROW_DURATION         = 30 days;
    uint32 private constant MIN_DONATION_FOR_BORROW = 1e6;
    uint32 private constant MAX_DONATION            = 5e6;

    // Expose reputation constants for interface compatibility
    int32 public constant REPUTATION_GAIN_BY_REPAYNG_DEBT  = ReputationLib.GAIN_REPAY;
    int32 public constant REPUTATION_GAIN_BY_COVERING_LOAN = ReputationLib.GAIN_COVER;
    int32 public constant REPUTATION_LOSS_BY_DEBT_NOT_PAYD = ReputationLib.LOSS_OVERDUE;

    // =============================================================
    //                      CUSTOM ERRORS
    // =============================================================

    error LoanMachine_NotCoopAdmin();
    error LoanMachine_AlreadyInitialized();
    error LoanMachine_NotApproved();
    error LoanMachine_AlreadyMember();
    error LoanMachine_InvalidAccessCode();
    error LoanMachine_CoopNotActive();
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
    error LoanMachine_InvalidParcelsCount();
    error LoanMachine_ExcessiveDonation();
    error LoanMachine_TokenTransferFailed();
    error LoanMachine_MemberIdOrWalletInvalid();
    error LoanMachine_WalletAlreadyVinculated();
    error LoanMachine_MinimumPercentageCover();
    error LoanMachine_InsufficientWithdrawableBalance();
    error LoanMachine_CheckIntervalNotYetPassed();
    error LoanMachine_OnlyBorrowerCanCancelRequisition();
    error LoanMachine_RequisitionNotCancellable();
    error LoanMachine_RequisitionAlreadyFullyCovered();
    error LoanMachine_IntervalOfPaymentAboveLimit();

    // =============================================================
    //                         MODIFIERS
    // =============================================================

    modifier onlyCoopAdmin() {
        if (msg.sender != coopAdmin) revert LoanMachine_NotCoopAdmin();
        _;
    }

    modifier onlyActive() {
        if (!active) revert LoanMachine_CoopNotActive();
        _;
    }

    modifier validMember(uint32 memberId, address wallet) {
        if (memberId == 0 || _rs.walletToMemberId[wallet] == 0)
            revert LoanMachine_MemberIdOrWalletInvalid();
        _;
    }

    modifier minimumPercentCoveragePermited(uint256 minimumCoverage) {
        if (minimumCoverage <= 70 || minimumCoverage > 100)
            revert LoanMachine_InvalidCoveragePercentage();
        _;
    }

    modifier validAmount(uint256 _amount) {
        if (_amount == 0) revert LoanMachine_InvalidAmount();
        _;
    }

    modifier validCoverage(uint32 pct) {
        if (pct == 0 || pct > 100) revert LoanMachine_InvalidCoveragePercentage();
        if (pct <= 9)              revert LoanMachine_MinimumPercentageCover();
        _;
    }

    modifier maxParcelsCountAndInterval(uint32 count, uint32 interval) {
        if (count < 1 || count > 12) revert LoanMachine_InvalidParcelsCount();
        if (interval > 30)           revert LoanMachine_IntervalOfPaymentAboveLimit();
        _;
    }

    modifier borrowingActive(uint256 requisitionId) {
        LoanContract storage loan = loanContracts[requisitionId];
        if (borrowings[msg.sender] == 0)         revert LoanMachine_NoActiveBorrowing();
        if (loan.walletAddress != msg.sender)     revert LoanMachine_NoActiveBorrowing();
        if (loan.status != ContractStatus.Active) revert LoanMachine_NoActiveBorrowing();
        if (loan.parcelsPending == 0)             revert LoanMachine_NoActiveBorrowing();
        _;
    }

    modifier checkLoanRequisitionOpened(uint32 memberId) {
        if (loanRequisitionNumber[memberId] >= 3)
            revert LoanMachine_MaxLoanRequisitionPendingReached();
        _;
    }

    // =============================================================
    //                        CONSTRUCTOR
    // =============================================================

    constructor(address _usdtToken) {
        usdtToken = _usdtToken;
    }

    // =============================================================
    //                      INITIALIZATION
    // =============================================================

    function initializeAdmin(
        address _coopAdmin,
        string calldata accessCode
    ) external {
        if (_initialized) revert LoanMachine_AlreadyInitialized();
        coopAdmin      = _coopAdmin;
        accessCodeHash = keccak256(abi.encodePacked(accessCode));
        active         = true;
        _initialized   = true;
    }

    // =============================================================
    //                       ADMIN FUNCTIONS
    // =============================================================

    function transferAdmin(address newAdmin) external onlyCoopAdmin {
        emit AdminTransferred(coopAdmin, newAdmin);
        coopAdmin = newAdmin;
    }

    function rotateAccessCode(string calldata newCode) external onlyCoopAdmin {
        accessCodeHash = keccak256(abi.encodePacked(newCode));
        emit AccessCodeRotated();
    }

    function approveWallet(address wallet) external onlyCoopAdmin {
        approvedWallets[wallet] = true;
        emit WalletApproved(wallet);
    }

    function approveWalletBatch(address[] calldata wallets) external onlyCoopAdmin {
        for (uint256 i = 0; i < wallets.length; i++) {
            approvedWallets[wallets[i]] = true;
            emit WalletApproved(wallets[i]);
        }
    }

    function revokeWallet(address wallet) external onlyCoopAdmin {
        approvedWallets[wallet] = false;
        emit WalletRevoked(wallet);
    }

    function deactivateCoop() external onlyCoopAdmin {
        active = false;
        emit CoopDeactivated();
    }

    function reactivateCoop() external onlyCoopAdmin {
        active = true;
        emit CoopReactivated();
    }

    // kept for IReputationSystem interface — admin-gated here
    function setAuthorizedCaller(address caller, bool authorized) external onlyCoopAdmin {
        authorizedCallers[caller] = authorized;
        emit AuthorizedCallerUpdated(caller, authorized);
    }

    // =============================================================
    //                       MEMBER JOIN
    // =============================================================

    function joinCoop(
        uint32 memberId,
        address wallet,
        string calldata accessCode
    ) external onlyActive {
        if (!approvedWallets[wallet])
            revert LoanMachine_NotApproved();
        if (isMember[wallet])
            revert LoanMachine_AlreadyMember();
        if (keccak256(abi.encodePacked(accessCode)) != accessCodeHash)
            revert LoanMachine_InvalidAccessCode();

        isMember[wallet] = true;
        memberWallets.push(wallet);

        // library handles duplicate + cross-member checks
        _rs.registerMemberWallet(memberId, wallet);

        emit MemberJoined(wallet, memberId);
    }

    // =============================================================
    //               REPUTATION SYSTEM — PUBLIC SURFACE
    //        (thin wrappers that delegate into ReputationLib)
    // =============================================================

    function vinculationMemberToWallet(uint32 memberId, address wallet) external {
        _rs.registerMemberWallet(memberId, wallet);
    }

    function openElection(uint32 candidateId, uint32 opponent) external {
        _rs.openElection(candidateId, opponent);
    }

    function addCandidate(uint32 electionId, uint32 candidateId) external {
        _rs.addCandidate(electionId, candidateId);
    }

    function voteForModerator(
        uint32 electionId,
        uint32 candidateId,
        uint32 memberId
    )
        external
        validMember(memberId, msg.sender)
    {
        _rs.voteForModerator(electionId, candidateId, memberId);
    }

    function closeElection(uint32 electionId) external {
        _rs.closeElection(electionId);
    }

    // =============================================================
    //                   LOAN MACHINE FUNCTIONS
    // =============================================================

    function performPeriodicDebtCheck(uint256 batchSize) external nonReentrant {
        uint256 len = debtWatchlist.length;
        if (len == 0) return;

        bool intervalPassed = block.timestamp > lastPeriodicCheckTimestamp + CHECK_INTERVAL;
        if (nextCheckIndex == 0 && !intervalPassed)
            revert LoanMachine_CheckIntervalNotYetPassed();

        uint256 checked = 0;
        for (uint256 i = 0; i < batchSize && nextCheckIndex < len; i++) {
            DebtWatchItem storage item = debtWatchlist[nextCheckIndex];
            if (!item.isOverdue && block.timestamp > item.nextDueDate) {
                item.isOverdue = true;
                emit BorrowerOverdue(item.requisitionId, item.borrower, item.nextDueDate);
            }
            nextCheckIndex++;
            checked++;
        }

        if (nextCheckIndex >= len) {
            nextCheckIndex = 0;
            lastPeriodicCheckTimestamp = block.timestamp;
        }

        emit PeriodicCheckRun(checked, nextCheckIndex);
    }

    function cancelLoanRequisition(uint256 requisitionId, uint32 memberId)
        external
        nonReentrant
        validMember(memberId, msg.sender)
        returns (uint256 totalUncoveredAmount)
    {
        LoanRequisition storage req = loanRequisitions[requisitionId];

        if (req.currentCoverage == 100) revert LoanMachine_RequisitionAlreadyFullyCovered();
        if (req.borrower != msg.sender) revert LoanMachine_OnlyBorrowerCanCancelRequisition();
        if (
            req.status == BorrowStatus.Active ||
            req.status == BorrowStatus.Cancelled
        ) revert LoanMachine_RequisitionNotCancellable();

        for (uint256 i = 0; i < req.coveringLenders.length; i++) {
            address lender         = req.coveringLenders[i];
            uint256 coverageAmount = req.coverageAmounts[lender];
            if (coverageAmount > 0) {
                donationsInCoverage[lender] -= coverageAmount;
                donations[lender]           += coverageAmount;
                totalUncoveredAmount        += coverageAmount;
                delete req.coverageAmounts[lender];
                emit LoanUncovered(requisitionId, lender, coverageAmount);
            }
        }

        if (loanRequisitionNumber[memberId] > 0) loanRequisitionNumber[memberId]--;

        req.status          = BorrowStatus.Cancelled;
        req.currentCoverage = 0;

        emit LoanRequisitionCreatedCancelled(
            requisitionId, msg.sender, req.amount, req.parcelsCount, req.status
        );
    }

    function withdraw(uint256 amount, uint32 memberId)
        external
        validMember(memberId, msg.sender)
        validAmount(amount)
        nonReentrant
    {
        uint256 withdrawable = getWithdrawableBalance(msg.sender);
        if (amount > withdrawable) revert LoanMachine_InsufficientWithdrawableBalance();

        donations[msg.sender] -= amount;
        totalDonations        -= amount;
        availableBalance      -= amount;

        _safeTransfer(msg.sender, amount);

        emit Withdrawn(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);
    }

    function repay(uint256 requisitionId, uint256 amount, uint32 memberId)
        external
        nonReentrant
        validMember(memberId, msg.sender)
        borrowingActive(requisitionId)
    {
        if (amount == 0) revert LoanMachine_InvalidAmount();

        LoanContract storage loan  = loanContracts[requisitionId];
        uint32  parcelIdx          = loan.parcelsCount - loan.parcelsPending;
        uint256 parcelDueDate      = loan.paymentDates[parcelIdx];
        DebtWatchItem storage item = debtWatchlist[watchlistIndex[requisitionId]];

        if (block.timestamp > parcelDueDate) {
            _rs.reputationChange(memberId, REPUTATION_LOSS_BY_DEBT_NOT_PAYD, false);
            if (!item.isOverdue) {
                item.isOverdue = true;
                emit BorrowerOverdue(requisitionId, msg.sender, parcelDueDate);
            }
        } else {
            _rs.reputationChange(memberId, REPUTATION_GAIN_BY_REPAYNG_DEBT, true);
            if (item.isOverdue) {
                item.isOverdue = false;
                emit BorrowerDebtSettled(requisitionId, msg.sender);
            }
        }

        if (amount != loan.parcelsValues) revert LoanMachine_InvalidAmount();

        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        borrowings[msg.sender] -= amount;
        totalBorrowed          -= amount;
        availableBalance       += amount;

        _distributeRepaymentToLenders(requisitionId, amount);

        loan.parcelsPending--;
        if (loan.parcelsPending == 0) {
            loan.status = ContractStatus.Closed;
            emit LoanCompleted(requisitionId);
            _removeFromWatchlist(requisitionId);
        } else {
            uint32 nextIdx   = loan.parcelsCount - loan.parcelsPending;
            item.nextDueDate = loan.paymentDates[nextIdx];
            item.isOverdue   = false;
        }

        emit Repaid(msg.sender, amount, borrowings[msg.sender]);
        emit ParcelPaid(requisitionId, loan.parcelsPending);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
    }

    function donate(uint256 amount, uint32 memberId)
        external
        validMember(memberId, msg.sender)
        validAmount(amount)
        nonReentrant
    {
        bool isNewDonor = donations[msg.sender] == 0;

        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        donations[msg.sender] += amount;
        totalDonations        += amount;
        availableBalance      += amount;

        emit Donated(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);
        if (isNewDonor) emit NewDonor(msg.sender);
    }

    function createLoanRequisition(
        uint256 amount,
        uint32  minimumCoverage,
        uint32  parcelscount,
        uint32  memberId,
        uint32  daysIntervalOfPayment
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

        uint256 lastId = lastContractPerWalletId[msg.sender];
        if (lastId != 0) {
            if (loanContracts[lastId].creationTime + BORROW_DURATION > block.timestamp)
                revert LoanMachine_BorrowNotExpired();
        }

        uint256 requisitionId = requisitionCounter++;

        LoanRequisition storage req = loanRequisitions[requisitionId];
        req.requisitionId         = requisitionId;
        req.borrower              = msg.sender;
        req.amount                = amount;
        req.minimumCoverage       = minimumCoverage;
        req.currentCoverage       = 0;
        req.status                = BorrowStatus.Pending;
        req.creationTime          = block.timestamp;
        req.parcelsCount          = parcelscount;
        req.daysIntervalOfPayment = daysIntervalOfPayment;

        borrowerRequisitions[msg.sender].push(requisitionId);
        loanRequisitionNumber[memberId]++;

        emit LoanRequisitionCreatedCancelled(
            requisitionId, msg.sender, amount, parcelscount, req.status
        );
        return requisitionId;
    }

    function coverLoan(
        uint256 requisitionId,
        uint32  coveragePercentage,
        uint32  memberId
    )
        external
        validMember(memberId, msg.sender)
        validCoverage(coveragePercentage)
        nonReentrant
    {
        LoanRequisition storage req = loanRequisitions[requisitionId];

        if (
            req.status != BorrowStatus.Pending &&
            req.status != BorrowStatus.PartiallyCovered
        ) revert LoanMachine_LoanNotAvailable();

        if (uint256(req.currentCoverage) + uint256(coveragePercentage) > 100)
            revert LoanMachine_OverCoverage();

        uint256 lastId = lastContractPerWalletId[req.borrower];
        if (lastId != 0) {
            if (loanContracts[lastId].creationTime + BORROW_DURATION > block.timestamp)
                revert LoanMachine_BorrowNotExpired();
        }

        // Ceiling division — prevents dust shortfall in coverage
        uint256 PRECISION   = 1e18;
        uint256 coverAmount =
            ((req.amount * coveragePercentage * PRECISION) + (100 * PRECISION - 1)) /
            (100 * PRECISION);

        if (donations[msg.sender] < coverAmount)
            revert LoanMachine_InsufficientDonationBalance();

        donations[msg.sender]           -= coverAmount;
        donationsInCoverage[msg.sender] += coverAmount;
        req.currentCoverage             += coveragePercentage;

        if (req.coverageAmounts[msg.sender] == 0) {
            req.coveringLenders.push(msg.sender);
        }
        req.coverageAmounts[msg.sender] += coverAmount;

        if (req.currentCoverage >= req.minimumCoverage) {
            req.status = BorrowStatus.FullyCovered;
            loanRequisitionNumber[_rs.walletToMemberId[req.borrower]] = 0;
            _generateLoanContract(requisitionId);
            _fundLoan(requisitionId);
        } else {
            req.status = BorrowStatus.PartiallyCovered;
        }

        int32 gain = (REPUTATION_GAIN_BY_COVERING_LOAN * int32(coveragePercentage)) / 10;
        _rs.reputationChange(memberId, gain, true);

        emit LoanCovered(requisitionId, msg.sender, coverAmount);
    }

    // =============================================================
    //                      INTERNAL HELPERS
    // =============================================================

    function _safeTransfer(address to, uint256 amount) internal {
        bool success = IERC20(usdtToken).transfer(to, amount);
        if (!success) revert LoanMachine_TokenTransferFailed();
    }

    function _generateLoanContract(uint256 requisitionId) internal {
        LoanContract storage loan   = loanContracts[requisitionId];
        LoanRequisition storage req = loanRequisitions[requisitionId];

        loan.walletAddress  = req.borrower;
        loan.requisitionId  = requisitionId;
        loan.status         = ContractStatus.Active;
        loan.parcelsCount   = req.parcelsCount;
        loan.parcelsPending = req.parcelsCount;
        loan.creationTime   = block.timestamp;

        uint256 base      = req.amount / req.parcelsCount;
        uint256 remainder = req.amount % req.parcelsCount;
        loan.parcelsValues = base;

        _generatePaymentDates(loan, req.parcelsCount, req.daysIntervalOfPayment);
        lastContractPerWalletId[loan.walletAddress] = requisitionId;

        for (uint32 i = 0; i < req.parcelsCount; i++) {
            loan.parcelsAmounts.push(i < remainder ? base + 1 : base);
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

    function _generatePaymentDates(
        LoanContract storage loan,
        uint32 count,
        uint32 intervalDays
    ) internal {
        uint256 interval = uint256(intervalDays) * 1 days;
        uint256 start    = block.timestamp;
        for (uint32 i = 0; i < count; i++) {
            loan.paymentDates.push(start + interval * (i + 1));
        }
    }

    function _fundLoan(uint256 requisitionId) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        address borrower = req.borrower;

        borrowings[borrower]     += req.amount;
        totalBorrowed            += req.amount;
        availableBalance         -= req.amount;
        lastBorrowTime[borrower]  = block.timestamp;
        req.status = BorrowStatus.Active;

        _safeTransfer(borrower, req.amount);

        uint256 firstDue = loanContracts[requisitionId].paymentDates[0];
        watchlistIndex[requisitionId] = debtWatchlist.length;
        debtWatchlist.push(DebtWatchItem({
            requisitionId: requisitionId,
            borrower:      borrower,
            nextDueDate:   firstDue,
            isOverdue:     false
        }));

        emit Borrowed(borrower, req.amount, borrowings[borrower]);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
        emit LoanFunded(requisitionId);
    }

    function _distributeRepaymentToLenders(
        uint256 requisitionId,
        uint256 repaymentAmount
    ) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        uint256 total = req.amount;

        for (uint256 i = 0; i < req.coveringLenders.length; i++) {
            address lender = req.coveringLenders[i];
            uint256 share  = (repaymentAmount * req.coverageAmounts[lender]) / total;
            if (share == 0) continue;
            if (share > donationsInCoverage[lender]) share = donationsInCoverage[lender];
            donationsInCoverage[lender] -= share;
            donations[lender]           += share;
            emit LenderRepaid(requisitionId, lender, share);
        }
    }

    function _removeFromWatchlist(uint256 requisitionId) internal {
        uint256 idx  = watchlistIndex[requisitionId];
        uint256 last = debtWatchlist.length - 1;
        if (idx != last) {
            DebtWatchItem storage lastItem = debtWatchlist[last];
            debtWatchlist[idx]                     = lastItem;
            watchlistIndex[lastItem.requisitionId] = idx;
        }
        debtWatchlist.pop();
        delete watchlistIndex[requisitionId];
    }

    // =============================================================
    //                       VIEW FUNCTIONS
    // =============================================================

    // ── Admin / member ────────────────────────────────────────────

    function isWalletApproved(address wallet) external view returns (bool) {
        return approvedWallets[wallet];
    }

    function getMemberWallets() external view returns (address[] memory) {
        return memberWallets;
    }

    function isCoopActive() external view returns (bool) { return active; }

    // ── Reputation (reads from _rs struct) ───────────────────────

    function getReputation(uint32 memberId) external view returns (int32) {
        return _rs.memberReputation[memberId];
    }

    function getMemberId(address wallet) external view returns (uint32) {
        return _rs.walletToMemberId[wallet];
    }

    function isWalletVinculated(address wallet) external view returns (bool) {
        return _rs.walletToMemberId[wallet] != 0;
    }

    function getActiveMemberCount() external view returns (uint32) {
        return uint32(_rs.membersWithPositiveReputation.length);
    }

    function getAverageReputation() external view returns (int32) {
        uint32 n = uint32(_rs.membersWithPositiveReputation.length);
        if (n == 0) return 0;
        return _rs.totalPotentialVotes / int32(n);
    }

    function isMemberActive(uint32 memberId) external view returns (bool) {
        return _rs.activeMembers[memberId];
    }

    function getCandidateVotes(uint32 candidateId) external view returns (int32) {
        return _rs.moderatorVotesReceived[candidateId];
    }

    function hasMemberVoted(uint32 electionId, uint32 memberId)
        external view returns (bool)
    {
        return _rs.hasVotedInElection[electionId][memberId];
    }

    function isModerator(uint32 memberId) external view returns (bool) {
        return _rs.isModerator[memberId];
    }

    function getCurrentElectionId() external view returns (int32) {
        uint256 len = _rs.elections.length;
        if (len == 0) return -1;
        IReputationSystem.ElectionStatus storage last = _rs.elections[len - 1];
        return last.active ? int32(last.id) : -1;
    }

    /**
     * @dev Shadow warning FIXED: return variable renamed from `active` to
     *      `isActive` so it no longer shadows the state var `bool public active`.
     */
    function getElectionInfo(uint32 electionId)
        external
        view
        returns (
            uint32   id,
            uint32[] memory candidates,
            uint256  startTime,
            uint256  endTime,
            bool     isActive,       // ← was `active`, shadowed state var
            uint32   winnerId,
            int32    winningVotes,
            int32    totalVotesCast
        )
    {
        if (electionId >= _rs.elections.length)
            revert ReputationLib.RS_ElectionNotActive();

        IReputationSystem.ElectionStatus storage e = _rs.elections[electionId];
        return (
            e.id,
            e.candidates,
            e.startTime,
            e.endTime,
            e.active,
            e.winnerId,
            e.winningVotes,
            e.totalVotesCast
        );
    }

    function checkUnbeatableMajority(uint32 electionId)
        external
        view
        returns (
            bool   isUnbeatable,
            uint32 leadingCandidate,
            int32  leadingVotes,
            uint32 secondCandidate,
            int32  secondVotes
        )
    {
        if (electionId >= _rs.elections.length)
            revert ReputationLib.RS_ElectionNotActive();

        IReputationSystem.ElectionStatus storage e = _rs.elections[electionId];
        if (e.candidates.length < 2) return (false, 0, 0, 0, 0);

        (leadingCandidate, leadingVotes, secondCandidate, secondVotes) =
            _rs.getTopTwoCandidatesView(electionId);

        isUnbeatable = (leadingVotes > secondVotes + e.potentialRemainingVotes);
    }

    // ── Loan ─────────────────────────────────────────────────────

    function getWithdrawableBalance(address user) public view returns (uint256) {
        uint256 avail  = donations[user];
        uint256 locked = donationsInCoverage[user];
        return avail > locked ? avail - locked : 0;
    }

    function getActiveLoans(address borrower)
        public view
        returns (LoanContract[] memory activeLoans, uint256[] memory ids)
    {
        uint256[] storage all = borrowerRequisitions[borrower];
        uint256 count = 0;
        for (uint256 i = 0; i < all.length; i++) {
            LoanContract storage l = loanContracts[all[i]];
            if (l.walletAddress == borrower && l.status == ContractStatus.Active && l.parcelsPending > 0)
                count++;
        }
        activeLoans = new LoanContract[](count);
        ids         = new uint256[](count);
        uint256 idx = 0;
        for (uint256 i = 0; i < all.length; i++) {
            LoanContract storage l = loanContracts[all[i]];
            if (l.walletAddress == borrower && l.status == ContractStatus.Active && l.parcelsPending > 0) {
                activeLoans[idx] = l;
                ids[idx]         = all[i];
                idx++;
            }
        }
    }

    function getDebtWatchlist() external view returns (DebtWatchItem[] memory) {
        return debtWatchlist;
    }

    function isBorrowerOverdue(uint256 requisitionId) external view returns (bool) {
        uint256 idx = watchlistIndex[requisitionId];
        if (debtWatchlist.length == 0) return false;
        if (idx == 0 && debtWatchlist[0].requisitionId != requisitionId) return false;
        return debtWatchlist[idx].isOverdue;
    }

    function getNextPaymentAmount(uint256 requisitionId)
        external view returns (uint256 paymentAmount, bool canPay)
    {
        LoanContract storage loan = loanContracts[requisitionId];
        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0)
            return (loan.parcelsValues, true);
        return (0, false);
    }

    function getRepaymentSummary(uint256 requisitionId)
        external view
        returns (
            uint256 totalRemainingDebt,
            uint256 nextPaymentAmount,
            uint256 parcelsRemaining,
            uint256 totalParcels,
            bool    isActive
        )
    {
        LoanContract storage loan   = loanContracts[requisitionId];
        LoanRequisition storage req = loanRequisitions[requisitionId];
        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
            return (
                loan.parcelsPending * loan.parcelsValues,
                loan.parcelsValues,
                loan.parcelsPending,
                req.parcelsCount,
                true
            );
        }
        return (0, 0, 0, req.parcelsCount, false);
    }

    function canPayRequisition(uint256 requisitionId, address borrower)
        external view returns (bool)
    {
        LoanContract storage loan = loanContracts[requisitionId];
        return (
            loan.walletAddress  == borrower &&
            loan.status         == ContractStatus.Active &&
            loan.parcelsPending > 0
        );
    }

    function getPaymentDates(uint256 requisitionId)
        external view returns (uint256[] memory)
    { return loanContracts[requisitionId].paymentDates; }

    function getDonationsInCoverage(address lender)
        external view returns (uint256)
    { return donationsInCoverage[lender]; }

    function getUSDTBalance()           external view returns (uint256) { return IERC20(usdtToken).balanceOf(address(this)); }
    function getAllowance(address user)  external view returns (uint256) { return IERC20(usdtToken).allowance(user, address(this)); }
    function getAvailableBorrowAmount() external view returns (uint256) { return availableBalance; }
    function getTotalDonations()        external view returns (uint256) { return totalDonations; }
    function getTotalBorrowed()         external view returns (uint256) { return totalBorrowed; }
    function getAvailableBalance()      external view returns (uint256) { return availableBalance; }
    function getContractBalance()       external view returns (uint256) { return IERC20(usdtToken).balanceOf(address(this)); }
    function getDonation(address user)  external view returns (uint256) { return donations[user]; }
    function getBorrowing(address user) external view returns (uint256) { return borrowings[user]; }
    function getLastBorrowTime(address user) external view returns (uint256) { return lastBorrowTime[user]; }

    function getCoveringLenders(uint256 requisitionId)
        external view returns (address[] memory)
    { return loanRequisitions[requisitionId].coveringLenders; }

    function getLenderCoverage(uint256 requisitionId, address lender)
        external view returns (uint256)
    { return loanRequisitions[requisitionId].coverageAmounts[lender]; }

    function getBorrowerRequisitions(address borrower)
        external view returns (uint256[] memory)
    { return borrowerRequisitions[borrower]; }

    function getRequisitionInfo(uint256 requisitionId)
        external view returns (RequisitionInfo memory)
    {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        return RequisitionInfo({
            requisitionId:   req.requisitionId,
            borrower:        req.borrower,
            amount:          req.amount,
            minimumCoverage: req.minimumCoverage,
            currentCoverage: req.currentCoverage,
            status:          req.status,
            creationTime:    req.creationTime,
            coveringLenders: req.coveringLenders,
            parcelsCount:    req.parcelsCount
        });
    }

    function getLoanContract(uint256 requisitionId)
        external view returns (LoanContract memory)
    { return loanContracts[requisitionId]; }

    function canUserBorrow(address user, uint256 amount)
        external view returns (bool)
    {
        return (
            amount > 0 &&
            amount <= availableBalance &&
            (donations[user] >= MIN_DONATION_FOR_BORROW || borrowings[user] == 0) &&
            (lastBorrowTime[user] + BORROW_DURATION < block.timestamp || borrowings[user] == 0)
        );
    }
}
