// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface ILoanMachine {
    enum BorrowStatus { Pending, PartiallyCovered, FullyCovered, Active, Repaid, Defaulted, Cancelled }

    struct RequisitionInfo {
        uint256      requisitionId;
        address      borrower;
        uint256      amount;
        uint32       minimumCoverage;
        uint32       currentCoverage;
        BorrowStatus status;
        uint256      creationTime;
        address[]    coveringLenders;
        uint32       parcelsCount;
    }

    enum ContractStatus { Active, Pending, Closed }

    struct LoanContract {
        address       walletAddress;
        uint256       requisitionId;
        ContractStatus status;
        uint32        parcelsCount;
        uint32        parcelsPending;
        uint256       parcelsValues;
        uint256[]     paymentDates;
        uint256[]     parcelsAmounts;
        uint256       creationTime;
    }

    struct WithdrawalRequest {
        address requester;
        uint256 amount;
        uint256 requestedAt;
        uint256 executableAfter;
        bool    executed;
        bool    blocked;
    }

    struct DebtWatchItem {
        uint256 requisitionId;
        address borrower;
        uint256 nextDueDate;
        bool    isOverdue;
    }

    // ── Events ───────────────────────────────────────────────
    event Withdrawn(address indexed donor, uint256 amount, uint256 donations);
    event Donated(address indexed donor, uint256 amount, uint256 totalDonation);
    event Borrowed(address indexed borrower, uint256 amount, uint256 totalBorrowing);
    event Repaid(address indexed borrower, uint256 amount, uint256 remainingDebt);
    event TotalDonationsUpdated(uint256 total);
    event TotalBorrowedUpdated(uint256 total);
    event AvailableBalanceUpdated(uint256 total);
    event NewDonor(address indexed donor);
    event LoanRequisitionCreatedCancelled(uint256 indexed requisitionId, address indexed borrower, uint256 amount, uint32 parcelsCount, BorrowStatus status);
    event LoanCovered(uint256 indexed requisitionId, address indexed lender, uint256 coverageAmount);
    event LoanFunded(uint256 indexed requisitionId);
    event LoanContractGenerated(address indexed walletAddress, uint256 indexed requisitionId, ContractStatus status, uint32 parcelsPending, uint256 parcelsValues, uint256[] paymentDates, uint256 creationTime);
    event ParcelPaid(uint256 indexed requisitionId, uint256 parcelsRemaining);
    event LenderRepaid(uint256 indexed requisitionId, address indexed lender, uint256 amount);
    event LoanCompleted(uint256 indexed requisitionId);
    event LoanUncovered(uint256 indexed requisitionId, address indexed lender, uint256 amountReturnedToLender);

    // Overdue events — still emitted from repay() for subgraph indexing
    // but no on-chain watchlist tracking anymore
    event BorrowerOverdue(uint256 indexed requisitionId, address indexed borrower, uint256 dueDate);
    event BorrowerDebtSettled(uint256 indexed requisitionId, address indexed borrower);

    // Withdrawal events
    event WithdrawalRequested(uint256 indexed requestId, address indexed requester, uint256 amount, uint256 executableAfter);
    event WithdrawalExecuted(uint256 indexed requestId, address indexed requester, uint256 amount);
    event WithdrawalBlocked(uint256 indexed requestId, address indexed blocker);
    event WithdrawalCancelled(uint256 indexed requestId, address indexed requester);

    // Core functions
    function donate(uint256 amount, bytes32 memberId) external;
    function createLoanRequisition(uint256 _amount, uint32 _minimumCoverage, uint32 _parcelsCount, bytes32 memberId, uint32 daysIntervalOfPayment) external returns (uint256);
    function coverLoan(uint256 requisitionId, uint32 coveragePercentage, bytes32 memberId) external;
    function repay(uint256 requisitionId, uint256 amount, bytes32 memberId) external;

    function requestWithdrawal(uint256 amount, bytes32 memberId) external returns (uint256 requestId);
    function executeWithdrawal(uint256 requestId, bytes32 memberId) external;
    function cancelWithdrawal(uint256 requestId) external;

    // View functions — consolidated to reduce bytecode
    function getCoopStats() external view returns (
        uint256 totalDonations, uint256 totalBorrowed, uint256 availableBalance,
        uint256 contractBalance, bool isActive, uint32 activeMemberCount, int32 averageReputation
    );
    function getUserFinancials(address user) external view returns (
        uint256 donation, uint256 borrowing, uint256 lastBorrowTime,
        uint256 inCoverage, uint256 withdrawable, uint256 allowance
    );

    function getRequisitionInfo(uint256 requisitionId) external view returns (RequisitionInfo memory);
    function getLoanContract(uint256 requisitionId) external view returns (LoanContract memory);
    function getNextPaymentAmount(uint256 requisitionId) external view returns (uint256 paymentAmount, bool canPay);
    function canPayRequisition(uint256 requisitionId, address borrower) external view returns (bool);
    function canUserBorrow(address user, uint256 amount) external view returns (bool);
    function getPaymentDates(uint256 requisitionId) external view returns (uint256[] memory);
    function getRepaymentSummary(uint256 requisitionId) external view returns (
        uint256 totalRemainingDebt, uint256 nextPaymentAmount,
        uint256 parcelsRemaining, uint256 totalParcels, bool isActive
    );
    function getActiveLoans(address borrower) external view returns (LoanContract[] memory activeLoans, uint256[] memory requisitionIds);
    function getDebtWatchlist() external view returns (DebtWatchItem[] memory);

    function usdtToken() external view returns (address);
    function requisitionCounter() external view returns (uint256);
}
