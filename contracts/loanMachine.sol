// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "./interfaces/ILoanMachine.sol";
import "./ReputationSystem.sol";

contract LoanMachine is ILoanMachine, ReentrancyGuard {

    // Reputation system integration
    ReputationSystem public reputationSystem;
    
    // Your struct idea, slightly refined
    struct DebtWatchItem {
        uint256 requisitionId; 
        address borrower;      
        uint256 nextDueDate;   
        bool isOverdue;        
    }

    // The array we will check periodically
    DebtWatchItem[] public debtWatchlist;

    // This mapping is the KEY to fast removal. It tracks:
    // requisitionId -> index in the debtWatchlist array
    mapping(uint256 => uint256) private watchlistIndex;

    // --- NEW KEEPER VARIABLES ---
    uint256 public nextCheckIndex;
    uint256 public lastPeriodicCheckTimestamp;
    // You want twice a month, so ~15 days
    uint256 public constant CHECK_INTERVAL = 15 days;

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
    mapping (uint32 => uint32) private loanRequisitionNumber;
    mapping (address => uint256) private lastContractPerWallet;

    // Loan Contracts
    mapping(uint256 => LoanContract) public loanContracts;

    // Constants
    uint32 private constant BORROW_DURATION = 30 days;
    uint32 private constant MIN_DONATION_FOR_BORROW = 1e6; // 1 USDT (6 decimals)
    uint32 private constant MAX_DONATION = 5e6; // 5 USDT (6 decimals)

    // Custom errors
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
        if (minimumCoverage <= 70 || minimumCoverage > 100) revert LoanMachine_InvalidCoveragePercentage();
        _;
    }

    modifier validAmount(uint256 _amount) {
        if (_amount == 0) revert LoanMachine_InvalidAmount();
        _;
    }

    modifier canBorrow(uint256 _amount) {
        if (_amount > availableBalance) revert LoanMachine_InsufficientFunds();
        if (donations[msg.sender] < MIN_DONATION_FOR_BORROW && borrowings[msg.sender] != 0) {
            revert LoanMachine_MinimumDonationRequired();
        }
        if (lastBorrowTime[msg.sender] + BORROW_DURATION >= block.timestamp && borrowings[msg.sender] != 0) {
            revert LoanMachine_BorrowNotExpired();
        }
        _;
    }

    modifier validCoverage(uint32 coveragePercentage) {
        if (coveragePercentage == 0 || coveragePercentage > 100) revert LoanMachine_InvalidCoveragePercentage();
        if(coveragePercentage <= 9) revert LoanMachine_MinimumPercentageCover();
        _;
    }

    modifier maxParcelsCount(uint32 parcelscount) {
        if (parcelscount < 1 || parcelscount > 12) revert LoanMachine_InvalidParcelsCount();
        _;
    }

    modifier validMember(uint32 memberId, address wallet) {
        if(memberId == 0 || !reputationSystem.isWalletVinculated(wallet)) 
            revert LoanMachine_MemberIdOrWalletInvalid();
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

    constructor(address _usdtToken, address _reputationSystem) {
        usdtToken = _usdtToken;
        reputationSystem = ReputationSystem(_reputationSystem);
    }
    
    /**
     * @dev Performs a periodic check for overdue loans on a small batch.
     * Anyone can call this function, ideally a bot ("Keeper").
     * This distributes the gas cost over many blocks.
     * @param batchSize The number of items to check in this run.
     */
    function performPeriodicDebtCheck(uint256 batchSize) external nonReentrant {
        uint256 watchlistLength = debtWatchlist.length;
        if (watchlistLength == 0) { return; } // Nothing to check

        // Check if the full interval has passed since the *last full run*
        bool intervalPassed = block.timestamp > lastPeriodicCheckTimestamp + CHECK_INTERVAL;
        
        // If we are at the start and the interval hasn't passed, revert.
        // This enforces your "twice a month" rule.
        if (nextCheckIndex == 0 && !intervalPassed) {
            revert LoanMachine_CheckIntervalNotYetPassed();
        }

        // Determine how many items to check, max is the batchSize
        uint256 itemsChecked = 0;
        for (uint256 i = 0; i < batchSize && nextCheckIndex < watchlistLength; i++) {
            DebtWatchItem storage item = debtWatchlist[nextCheckIndex];

            // --- THIS IS THE CORE LOGIC ---
            // If it's not already marked as overdue AND the due date has passed
            if (!item.isOverdue && block.timestamp > item.nextDueDate) {
                item.isOverdue = true;
                emit BorrowerOverdue(item.requisitionId, item.borrower, item.nextDueDate);
            }
            // ---

            nextCheckIndex++;
            itemsChecked++;
        }

        // If we've finished checking the whole array
        if (nextCheckIndex >= watchlistLength) {
            nextCheckIndex = 0; // Reset for the next cycle
            lastPeriodicCheckTimestamp = block.timestamp; // Mark the end of this full run
        }
        
        emit PeriodicCheckRun(itemsChecked, nextCheckIndex);
    }

    // --- NEW VIEW FUNCTIONS ---
    function getDebtWatchlist() external view returns (DebtWatchItem[] memory) {
        return debtWatchlist;
    }

    function isBorrowerOverdue(uint256 requisitionId) external view returns (bool) {
        // Check if it's even in the watchlist
        if (debtWatchlist.length == 0 || watchlistIndex[requisitionId] == 0 && debtWatchlist[0].requisitionId != requisitionId) {
            return false;
        }
        return debtWatchlist[watchlistIndex[requisitionId]].isOverdue;
    }

    /**
 * @dev Allows the borrower to cancel their own loan requisition and return covered funds to lenders.
 * Can only be called if the requisition is Pending or PartiallyCovered.
 * @param requisitionId The ID of the loan requisition to cancel.
 * @param memberId The member ID of the borrower (used for reputation system calls).
 * @return totalUncoveredAmount The total amount of funds that were returned to the lenders.
 */
function cancelLoanRequisition(uint256 requisitionId, uint32 memberId) 
    external 
    nonReentrant 
    validMember(memberId, msg.sender)
    returns (uint256 totalUncoveredAmount) 
{
    LoanRequisition storage req = loanRequisitions[requisitionId];
    uint256 initialCoverage = req.currentCoverage; // Store initial value

    // 1. Status Check & Authorization
    if (req.borrower != msg.sender) {
        // Assuming you have a custom error for this, or use a standard revert
        revert LoanMachine_OnlyBorrowerCanCancelRequisition(); 
    }
    
    // Cannot cancel if it's already active (funded/disbursed)
    if (req.status == BorrowStatus.Active || req.status == BorrowStatus.Cancelled) {
        revert LoanMachine_RequisitionNotCancellable();
    }
    
    // 2. Fund Unwind (Return coverage to lenders)
    totalUncoveredAmount = 0;
    
    // Iterate over all lenders who covered this loan
    for (uint256 i = 0; i < req.coveringLenders.length; i++) {
        address lender = req.coveringLenders[i];
        uint256 coverageAmount = req.coverageAmounts[lender];
        
        if (coverageAmount > 0) {
            // Move funds from 'donationsInCoverage' back to 'donations'
            unchecked {
                donationsInCoverage[lender] -= coverageAmount;
                donations[lender] += coverageAmount;
            }
            totalUncoveredAmount += coverageAmount;

            // Clear the coverage amount for this lender on this specific requisition
            delete req.coverageAmounts[lender]; 
            
            // Note: We leave the lender in the `coveringLenders` array for historical reference
            // as Solidity arrays are costly to shrink/shift, but we clear their contribution.
            
            emit LoanUncovered(requisitionId, lender, coverageAmount);
        }
    }

    // 3. State Cleanup
    
    // Decrease the pending loan count for the member (used in checkLoanRequisitionOpened)
    if (initialCoverage < 100) { // Only decrease if it wasn't fully covered
        loanRequisitionNumber[memberId] -= 1;
    }

    // Mark the requisition as cancelled
    req.status = BorrowStatus.Cancelled;
    req.currentCoverage = 0; // Clear coverage percentage
    // Note: The requisition still exists in the mapping (loanRequisitions) for historical lookup

    // Optional: Remove the requisition ID from the borrowerRequisitions array
    // This is optional due to high gas cost, but is generally good practice for cleanup.
    // If you choose to implement this, you'd need an array swapping utility similar to _removeFromWatchlist

    emit LoanRequisitionCancelled(requisitionId, msg.sender, totalUncoveredAmount);
    
    return totalUncoveredAmount;
}
    
    /**
     * @dev Withdraw function - allows users to withdraw only from donations not in cover
     */
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

        // Update state
        donations[msg.sender] -= amount;
        totalDonations -= amount;
        availableBalance -= amount;

        // Transfer USDT to user
        bool success = IERC20(usdtToken).transfer(msg.sender, amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        emit Withdrawn(msg.sender, amount, donations[msg.sender]);
        emit TotalDonationsUpdated(totalDonations);
        emit AvailableBalanceUpdated(availableBalance);
    }

    /**
     * @dev Get the withdrawable balance for a user (donations not in cover)
     */
    function getWithdrawableBalance(address user) public view returns (uint256) {
        // Withdrawable balance is the minimum between available donations and donations not locked in coverage
        uint256 availableDonations = donations[user];
        uint256 lockedInCoverage = donationsInCoverage[user];
        
        // If user has more donations than what's locked in coverage, they can withdraw the difference
        if (availableDonations > lockedInCoverage) {
            return availableDonations - lockedInCoverage;
        }
        return 0;
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
        

        if (amount == 0) revert LoanMachine_InvalidAmount();

        LoanContract storage loan = loanContracts[requisitionId];
        address borrower = msg.sender;
        
        uint32 currentParcelIndex = loan.parcelsCount - loan.parcelsPending;
        uint256 currentParcelDueDate = loan.paymentDates[currentParcelIndex];
        
        uint256 itemIndex = watchlistIndex[requisitionId];
        DebtWatchItem storage item = debtWatchlist[itemIndex];

        // Update reputation AND debt status based on payment timing
        if (block.timestamp > currentParcelDueDate) {
            reputationSystem.reputationChange(memberId, reputationSystem.REPUTATION_LOSS_BY_DEBT_NOT_PAYD(), false);
            
            // They were overdue *before* this payment. Set the flag.
            if (!item.isOverdue) {
                item.isOverdue = true;
                emit BorrowerOverdue(requisitionId, borrower, currentParcelDueDate); 
            }
        } else {
            reputationSystem.reputationChange(memberId, reputationSystem.REPUTATION_GAIN_BY_REPAYNG_DEBT(), true);
            
            // They paid on time, so they are definitely not overdue
            if (item.isOverdue) {
                item.isOverdue = false;
                emit BorrowerDebtSettled(requisitionId, borrower); 
            }
        }

        if (amount != loan.parcelsValues) revert LoanMachine_InvalidAmount();
        
        // Transfer USDT from borrower to contract
        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

        // Update global borrowing state
        borrowings[borrower] -= amount;
        totalBorrowed -= amount;
        availableBalance += amount;

        _distributeRepaymentToLenders(requisitionId, amount);
        
        loan.parcelsPending -= 1;
        if (loan.parcelsPending == 0) {
            loan.status = ContractStatus.Closed;
            emit LoanCompleted(requisitionId);
            
            // Loan is done. Remove it from the watchlist.
            _removeFromWatchlist(requisitionId);

        } else {
            // Loan is still active. Update the watchlist item.
            uint32 nextParcelIndex = loan.parcelsCount - loan.parcelsPending;
            item.nextDueDate = loan.paymentDates[nextParcelIndex];
            // They just paid, so reset their status for the next parcel
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

        // Don't do anything if it's already the last item
        if (indexToRemove != lastIndex) {
            // Move the last item into the spot we are removing
            DebtWatchItem storage lastItem = debtWatchlist[lastIndex];
            debtWatchlist[indexToRemove] = lastItem;
            // Update the index mapping for the item we just moved
            watchlistIndex[lastItem.requisitionId] = indexToRemove;
        }

        // Delete the (now duplicate) last item
        debtWatchlist.pop();
        delete watchlistIndex[requisitionId];
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

        // Transfer USDT from user to contract
        bool success = IERC20(usdtToken).transferFrom(msg.sender, address(this), amount);
        if (!success) revert LoanMachine_TokenTransferFailed();

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
        uint256 amount,
        uint32 minimumCoverage,
        uint32 parcelscount,
        uint32 memberId
    ) 
        external 
        checkLoanRequisitionOpened(memberId)
        validMember(memberId, msg.sender) 
        validAmount(amount) 
        minimumPercentCoveragePermited(minimumCoverage) 
        maxParcelsCount(parcelscount) 
        returns (uint256) 
    {
        if (amount > availableBalance) revert LoanMachine_InsufficientFunds();
        
        uint256 lastContractId = lastContractPerWallet[msg.sender];

        if (lastContractId != 0) {
            uint256 lastCreationTimeOfLastContract = loanContracts[lastContractId].creationTime;
            if (lastCreationTimeOfLastContract + BORROW_DURATION > block.timestamp) {
                revert LoanMachine_BorrowNotExpired();
            }
        }
        uint256 requisitionId = requisitionCounter++;
        
        LoanRequisition storage newReq = loanRequisitions[requisitionId];
        newReq.borrower = msg.sender;
        newReq.amount = amount;
        newReq.minimumCoverage = minimumCoverage;
        newReq.currentCoverage = 0;
        newReq.status = BorrowStatus.Pending;
        newReq.creationTime = block.timestamp;
        newReq.parcelsCount = parcelscount;
        newReq.requisitionId = requisitionId;
        borrowerRequisitions[msg.sender].push(requisitionId);
        
        loanRequisitionNumber[memberId] += 1;
        emit LoanRequisitionCreated(requisitionId, msg.sender, amount, parcelscount);
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
            revert LoanMachine_LoanNotAvailable();
        }
        
        if (uint256(req.currentCoverage) + uint256(coveragePercentage) > 100) {
            revert LoanMachine_OverCoverage();
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

            uint32 borrowerId = reputationSystem.walletToMemberId(req.borrower);
            loanRequisitionNumber[borrowerId] = 0;
            _generateLoanContract(requisitionId);
            _fundLoan(requisitionId);
        } else {
            req.status = BorrowStatus.PartiallyCovered;
        }


        int32 reputationGain = (reputationSystem.REPUTATION_GAIN_BY_COVERING_LOAN() * int32(coveragePercentage))/10;
        reputationSystem.reputationChange(memberId, int32(reputationGain), true);
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

    // Calculate base parcel and remainder
    uint256 base = totalAmount / count;
    uint256 remainder = totalAmount % count;

    // Generate parcel values with perfect distribution
    uint256[] memory parcels = new uint256[](count);
    for (uint256 i = 0; i < count; i++) {
        if (i < remainder) {
            parcels[i] = base + 1;
        } else {
            parcels[i] = base;
        }
    }

    // Optional: store only one base value for simplicity
    // but we’ll emit the full array for clarity
    loan.parcelsValues = base;

    _generatePaymentDates(loan, count);

    address lastContractAddress = loan.walletAddress;
    lastContractPerWallet[lastContractAddress] = requisitionId;


    emit LoanContractGenerated(
        loan.walletAddress,
        requisitionId,
        loan.status,
        loan.parcelsPending,
        loan.parcelsValues, 
        loan.paymentDates,
        loan.creationTime
    );

    // If you need to store all parcel values (recommended for repayments tracking)
    for (uint256 i = 0; i < count; i++) {
        loan.parcelsAmounts.push(parcels[i]);
    }
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


        borrowings[borrower] += req.amount;
        totalBorrowed += req.amount;
        availableBalance -= req.amount;
        lastBorrowTime[borrower] = block.timestamp;
        req.status = BorrowStatus.Active;

        
        // Transfer USDT to borrower
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