// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "./interfaces/ILoanMachine.sol";
import "./ReputationSystem.sol";
import "./libraries/DebtTracker.sol";

contract LoanMachine is ILoanMachine, ReentrancyGuard {
    using DebtTracker for DebtTracker.DebtStorage;

    // Reputation system integration
    ReputationSystem public reputationSystem;
    
    // Debt tracker
    DebtTracker.DebtStorage private debtStorage;

    // Track all borrowers for monthly updates
    address[] private allBorrowers;
    mapping(address => bool) private isRegisteredBorrower;

    // State variables
    mapping(address => uint256) private donations;
    mapping(address => uint256) private borrowings;
    mapping(address => uint256) private lastBorrowTime;

    uint256 private totalDonations;
    uint256 private totalBorrowed;
    uint256 private availableBalance;

    // USDT Variable
    address public immutable usdtToken; 

    // Loan requisition system
    mapping(uint256 => LoanRequisition) private loanRequisitions;
    uint256 public requisitionCounter;
    mapping(address => uint256[]) public borrowerRequisitions;
    mapping(address => uint256) private donationsInCoverage;

    // Loan Contracts
    mapping(uint256 => LoanContract) public loanContracts;

    // Constants
    uint32 private constant BORROW_DURATION = 7 days;
    uint32 private constant MIN_DONATION_FOR_BORROW = 1e6; // 1 USDT (6 decimals)
    uint32 private constant MAX_DONATION = 5e6; // 5 USDT (6 decimals)

    // Custom errors
    error InvalidAmount();
    error InsufficientFunds();
    error MinimumDonationRequired();
    error BorrowNotExpired();
    error InvalidCoveragePercentage();
    error OverCoverage();
    error LoanNotAvailable();
    error InsufficientDonationBalance();
    error NoActiveBorrowing();
    error RepaymentExceedsBorrowed();
    error InvalidParcelsCount();
    error ExcessiveDonation();
    error TokenTransferFailed();
    error MemberIdOrWalletInvalid();
    error WalletAlreadyVinculated();
    error ParcelAlreadyPaid();

    // Structs
    struct LoanRequisition {    
        uint256 requisitionId;
        address borrower;
        uint256 amount;
        uint32 minimumCoverage;
        uint32 currentCoverage;
        BorrowStatus status;
        uint256 durationDays;
        uint256 creationTime;
        address[] coveringLenders;
        mapping(address => uint256) coverageAmounts;
        uint32 parcelsCount;
    }

    modifier minimumPercentCoveragePermited(uint256 minimumCoverage) {
        if (minimumCoverage <= 70 || minimumCoverage > 100) revert InvalidCoveragePercentage();
        _;
    }

    modifier validAmount(uint256 _amount) {
        if (_amount == 0) revert InvalidAmount();
        _;
    }

    modifier canBorrow(uint256 _amount) {
        if (_amount > availableBalance) revert InsufficientFunds();
        if (donations[msg.sender] < MIN_DONATION_FOR_BORROW && borrowings[msg.sender] != 0) {
            revert MinimumDonationRequired();
        }
        if (lastBorrowTime[msg.sender] + BORROW_DURATION >= block.timestamp && borrowings[msg.sender] != 0) {
            revert BorrowNotExpired();
        }
        _;
    }

    modifier validCoverage(uint32 coveragePercentage) {
        if (coveragePercentage == 0 || coveragePercentage > 100) revert InvalidCoveragePercentage();
        _;
    }

    modifier maxParcelsCount(uint32 parcelscount) {
        if (parcelscount < 1 || parcelscount > 12) revert InvalidParcelsCount();
        _;
    }

    modifier validMember(uint32 memberId, address wallet) {
        if(memberId == 0 || !reputationSystem.isWalletVinculated(wallet)) 
            revert MemberIdOrWalletInvalid();
        _;
    }

    modifier borrowingActive(uint256 requisitionId) {
        LoanContract storage loan = loanContracts[requisitionId];
        if (borrowings[msg.sender] == 0) revert NoActiveBorrowing();
        if (loan.walletAddress != msg.sender) revert NoActiveBorrowing();
        if (loan.status != ContractStatus.Active) revert NoActiveBorrowing();
        if (loan.parcelsPending == 0) revert NoActiveBorrowing();
        _;
    }

    constructor(address _usdtToken, address _reputationSystem) {
        usdtToken = _usdtToken;
        reputationSystem = ReputationSystem(_reputationSystem);
        debtStorage.initialize();
    }
    
    /**
     * @dev Repay loan with auto debt status update
     */
    function repay(uint256 requisitionId, uint256 amount, uint32 memberId) 
        validMember(memberId, msg.sender) 
        borrowingActive(requisitionId) 
        external 
        nonReentrant 
    {
        // Auto-trigger monthly update if needed
        debtStorage.checkAndTriggerMonthlyUpdate(allBorrowers, this.getActiveLoans);

        if (amount == 0) revert InvalidAmount();

        LoanContract storage loan = loanContracts[requisitionId];
        address borrower = msg.sender;
        
        uint32 currentParcelIndex = loan.parcelsCount - loan.parcelsPending;
        uint256 currentParcelDueDate = loan.paymentDates[currentParcelIndex];
        
        // Update reputation based on payment timing
        if (block.timestamp > currentParcelDueDate) {
            reputationSystem.reputationChange(memberId, reputationSystem.REPUTATION_LOSS_BY_DEBT_NOT_PAYD(), false);
        } else {
            reputationSystem.reputationChange(memberId, reputationSystem.REPUTATION_GAIN_BY_REPAYNG_DEBT(), true);
        }

        if (amount != loan.parcelsValues) revert InvalidAmount();
        
        // Transfer USDT from borrower to contract
        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert TokenTransferFailed();

        // Update global borrowing state
        borrowings[borrower] -= amount;
        totalBorrowed -= amount;
        availableBalance += amount;

        _distributeRepaymentToLenders(requisitionId, amount);

        emit Repaid(borrower, amount, borrowings[borrower]);
        emit ParcelPaid(requisitionId, loan.parcelsPending);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
        
        loan.parcelsPending -= 1;
        if (loan.parcelsPending == 0) {
            loan.status = ContractStatus.Closed;
            emit LoanCompleted(requisitionId);
        }
        
        // Update debt status after repayment
        debtStorage.updateBorrowerDebtStatus(borrower, this.getActiveLoans);
    }

    function vinculationMemberToWallet(uint32 memberId, address wallet) external {
        reputationSystem.vinculationMemberToWallet(memberId, wallet); 
    }

    /**
     * @dev Donate function with auto debt status update
     */
    function donate(uint256 amount, uint32 memberId) 
        validMember(memberId, msg.sender) 
        external 
        validAmount(amount) 
        nonReentrant 
    {
        // Auto-trigger monthly update if needed
        debtStorage.checkAndTriggerMonthlyUpdate(allBorrowers, this.getActiveLoans);

        // Check donation limit
        if (amount > MAX_DONATION) {
            revert ExcessiveDonation();
        }

        // Transfer USDT from user to contract
        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert TokenTransferFailed();

        bool isNewDonor = donations[msg.sender] == 0;
        
        unchecked {
            donations[msg.sender] += amount;
            totalDonations += amount;
            availableBalance += amount;
        }
        
        emit Donated(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);
        
        if (isNewDonor) {
            emit NewDonor(msg.sender);
        }
    }

    // Create loan requisition
    function createLoanRequisition(
        uint256 _amount,
        uint32 _minimumCoverage,
        uint256 _durationDays,
        uint32 parcelscount,
        uint32 memberId
    ) 
        external 
        validMember(memberId, msg.sender) 
        validAmount(_amount) 
        minimumPercentCoveragePermited(_minimumCoverage) 
        maxParcelsCount(parcelscount) 
        returns (uint256) 
    {
        if (_amount > availableBalance) revert InsufficientFunds();
        
        uint256 requisitionId = requisitionCounter++;
        
        LoanRequisition storage newReq = loanRequisitions[requisitionId];
        newReq.borrower = msg.sender;
        newReq.amount = _amount;
        newReq.minimumCoverage = _minimumCoverage;
        newReq.currentCoverage = 0;
        newReq.status = BorrowStatus.Pending;
        newReq.durationDays = _durationDays;
        newReq.creationTime = block.timestamp;
        newReq.parcelsCount = parcelscount;
        newReq.requisitionId = requisitionId;
        borrowerRequisitions[msg.sender].push(requisitionId);
        
        emit LoanRequisitionCreated(requisitionId, msg.sender, _amount, parcelscount);
        return requisitionId;
    }

    // Cover a loan requisition
    function coverLoan(uint256 requisitionId, uint32 coveragePercentage, uint32 memberId) 
        external 
        validMember(memberId, msg.sender)
        validCoverage(coveragePercentage)
        nonReentrant 
    {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        
        if (req.status != BorrowStatus.Pending && req.status != BorrowStatus.PartiallyCovered) {
            revert LoanNotAvailable();
        }
        
        if (uint256(req.currentCoverage) + uint256(coveragePercentage) > 100) {
            revert OverCoverage();
        }

        uint256 PRECISION = 1e18;
        uint256 coverageAmount = ((req.amount * coveragePercentage * PRECISION) + (100 * PRECISION - 1)) / (100 * PRECISION);
        
        if (donations[msg.sender] < coverageAmount) {
            revert InsufficientDonationBalance();
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
            _generateLoanContract(requisitionId);
            _fundLoan(requisitionId);
        } else {
            req.status = BorrowStatus.PartiallyCovered;
        }
        
        reputationSystem.reputationChange(memberId, int32(reputationSystem.REPUTATION_GAIN_BY_COVERING_LOAN()), true);
        emit LoanCovered(requisitionId, msg.sender, coverageAmount);
    }

    function _generateLoanContract(uint256 requisitionId) internal {
        LoanContract storage loan = loanContracts[requisitionId];
        loan.walletAddress = loanRequisitions[requisitionId].borrower;
        loan.requisitionId = requisitionId;
        loan.status = ContractStatus.Active;
        loan.parcelsCount = loanRequisitions[requisitionId].parcelsCount;
        loan.parcelsPending = loanRequisitions[requisitionId].parcelsCount;
        loan.parcelsValues = loanRequisitions[requisitionId].amount / loanRequisitions[requisitionId].parcelsCount;
        
        _generatePaymentDates(loan, loanRequisitions[requisitionId].parcelsCount);
        
        emit LoanContractGenerated(
            loan.walletAddress, 
            requisitionId, 
            loan.status, 
            loan.parcelsPending, 
            loan.parcelsValues,
            loan.paymentDates
        );
    }

    function _generatePaymentDates(LoanContract storage loan, uint32 parcelsCount) internal {
        uint256 startDate = block.timestamp;
        uint256 oneMonthInSeconds = 30 days;
        
        for (uint32 i = 0; i < parcelsCount; i++) {
            uint256 paymentDate = startDate + (oneMonthInSeconds * (i + 1));
            loan.paymentDates.push(paymentDate);
        }
    }

    /**
     * @dev Internal function to fund the loan with debt tracking
     */
    function _fundLoan(uint256 requisitionId) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        address borrower = req.borrower;
        
        // Register borrower if not already registered
        if (!isRegisteredBorrower[borrower]) {
            allBorrowers.push(borrower);
            isRegisteredBorrower[borrower] = true;
        }

        borrowings[borrower] += req.amount;
        totalBorrowed += req.amount;
        availableBalance -= req.amount;
        lastBorrowTime[borrower] = block.timestamp;
        req.status = BorrowStatus.Active;

        // Update debt status for the borrower
        debtStorage.updateBorrowerDebtStatus(borrower, this.getActiveLoans);

        // Transfer USDT to borrower
        bool success = IERC20(usdtToken).transfer(borrower, req.amount);
        if (!success) revert TokenTransferFailed();

        emit Borrowed(borrower, req.amount, borrowings[borrower]);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
        emit LoanFunded(requisitionId);
    }

    function _distributeRepaymentToLenders(uint256 requisitionId, uint256 repaymentAmount) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        uint256 totalCoverageAmount = req.amount;

        // Distribute to each covering lender proportionally
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

    // Debt tracker functions
    function triggerMonthlyUpdate() external nonReentrant {
        bool triggered = debtStorage.checkAndTriggerMonthlyUpdate(allBorrowers, this.getActiveLoans);
        require(triggered, "Monthly update not due yet");
    }

    function updateBorrowerDebtStatus(address borrower) external {
        debtStorage.updateBorrowerDebtStatus(borrower, this.getActiveLoans);
    }

    function batchUpdateDebtStatus(address[] calldata borrowers) external {
        for (uint i = 0; i < borrowers.length; i++) {
            debtStorage.updateBorrowerDebtStatus(borrowers[i], this.getActiveLoans);
        }
    }

    function getBorrowersWithDebt() external view returns (address[] memory) {
        return debtStorage.getBorrowersWithDebt();
    }

    function getBorrowersWithoutDebt() external view returns (address[] memory) {
        return debtStorage.getBorrowersWithoutDebt();
    }

    function getBorrowerDebtStatus(address borrower) external view returns (DebtTracker.DebtStatus) {
        return debtStorage.getBorrowerStatus(borrower);
    }

    function hasOverdueDebt(address borrower) external view returns (bool) {
        return debtStorage.hasOverdueDebt(borrower);
    }

    function hasActiveDebt(address borrower) external view returns (bool) {
        return debtStorage.hasActiveDebt(borrower);
    }

    function nextMonthlyUpdate() external view returns (uint256) {
        return debtStorage.nextMonthlyUpdate();
    }

    function shouldTriggerUpdate() external view returns (bool) {
        return debtStorage.shouldTriggerUpdate();
    }

    // Helper function to get active loans for a borrower
    function getActiveLoans(address borrower) public view returns (LoanContract[] memory activeLoans, uint256[] memory requisitionIds) {
        uint256[] storage allRequisitionIds = borrowerRequisitions[borrower];
        uint256 activeCount = 0;
        
        // count active loans
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

    // Reputation system view functions
    function getReputation(uint32 memberId) external view returns (int32) {
        return reputationSystem.getReputation(memberId);
    }

    function getMemberId(address wallet) external view returns (uint32) {
        return reputationSystem.getMemberId(wallet);
    }

    function isWalletVinculated(address wallet) external view returns (bool) {
        return reputationSystem.isWalletVinculated(wallet);
    }

    // Helper function to get USDT balance of contract
    function getUSDTBalance() external view returns (uint256) {
        return IERC20(usdtToken).balanceOf(address(this));
    }

    // Helper function to check USDT allowance for this contract
    function getAllowance(address user) external view returns (uint256) {
        return IERC20(usdtToken).allowance(user, address(this));
    }

    // Function to check next payment amount for a specific requisition
    function getNextPaymentAmount(uint256 requisitionId) external view returns (uint256 paymentAmount, bool canPay) {
        LoanContract storage loan = loanContracts[requisitionId];
        
        if (loan.status == ContractStatus.Active && loan.parcelsPending > 0) {
            return (loan.parcelsValues, true);
        }
        
        return (0, false);
    }

    // Function to get repayment summary for a specific requisition
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

    // Function to check if a requisition can be paid
    function canPayRequisition(uint256 requisitionId, address borrower) external view returns (bool) {
        LoanContract storage loan = loanContracts[requisitionId];
        
        return (
            loan.walletAddress == borrower &&
            loan.status == ContractStatus.Active &&
            loan.parcelsPending > 0
        );
    }

    // View functions from interface
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
            durationDays: req.durationDays,
            creationTime: req.creationTime,
            coveringLenders: req.coveringLenders,
            parcelsCount: req.parcelsCount
        });
    }

    // View function to get loan contract details
    function getLoanContract(uint256 requisitionId) external view returns (LoanContract memory) {
        LoanContract memory loan = loanContracts[requisitionId];
        return loan;
    }

    // Additional helper function (not in interface but useful)
    function canUserBorrow(address _user, uint256 _amount) external view returns (bool) {
        return (_amount > 0 &&
            _amount <= availableBalance &&
            (donations[_user] >= MIN_DONATION_FOR_BORROW || borrowings[_user] == 0) &&
            (lastBorrowTime[_user] + BORROW_DURATION < block.timestamp || borrowings[_user] == 0));
    }
    
    // Helper function to get payment dates
    function getPaymentDates(uint256 requisitionId) external view returns (uint256[] memory) {
        return loanContracts[requisitionId].paymentDates;
    }

    function getDonationsInCoverage(address lender) external view returns (uint256) {
        return donationsInCoverage[lender];
    }
}