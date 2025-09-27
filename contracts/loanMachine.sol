// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "./interfaces/ILoanMachine.sol";

contract LoanMachine is ILoanMachine, ReentrancyGuard {
    // State variables
    mapping(address => uint256) private donations;
    mapping(address => uint256) private borrowings;
    mapping(address => uint256) private lastBorrowTime;

    uint256 private totalDonations;
    uint256 private totalBorrowed;
    uint256 private availableBalance;

    // Loan requisition system
    mapping(uint256 => LoanRequisition) private loanRequisitions;
    mapping(address => uint256[]) public borrowerRequisitions;
    mapping(address => uint256) private donationsInCoverage;
    uint256 public requisitionCounter;

    //Loan Contracts
    mapping(uint256 => LoanContract) public loanContracts;

    // Constants
    uint256 private constant BORROW_DURATION = 7 days;
    uint256 private constant MIN_DONATION_FOR_BORROW = 0.1 ether;

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

    // Donate function
    function donate() external payable validAmount(msg.value) nonReentrant {
        bool isNewDonor = donations[msg.sender] == 0;

        unchecked {
            donations[msg.sender] += msg.value;
            totalDonations += msg.value;
            availableBalance += msg.value;
        }

        emit Donated(msg.sender, msg.value, donations[msg.sender]);
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
        uint32 parcelscount
    ) external validAmount(_amount) minimumPercentCoveragePermited(_minimumCoverage) returns (uint256) {
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
    function coverLoan(uint256 requisitionId, uint32 coveragePercentage) 
        external 
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
        
        uint256 coverageAmount = (req.amount * uint256(coveragePercentage)) / 100;
        
        if (donations[msg.sender] < coverageAmount) {
            revert InsufficientDonationBalance();
        }
        
        unchecked {
            donations[msg.sender] -= coverageAmount;
            donationsInCoverage[msg.sender] += coverageAmount;
        }
        
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
        
        emit LoanCovered(requisitionId, msg.sender, coverageAmount);
    }

    function _generateLoanContract(uint256 requisitionId) internal {
    LoanContract storage loan = loanContracts[requisitionId];
    loan.walletAddress = loanRequisitions[requisitionId].borrower;
    loan.requisitionId = requisitionId;
    loan.status = ContractStatus.Active;
    loan.parcelsPending = loanRequisitions[requisitionId].parcelsCount;

    emit LoanContractGenerated(loan.walletAddress, requisitionId, loan.status, loan.parcelsPending);
}

    // Internal function to fund the loan
    function _fundLoan(uint256 requisitionId) internal {
        LoanRequisition storage req = loanRequisitions[requisitionId];
        
        unchecked {
            borrowings[req.borrower] += req.amount;
            totalBorrowed += req.amount;
            availableBalance -= req.amount;
        }
        
        lastBorrowTime[req.borrower] = block.timestamp;
        req.status = BorrowStatus.Active;
        
        payable(req.borrower).transfer(req.amount);
        
        emit Borrowed(req.borrower, req.amount, borrowings[req.borrower]);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
        emit LoanFunded(requisitionId);
    }

    // Direct borrow function
    function borrow(uint256 _amount) external validAmount(_amount) canBorrow(_amount) nonReentrant {
        bool isNewBorrower = borrowings[msg.sender] == 0;

        unchecked {
            borrowings[msg.sender] += _amount;
            totalBorrowed += _amount;
            availableBalance -= _amount;
        }
        
        lastBorrowTime[msg.sender] = block.timestamp;

        emit Borrowed(msg.sender, _amount, borrowings[msg.sender]);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
        
        if (isNewBorrower) {
            emit NewBorrower(msg.sender);
        }

        payable(msg.sender).transfer(_amount);
    }

    // Repay function
    function repay() external payable validAmount(msg.value) nonReentrant {
        if (borrowings[msg.sender] == 0) revert NoActiveBorrowing();
        if (msg.value > borrowings[msg.sender]) revert RepaymentExceedsBorrowed();

        unchecked {
            borrowings[msg.sender] -= msg.value;
            totalBorrowed -= msg.value;
            availableBalance += msg.value;
        }

        emit Repaid(msg.sender, msg.value, borrowings[msg.sender]);
        emit TotalBorrowedUpdated(totalBorrowed);
        emit AvailableBalanceUpdated(availableBalance);
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
        return address(this).balance;
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
}