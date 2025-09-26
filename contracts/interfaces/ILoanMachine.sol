// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface ILoanMachine {
    enum BorrowStatus { Pending, PartiallyCovered, FullyCovered, Active, Repaid, Defaulted }
    
    struct RequisitionInfo {
        address borrower;
        uint256 amount;
        uint32 minimumCoverage;
        uint32 currentCoverage;
        BorrowStatus status;
        uint256 durationDays;
        uint256 creationTime;
        uint256 coveringLendersCount;
    }

    // Events
    event Donated(address indexed donor, uint256 amount, uint256 totalDonation);
    event Borrowed(address indexed borrower, uint256 amount, uint256 totalBorrowing);
    event Repaid(address indexed borrower, uint256 amount, uint256 remainingDebt);

    event TotalDonationsUpdated(uint256 total);
    event TotalBorrowedUpdated(uint256 total);
    event AvailableBalanceUpdated(uint256 total);

    event NewDonor(address indexed donor); 
    event NewBorrower(address indexed borrower); 
    event BorrowLimitReached(address indexed borrower);

    event LoanRequisitionCreated(uint256 indexed requisitionId, address indexed borrower, uint256 amount);
    event LoanCovered(uint256 indexed requisitionId, address indexed lender, uint256 coverageAmount);
    event LoanFunded(uint256 indexed requisitionId);

    // Core functions
    function donate() external payable;

    function createLoanRequisition(uint256 _amount, uint32 _minimumCoverage, uint256 _durationDays) external returns (uint256);
    function coverLoan(uint256 requisitionId, uint32 coveragePercentage) external;

    function borrow(uint256 _amount) external;
    function repay() external payable;

    // View functions
    function getTotalDonations() external view returns (uint256);
    function getTotalBorrowed() external view returns (uint256);
    function getAvailableBalance() external view returns (uint256);
    
    function getRequisitionInfo(uint256 requisitionId) external view returns (RequisitionInfo memory);
    function getBorrowerRequisitions(address borrower) external view returns (uint256[] memory);
    
    function getCoveringLenders(uint256 requisitionId) external view returns (address[] memory);
    function getLenderCoverage(uint256 requisitionId, address lender) external view returns (uint256);
    
    function getAvailableBorrowAmount() external view returns (uint256);
}