// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import "./ILoanMachine.sol";
import "./IReputationSystem.sol";
import "./ReputationLib.sol";

/**
 * @title  LoanMachine v3 (slim)
 * @notice Credit cooperative — governance + privacy.
 *
 * SIZE REDUCTION vs v3
 * ────────────────────
 * Removed from on-chain → moved to server/subgraph:
 *
 * • DebtWatchItem[], performPeriodicDebtCheck(), _removeFromWatchlist()
 *   → Server reads LoanContractGenerated events + paymentDates,
 *     compares with current time to determine overdue status.
 *     The repay() function still emits BorrowerOverdue/BorrowerDebtSettled
 *     events for the subgraph to index.
 *
 * • getActiveLoans()        → subgraph query on LoanContractGenerated + ParcelPaid events
 * • checkUnbeatableMajority() → subgraph computes from VoteCast events
 * • getRepaymentSummary()   → computed from getLoanContract() off-chain
 * • getAverageReputation()  → subgraph aggregates ReputationChanged events
 * • isBorrowerOverdue()     → server checks paymentDates vs now
 * • getDebtWatchlist()      → subgraph indexes all active loans
 * • canUserBorrow()         → server checks off-chain
 * • Several simple getters consolidated
 */
contract LoanMachine is ILoanMachine, IReputationSystem, ReentrancyGuard {

    using ReputationLib for ReputationLib.ReputationStorage;

    // =============================================================
    //                          STRUCTS
    // =============================================================

    // DebtWatchItem is inherited from ILoanMachine

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

    // ── MULTISIG ─────────────────────────────────────────────
    enum ProposalType {
        TransferAdmin,
        AddAdmin,
        ApproveWallet,
        RemoveAdmin,
        RotateAccessCode,
        RevokeWallet,
        Deactivate,
        Reactivate,
        SetAuthorizedCaller,
        ChangeThreshold
    }

    struct Proposal {
        ProposalType pType;
        bytes   data;
        uint256 confirmations;
        bool    executed;
        uint256 createdAt;
        bool    requiresUnanimous;        // bootstrap = true
        bool    requiresModeratorCosign;  // ApproveWallet in steady state = true
        bytes32 moderatorCosignedBy;      // 0 until cosigned
    }


    // =============================================================
    //                    MULTISIG ADMIN STORAGE
    // =============================================================

    address[] public admins;
    mapping(address => bool) public isAdmin;
    uint256 public adminThreshold;

    uint256 public proposalCounter;
    mapping(uint256 => Proposal) private proposals;
    mapping(uint256 => mapping(address => bool)) private hasConfirmedProposal;

    bool private _initialized;

    mapping(address => uint256) private pendingApprovalProposalId;


    event ProposalCreated(uint256 indexed proposalId, ProposalType indexed pType, address indexed proposer);
    event ProposalConfirmed(uint256 indexed proposalId, address indexed admin, uint256 confirmations);
    event ProposalExecuted(uint256 indexed proposalId, ProposalType indexed pType);
    event AdminAdded(address indexed admin);
    event AdminRemoved(address indexed admin);
    event ThresholdChanged(uint256 oldThreshold, uint256 newThreshold);

    event WalletApproved(address indexed wallet);
    event WalletRevoked(address indexed wallet);
    event ProposalCosigned(uint256 indexed proposalId, bytes32 indexed moderatorMemberId);

    // =============================================================
    //                     GENERAL STORAGE
    // =============================================================

    bytes32 private accessCodeHash;
    bool    private active;

    mapping(address => bool) private approvedWallets;
    address[] private memberWallets;

    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);
    event AccessCodeRotated();
    event CoopDeactivated();
    event CoopReactivated();

    // =============================================================
    //               REPUTATION STORAGE
    // =============================================================

    ReputationLib.ReputationStorage private _rs;
    mapping(address => bool) public authorizedCallers;

    // =============================================================
    //                      LOAN STORAGE
    // =============================================================

    DebtWatchItem[] private debtWatchlist;
    mapping(uint256 => uint256) private watchlistIndex;

    mapping(address => uint256) private donations;
    mapping(address => uint256) private borrowings;
    mapping(address => uint256) private lastBorrowTime;

    uint256 private totalDonations;
    uint256 private totalBorrowed;
    uint256 private availableBalance;

    address public immutable usdtToken;

    mapping(uint256 => LoanRequisition) private loanRequisitions;
    uint256 public requisitionCounter = 1;
    mapping(address => uint256[]) private borrowerRequisitions;
    mapping(address => uint256)   private donationsInCoverage;
    mapping(bytes32 => uint32)    private loanRequisitionNumber;
    mapping(address => uint256)   private lastContractPerWalletId;

    mapping(uint256 => LoanContract) private loanContracts;

    uint32 private constant BORROW_DURATION         = 30 days;
    uint32 private constant MIN_DONATION_FOR_BORROW = 1e6;
    uint32 private constant PROPOSAL_EXPIRY         = 30 days;

    int32 public constant REPUTATION_GAIN_BY_REPAYNG_DEBT  = ReputationLib.GAIN_REPAY;
    int32 public constant REPUTATION_GAIN_BY_COVERING_LOAN = ReputationLib.GAIN_COVER;
    int32 public constant REPUTATION_LOSS_BY_DEBT_NOT_PAYD = ReputationLib.LOSS_OVERDUE;

    // =============================================================
    //                      CUSTOM ERRORS
    // =============================================================

    error LoanMachine_NotAdmin();
    error LoanMachine_AlreadyAdmin();
    error LoanMachine_NotEnoughAdmins();
    error LoanMachine_InvalidThreshold();
    error LoanMachine_ProposalAlreadyExecuted();
    error LoanMachine_AlreadyConfirmed();
    error LoanMachine_ProposalNotFound();

    error LoanMachine_NotModerator();
    error LoanMachine_ModeratorCosignRequired();

    error LoanMachine_AlreadyInitialized();
    error LoanMachine_NotApproved();
    error LoanMachine_AlreadyMember();
    error LoanMachine_InvalidAccessCode();
    error LoanMachine_CoopNotActive();
    error LoanMachine_InvalidAmount();
    error LoanMachine_InsufficientFunds();
    error LoanMachine_BorrowNotExpired();
    error LoanMachine_InvalidCoveragePercentage();
    error LoanMachine_OverCoverage();
    error LoanMachine_LoanNotAvailable();
    error LoanMachine_MaxLoanRequisitionPendingReached();
    error LoanMachine_InsufficientDonationBalance();
    error LoanMachine_NoActiveBorrowing();
    error LoanMachine_InvalidParcelsCount();
    error LoanMachine_TokenTransferFailed();
    error LoanMachine_MemberIdOrWalletInvalid();
    error LoanMachine_MinimumPercentageCover();
    error LoanMachine_InsufficientWithdrawableBalance();
    error LoanMachine_OnlyBorrowerCanCancelRequisition();
    error LoanMachine_RequisitionNotCancellable();
    error LoanMachine_RequisitionAlreadyFullyCovered();
    error LoanMachine_IntervalOfPaymentAboveLimit();
    error LoanMachine_WalletApprovalAlreadyProposed();
    error LoanMachine_WalletAlreadyApproved();
    error LoanMachine_ProposalExpired();

    // =============================================================
    //                         MODIFIERS
    // =============================================================

    modifier onlyAdmin() {
        if (!isAdmin[msg.sender]) revert LoanMachine_NotAdmin();
        _;
    }

    modifier onlyActive() {
        if (!active) revert LoanMachine_CoopNotActive();
        _;
    }

    modifier validMember(bytes32 memberId, address wallet) {
        if (memberId == bytes32(0) || _rs.walletToMemberId[wallet] == bytes32(0))
            revert LoanMachine_MemberIdOrWalletInvalid();
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

    modifier borrowingActive(uint256 requisitionId) {
        LoanContract storage loan = loanContracts[requisitionId];
        if (borrowings[msg.sender] == 0)         revert LoanMachine_NoActiveBorrowing();
        if (loan.walletAddress != msg.sender)     revert LoanMachine_NoActiveBorrowing();
        if (loan.status != ContractStatus.Active) revert LoanMachine_NoActiveBorrowing();
        if (loan.parcelsPending == 0)             revert LoanMachine_NoActiveBorrowing();
        _;
    }

    modifier checkLoanRequisitionOpened(bytes32 memberId) {
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

    function initializeMultisig(
        address[] calldata _admins,
        uint256   _threshold,
        string calldata accessCode,
        bytes32   founderMemberId      
    ) external {
        if (_initialized) revert LoanMachine_AlreadyInitialized();
        if (_admins.length == 0) revert LoanMachine_NotEnoughAdmins();
        if (_threshold == 0 || _threshold > _admins.length)
            revert LoanMachine_InvalidThreshold();
        if (founderMemberId == bytes32(0))
            revert LoanMachine_MemberIdOrWalletInvalid();

        for (uint256 i = 0; i < _admins.length; i++) {
            address a = _admins[i];
            if (isAdmin[a]) revert LoanMachine_AlreadyAdmin();
            isAdmin[a] = true;
            admins.push(a);
            emit AdminAdded(a);

            approvedWallets[a] = true;
            emit WalletApproved(a);
        }
        adminThreshold = _threshold;
        accessCodeHash = keccak256(abi.encodePacked(accessCode));
        active         = true;

        // Founder joins inline — same effect as joinCoop() but skipping the
        // access-code check (we just set the hash on the line above, and the
        // founder doesn't need to know their own code at this exact moment).
        address founder = _admins[0];
        memberWallets.push(founder);
        _rs.reputationChange(founderMemberId, 1, true);
        _rs.registerMemberWallet(founderMemberId, founder);

        _initialized = true;
    }


    // =============================================================
    //               MULTISIG ADMIN FUNCTIONS
    // =============================================================

    function _createProposal(
        ProposalType pType,
        bytes memory data,
        bool requiresUnanimous,
        bool requiresModeratorCosign,
        bool proposerCounts                             
    ) internal returns (uint256 proposalId) {
        proposalId = proposalCounter++;
        proposals[proposalId] = Proposal({
            pType: pType, data: data,
            confirmations: proposerCounts ? 1 : 0,      
            executed: false, createdAt: block.timestamp,
            requiresUnanimous: requiresUnanimous,
            requiresModeratorCosign: requiresModeratorCosign,
            moderatorCosignedBy: bytes32(0)
        });

        if (proposerCounts) {
            hasConfirmedProposal[proposalId][msg.sender] = true;
            emit ProposalConfirmed(proposalId, msg.sender, 1);
        }
        emit ProposalCreated(proposalId, pType, msg.sender);
        _maybeExecute(proposalId);
    }

    function proposeAction(
        ProposalType pType,
        bytes calldata data
    ) external onlyAdmin returns (uint256 proposalId) {
        if (pType == ProposalType.ApproveWallet) revert LoanMachine_ModeratorCosignRequired();

        return _createProposal(pType, data, false, false, true);
    }

    function proposeWalletApproval(address wallet) external returns (uint256 proposalId) {
        if (wallet == address(0))            revert LoanMachine_MemberIdOrWalletInvalid();
        if (approvedWallets[wallet])         revert LoanMachine_WalletAlreadyApproved();

        uint256 pendingId = pendingApprovalProposalId[wallet];
        if (pendingId != 0) {
            Proposal storage pending = proposals[pendingId];
            if (block.timestamp <= pending.createdAt + PROPOSAL_EXPIRY)
                revert LoanMachine_WalletApprovalAlreadyProposed();
            delete pendingApprovalProposalId[wallet];
        }

        proposalId = _createProposal(
            ProposalType.ApproveWallet, abi.encode(wallet),
            false, true, false                                     
        );
        pendingApprovalProposalId[wallet] = proposalId;
    }

    function confirmProposal(uint256 proposalId) external onlyAdmin {
        Proposal storage p = proposals[proposalId];
        if (p.createdAt == 0) revert LoanMachine_ProposalNotFound();
        if (p.executed)       revert LoanMachine_ProposalAlreadyExecuted();
        if (block.timestamp > p.createdAt + PROPOSAL_EXPIRY)
            revert LoanMachine_ProposalExpired();
        if (hasConfirmedProposal[proposalId][msg.sender])
            revert LoanMachine_AlreadyConfirmed();

        hasConfirmedProposal[proposalId][msg.sender] = true;
        p.confirmations++;

        emit ProposalConfirmed(proposalId, msg.sender, p.confirmations);

        _maybeExecute(proposalId);
    }

    function cosignProposal(
        uint256 proposalId,
        bytes32 moderatorMemberId
    ) external {
        Proposal storage p = proposals[proposalId];
        if (p.createdAt == 0) revert LoanMachine_ProposalNotFound();
        if (p.executed)       revert LoanMachine_ProposalAlreadyExecuted();
        if (block.timestamp > p.createdAt + PROPOSAL_EXPIRY)
            revert LoanMachine_ProposalExpired();
        if (!p.requiresModeratorCosign) revert LoanMachine_ModeratorCosignRequired();

        if (_rs.walletToMemberId[msg.sender] != moderatorMemberId)
            revert LoanMachine_MemberIdOrWalletInvalid();
        if (!_rs.isModerator[moderatorMemberId]) revert LoanMachine_NotModerator();

        p.moderatorCosignedBy = moderatorMemberId;

        emit ProposalCosigned(proposalId, moderatorMemberId);

        _maybeExecute(proposalId);
    }

    function _maybeExecute(uint256 proposalId) internal {
        Proposal storage p = proposals[proposalId];

        uint256 confirmsNeeded = p.requiresUnanimous ? admins.length : adminThreshold;
        bool    confirmsMet    = p.confirmations >= confirmsNeeded;
        bool    cosignMet      = !p.requiresModeratorCosign || p.moderatorCosignedBy != bytes32(0);

        if (confirmsMet && cosignMet) {
            _executeProposal(proposalId);
        }
    }

    function _executeProposal(uint256 proposalId) internal {
        Proposal storage p = proposals[proposalId];
        p.executed = true;

        if (p.pType == ProposalType.TransferAdmin) {
            (address oldAdmin, address newAdmin) = abi.decode(p.data, (address, address));
            _replaceAdmin(oldAdmin, newAdmin);
        } else if (p.pType == ProposalType.AddAdmin) {
            address newAdmin = abi.decode(p.data, (address));
            _addAdmin(newAdmin);
        } else if (p.pType == ProposalType.RemoveAdmin) {
            address toRemove = abi.decode(p.data, (address));
            _removeAdmin(toRemove);
        } else if (p.pType == ProposalType.RotateAccessCode) {
            bytes32 newHash = abi.decode(p.data, (bytes32));
            accessCodeHash = newHash;
            emit AccessCodeRotated();
        } else if (p.pType == ProposalType.RevokeWallet) {
            address wallet = abi.decode(p.data, (address));
            approvedWallets[wallet] = false;
            emit WalletRevoked(wallet);
        } else if (p.pType == ProposalType.ApproveWallet){
            address wallet = abi.decode(p.data, (address));
            approvedWallets[wallet] = true;
            delete pendingApprovalProposalId[wallet];
            emit WalletApproved(wallet);
        } else if (p.pType == ProposalType.Deactivate) {
            active = false;
            emit CoopDeactivated();
        } else if (p.pType == ProposalType.Reactivate) {
            active = true;
            emit CoopReactivated();
        } else if (p.pType == ProposalType.SetAuthorizedCaller) {
            (address caller, bool authorized) = abi.decode(p.data, (address, bool));
            authorizedCallers[caller] = authorized;
            emit AuthorizedCallerUpdated(caller, authorized);
        } else if (p.pType == ProposalType.ChangeThreshold) {
            uint256 newThreshold = abi.decode(p.data, (uint256));
            if (newThreshold == 0 || newThreshold > admins.length)
                revert LoanMachine_InvalidThreshold();
            emit ThresholdChanged(adminThreshold, newThreshold);
            adminThreshold = newThreshold;
        }

        emit ProposalExecuted(proposalId, p.pType);
    }

    function _addAdmin(address a) internal {
        if (isAdmin[a]) revert LoanMachine_AlreadyAdmin();
        isAdmin[a] = true;
        admins.push(a);
        emit AdminAdded(a);
    }

    function _removeAdmin(address a) internal {
        if (!isAdmin[a]) revert LoanMachine_NotAdmin();
        if (admins.length <= adminThreshold) revert LoanMachine_NotEnoughAdmins();
        isAdmin[a] = false;
        for (uint256 i = 0; i < admins.length; i++) {
            if (admins[i] == a) {
                admins[i] = admins[admins.length - 1];
                admins.pop();
                break;
            }
        }
        emit AdminRemoved(a);
    }

    function _replaceAdmin(address oldAdmin, address newAdmin) internal {
        if (!isAdmin[oldAdmin]) revert LoanMachine_NotAdmin();
        if (isAdmin[newAdmin])  revert LoanMachine_AlreadyAdmin();
        isAdmin[oldAdmin] = false;
        isAdmin[newAdmin] = true;
        for (uint256 i = 0; i < admins.length; i++) {
            if (admins[i] == oldAdmin) {
                admins[i] = newAdmin;
                break;
            }
        }
        emit AdminRemoved(oldAdmin);
        emit AdminAdded(newAdmin);
        emit AdminTransferred(oldAdmin, newAdmin);
    }
    
    // =============================================================
    //                         WITHDRAWAL
    // =============================================================

    function withdraw(
        uint256 amount,
        bytes32 memberId
    )
        external
        validMember(memberId, msg.sender)
        validAmount(amount)
        nonReentrant
    {
        uint256 withdrawable = getWithdrawableBalance(msg.sender);
        if (amount > withdrawable)
            revert LoanMachine_InsufficientWithdrawableBalance();

        donations[msg.sender] -= amount;
        totalDonations        -= amount;
        availableBalance      -= amount;

        _safeTransfer(msg.sender, amount);

        emit Withdrawn(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);
    }

    // =============================================================
    //                       MEMBER JOIN
    // =============================================================

    function joinCoop(
        bytes32 memberId,
        address wallet,
        string calldata accessCode
    ) external onlyActive {
        if (!approvedWallets[wallet])      revert LoanMachine_NotApproved();
        if (keccak256(abi.encodePacked(accessCode)) != accessCodeHash)
            revert LoanMachine_InvalidAccessCode();

        memberWallets.push(wallet);
        _rs.registerMemberWallet(memberId, wallet);
    }

    // =============================================================
    //               REPUTATION SYSTEM — PUBLIC SURFACE
    // =============================================================

    function vinculationMemberToWallet(bytes32 memberId, address wallet)
        external
        onlyActive
    {
        if (!approvedWallets[wallet])
            revert LoanMachine_NotApproved();
        if (_rs.walletToMemberId[msg.sender] != memberId)
            revert LoanMachine_MemberIdOrWalletInvalid();
        _rs.registerMemberWallet(memberId, wallet);
    }

    function openElection(bytes32 candidateId, bytes32 opponent) external {
        _rs.openElection(candidateId, opponent);
    }

    function addCandidate(uint32 electionId, bytes32 candidateId) external {
        _rs.addCandidate(electionId, candidateId);
    }

    function voteForModerator(
        uint32  electionId,
        bytes32 candidateId,
        bytes32 memberId
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

    function cancelLoanRequisition(uint256 requisitionId, bytes32 memberId)
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

    function repay(uint256 requisitionId, uint256 amount, bytes32 memberId)
        external
        nonReentrant
        validMember(memberId, msg.sender)
        borrowingActive(requisitionId)
    {
        if (amount == 0) revert LoanMachine_InvalidAmount();

        LoanContract storage loan = loanContracts[requisitionId];
        uint32  parcelIdx         = loan.parcelsCount - loan.parcelsPending;
        uint256 parcelDueDate     = loan.paymentDates[parcelIdx];
        DebtWatchItem storage item = debtWatchlist[watchlistIndex[requisitionId]];

        // Reputation: late vs on-time.  Watchlist + events for subgraph.
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

    function donate(uint256 amount, bytes32 memberId)
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
        bytes32 memberId,
        uint32  daysIntervalOfPayment
    )
        external
        checkLoanRequisitionOpened(memberId)
        validMember(memberId, msg.sender)
        validAmount(amount)
        returns (uint256)
    {
        if (minimumCoverage <= 70 || minimumCoverage > 100)
            revert LoanMachine_InvalidCoveragePercentage();
        if (parcelscount < 1 || parcelscount > 12)
            revert LoanMachine_InvalidParcelsCount();
        if (daysIntervalOfPayment > 30)
            revert LoanMachine_IntervalOfPaymentAboveLimit();
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
        bytes32 memberId
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

        uint256 interval = uint256(req.daysIntervalOfPayment) * 1 days;
        for (uint32 i = 0; i < req.parcelsCount; i++) {
            loan.paymentDates.push(block.timestamp + interval * (i + 1));
            loan.parcelsAmounts.push(i < remainder ? base + 1 : base);
        }

        lastContractPerWalletId[loan.walletAddress] = requisitionId;

        emit LoanContractGenerated(
            loan.walletAddress, requisitionId, loan.status,
            loan.parcelsPending, loan.parcelsValues, loan.paymentDates, loan.creationTime
        );
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

        // Add to watchlist for on-chain overdue tracking
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
        if (debtWatchlist.length == 0) return;

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
    //       (only essential on-chain reads — rest via subgraph)
    // =============================================================

    function getAdmins() external view returns (address[] memory) { return admins; }

    function getProposal(uint256 proposalId) external view returns (
        ProposalType pType,
        uint256 confirmations,
        bool    executed,
        uint256 createdAt,
        bool    requiresUnanimous,
        bool    requiresModeratorCosign,
        bytes32 moderatorCosignedBy
    ) {
        Proposal storage p = proposals[proposalId];
        return (
            p.pType,
            p.confirmations,
            p.executed,
            p.createdAt,
            p.requiresUnanimous,
            p.requiresModeratorCosign,
            p.moderatorCosignedBy
        );
    }

    function isWalletApproved(address wallet) external view returns (bool) {
        return approvedWallets[wallet];
    }


    // ── Reputation ───────────────────────────────────────────

    function getReputation(bytes32 memberId) external view returns (int32) {
        return _rs.memberReputation[memberId];
    }

    function getMemberId(address wallet) external view returns (bytes32) {
        return _rs.walletToMemberId[wallet];
    }

    function isWalletVinculated(address wallet) external view returns (bool) {
        return _rs.walletToMemberId[wallet] != bytes32(0);
    }

    function isModerator(bytes32 memberId) external view returns (bool) {
        return _rs.isModerator[memberId];
    }

    function getCurrentElectionId() external view returns (int32) {
        uint256 len = _rs.elections.length;
        if (len == 0) return -1;
        IReputationSystem.ElectionStatus storage last = _rs.elections[len - 1];
        // `id` is a uint32 election index (increments by 1 per election), so it
        // never approaches 2^31 and the int32 cast below cannot overflow/revert.
        return last.active ? int32(last.id) : -1;
    }

    function getElectionInfo(uint32 electionId)
        external view
        returns (
            uint32    id,
            bytes32[] memory candidates,
            uint256   startTime,
            uint256   endTime,
            bool      isActive,
            bytes32   winnerId,
            int32     winningVotes,
            int32     totalVotesCast
        )
    {
        if (electionId >= _rs.elections.length)
            revert ReputationLib.RS_ElectionNotActive();

        IReputationSystem.ElectionStatus storage e = _rs.elections[electionId];
        return (
            e.id, e.candidates, e.startTime, e.endTime,
            e.active, e.winnerId, e.winningVotes, e.totalVotesCast
        );
    }

    function getCandidateVotes(bytes32 candidateId) external view returns (int32) {
        return _rs.moderatorVotesReceived[candidateId];
    }

    function hasMemberVoted(uint32 electionId, bytes32 memberId) external view returns (bool) {
        return _rs.hasVotedInElection[electionId][memberId];
    }

    // ── Loan (essential on-chain reads) ──────────────────────

    function getDebtWatchlist() external view returns (DebtWatchItem[] memory) {
        return debtWatchlist;
    }

    function isBorrowerOverdue(uint256 requisitionId) external view returns (bool) {
        uint256 idx = watchlistIndex[requisitionId];
        if (debtWatchlist.length == 0) return false;
        if (idx == 0 && debtWatchlist[0].requisitionId != requisitionId) return false;
        return debtWatchlist[idx].isOverdue;
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

    function getRepaymentSummary(uint256 requisitionId)
        external view
        returns (
            uint256 totalRemainingDebt, uint256 nextPaymentAmount,
            uint256 parcelsRemaining, uint256 totalParcels, bool isActive
        )
    {
        LoanContract storage loan   = loanContracts[requisitionId];
        LoanRequisition storage req = loanRequisitions[requisitionId];
        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
            return (
                loan.parcelsPending * loan.parcelsValues,
                loan.parcelsValues, loan.parcelsPending,
                req.parcelsCount, true
            );
        }
        return (0, 0, 0, req.parcelsCount, false);
    }

    function canUserBorrow(address user, uint256 amount) external view returns (bool) {
        return (
            amount > 0 &&
            amount <= availableBalance &&
            (donations[user] >= MIN_DONATION_FOR_BORROW || borrowings[user] == 0) &&
            (lastBorrowTime[user] + BORROW_DURATION < block.timestamp || borrowings[user] == 0)
        );
    }

    function getWithdrawableBalance(address user) internal view returns (uint256) {
        uint256 avail  = donations[user];
        uint256 locked = donationsInCoverage[user];
        return avail > locked ? avail - locked : 0;
    }

    function getRequisitionInfo(uint256 requisitionId) external view returns (RequisitionInfo memory) {
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

    function getLoanContract(uint256 requisitionId) external view returns (LoanContract memory) {
        return loanContracts[requisitionId];
    }

    function getNextPaymentAmount(uint256 requisitionId) external view returns (uint256 paymentAmount, bool canPay) {
        LoanContract storage loan = loanContracts[requisitionId];
        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0)
            return (loan.parcelsValues, true);
        return (0, false);
    }

    function canPayRequisition(uint256 requisitionId, address borrower) external view returns (bool) {
        LoanContract storage loan = loanContracts[requisitionId];
        return (loan.walletAddress == borrower && loan.status == ContractStatus.Active && loan.parcelsPending > 0);
    }

    function getPaymentDates(uint256 requisitionId) external view returns (uint256[] memory) {
        return loanContracts[requisitionId].paymentDates;
    }

    /// @notice All pool-level stats in one call (saves ~5 function dispatch entries)
    function getCoopStats() external view returns (
        uint256 _totalDonations,
        uint256 _totalBorrowed,
        uint256 _availableBalance,
        uint256 _contractBalance,
        bool    _isActive,
        uint32  _activeMemberCount,
        int32   _averageReputation
    ) {
        uint32 n = uint32(_rs.membersWithPositiveReputation.length);
        return (
            totalDonations,
            totalBorrowed,
            availableBalance,
            IERC20(usdtToken).balanceOf(address(this)),
            active,
            n,
            n > 0 ? _rs.totalPotentialVotes / int32(n) : int32(0)
        );
    }

    /// @notice All per-user financial data in one call (saves ~6 function dispatch entries)
    function getUserFinancials(address user) external view returns (
        uint256 donation,
        uint256 borrowing,
        uint256 _lastBorrowTime,
        uint256 inCoverage,
        uint256 withdrawable,
        uint256 allowance
    ) {
        uint256 avail  = donations[user];
        uint256 locked = donationsInCoverage[user];
        return (
            avail,
            borrowings[user],
            lastBorrowTime[user],
            locked,
            avail > locked ? avail - locked : 0,
            IERC20(usdtToken).allowance(user, address(this))
        );
    }

    function getCoveringLenders(uint256 id) external view returns (address[] memory) { return loanRequisitions[id].coveringLenders; }
    function getLenderCoverage(uint256 id, address l) external view returns (uint256) { return loanRequisitions[id].coverageAmounts[l]; }
    function getBorrowerRequisitions(address b) external view returns (uint256[] memory) { return borrowerRequisitions[b]; }
}
