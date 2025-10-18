// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../libraries/DebtTracker.sol";

interface ILoanMachine {
    enum BorrowStatus { Pending, PartiallyCovered, FullyCovered, Active, Repaid, Defaulted }
    
    struct RequisitionInfo {
        uint256 requisitionId;
        address borrower;
        uint256 amount;
        uint32 minimumCoverage;
        uint32 currentCoverage;
        BorrowStatus status;
        uint256 durationDays;
        uint256 creationTime;
        address[] coveringLenders;
        uint32 parcelsCount;
    }
    
    enum ContractStatus { Active, Pending, Closed }
    
    struct LoanContract {
        address walletAddress;
        uint256 requisitionId;
        ContractStatus status;
        uint32 parcelsCount;
        uint32 parcelsPending;
        uint256 parcelsValues;
        uint256[] paymentDates;
    }


    // Events
    event Withdrawn(address indexed donor,uint256 amount,uint256 donations);
    event Donated(address indexed donor, uint256 amount, uint256 totalDonation);
    event Borrowed(address indexed borrower, uint256 amount, uint256 totalBorrowing);
    event Repaid(address indexed borrower, uint256 amount, uint256 remainingDebt);
    event TotalDonationsUpdated(uint256 total);
    event TotalBorrowedUpdated(uint256 total);
    event AvailableBalanceUpdated(uint256 total);
    event NewDonor(address indexed donor);
    event NewBorrower(address indexed borrower);
    event BorrowLimitReached(address indexed borrower);
    event LoanRequisitionCreated(uint256 indexed requisitionId, address indexed borrower, uint256 amount, uint32 parcelsCount);
    event LoanCovered(uint256 indexed requisitionId, address indexed lender, uint256 coverageAmount);
    event LoanFunded(uint256 indexed requisitionId);
    event LoanContractGenerated(address indexed walletAddress, uint256 indexed requisitionId, ContractStatus status, uint32 parcelsPending, uint256 parcelsValues, uint256[] paymentDates);
    event ParcelPaid(uint256 indexed requisitionId, uint256 parcelsRemaining);
    event LenderRepaid(uint256 indexed requisitionId, address indexed lender, uint256 amount);
    event LoanCompleted(uint256 indexed requisitionId);

    // declare events from DebtTracker to include in ABI
    event BorrowerStatusUpdated(address indexed borrower, DebtTracker.DebtStatus newStatus);
    event DebtorAdded(address indexed borrower);
    event DebtorRemoved(address indexed borrower);
    event MonthlyUpdateTriggered(uint256 timestamp);


    // Core functions
    function vinculationMemberToWallet(uint32 memberId, address wallet) external;
    function donate(uint256 amount, uint32 memberId) external;
    function createLoanRequisition(uint256 _amount, uint32 _minimumCoverage, uint256 _durationDays, uint32 _parcelsCount, uint32 memberId) external returns (uint256);
    function coverLoan(uint256 requisitionId, uint32 coveragePercentage, uint32 memberId) external;
    function repay(uint256 requisitionId, uint256 amount, uint32 memberId) external;
    
    // View functions
    function getTotalDonations() external view returns (uint256);
    function getTotalBorrowed() external view returns (uint256);
    function getAvailableBalance() external view returns (uint256);
    function canUserBorrow(address _user, uint256 _amount) external view returns (bool);
    function getRequisitionInfo(uint256 requisitionId) external view returns (RequisitionInfo memory);
    function getBorrowerRequisitions(address borrower) external view returns (uint256[] memory);
    function getCoveringLenders(uint256 requisitionId) external view returns (address[] memory);
    function getLenderCoverage(uint256 requisitionId, address lender) external view returns (uint256);
    function getAvailableBorrowAmount() external view returns (uint256);
    function getLoanContract(uint256 requisitionId) external view returns (LoanContract memory);
    function getActiveLoans(address borrower) external view returns (LoanContract[] memory activeLoans, uint256[] memory requisitionIds);
    function getNextPaymentAmount(uint256 requisitionId) external view returns (uint256 paymentAmount, bool canPay);
    function getRepaymentSummary(uint256 requisitionId) external view returns (
        uint256 totalRemainingDebt,
        uint256 nextPaymentAmount,
        uint256 parcelsRemaining,
        uint256 totalParcels,
        bool isActive
    );
    function canPayRequisition(uint256 requisitionId, address borrower) external view returns (bool);
    function getPaymentDates(uint256 requisitionId) external view returns (uint256[] memory);
    function getDonationsInCoverage(address lender) external view returns (uint256);
    function getUSDTBalance() external view returns (uint256);
    function getAllowance(address user) external view returns (uint256);
    
    // Reputation system view functions (new additions)
    function getReputation(uint32 memberId) external view returns (int32);
    function getMemberId(address wallet) external view returns (uint32);
    function isWalletVinculated(address wallet) external view returns (bool);
    
    // State variable getters
    function usdtToken() external view returns (address);
    function requisitionCounter() external view returns (uint256);
    function getDonation(address _user) external view returns (uint256);
    function getBorrowing(address _user) external view returns (uint256);
    function getLastBorrowTime(address _user) external view returns (uint256);
    function getContractBalance() external view returns (uint256);
}