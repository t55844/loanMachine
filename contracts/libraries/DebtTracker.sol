// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "../interfaces/ILoanMachine.sol";

library DebtTracker {
    // Debt status enum
    enum DebtStatus { 
        NoDebt, 
        HasActiveDebt, 
        HasOverdueDebt 
    }

    // Storage structure for debt tracking
    struct DebtStorage {
        mapping(address => DebtStatus) borrowerDebtStatus;
        address[] borrowersWithOpenDebt;
        address[] borrowersWithoutOpenDebt;
        mapping(address => bool) walletHasOpenDebtNotRepaid;
        mapping(uint32 => bool) memberHasOpenDebtNotRepaid;
        uint256 lastMonthlyUpdate;
        uint256 updateInterval;
    }

    // Events
    event BorrowerStatusUpdated(address indexed borrower, DebtStatus newStatus);
    event DebtorAdded(address indexed borrower);
    event DebtorRemoved(address indexed borrower);
    event MonthlyUpdateTriggered(uint256 timestamp);

    // Constants
    uint256 public constant MONTHLY_INTERVAL = 30 days;

    /**
     * @dev Initialize debt tracker storage
     */
    function initialize(DebtStorage storage ds) internal {
        ds.updateInterval = MONTHLY_INTERVAL;
        ds.lastMonthlyUpdate = block.timestamp;
    }

    /**
     * @dev Updates debt status for a borrower
     */
    function updateBorrowerDebtStatus(
        DebtStorage storage ds,
        address borrower,
        function(address) external view returns (ILoanMachine.LoanContract[] memory, uint256[] memory) getActiveLoans
    ) public {
        DebtStatus previousStatus = ds.borrowerDebtStatus[borrower];
        DebtStatus newStatus = _calculateDebtStatus(borrower, getActiveLoans);
        
        if (previousStatus != newStatus) {
            ds.borrowerDebtStatus[borrower] = newStatus;
            _updateBorrowerLists(ds, borrower, previousStatus, newStatus);
            emit BorrowerStatusUpdated(borrower, newStatus);
        }
    }

    /**
     * @dev Check if monthly update should be triggered and execute if needed
     */
    function checkAndTriggerMonthlyUpdate(
        DebtStorage storage ds,
        address[] memory allBorrowers,
        function(address) external view returns (ILoanMachine.LoanContract[] memory, uint256[] memory) getActiveLoans
    ) public returns (bool triggered) {
        if (block.timestamp >= ds.lastMonthlyUpdate + ds.updateInterval) {
            _executeMonthlyUpdate(ds, allBorrowers, getActiveLoans);
            ds.lastMonthlyUpdate = block.timestamp;
            emit MonthlyUpdateTriggered(block.timestamp);
            return true;
        }
        return false;
    }

    /**
     * @dev Execute monthly update for all borrowers
     */
    function _executeMonthlyUpdate(
        DebtStorage storage ds,
        address[] memory allBorrowers,
        function(address) external view returns (ILoanMachine.LoanContract[] memory, uint256[] memory) getActiveLoans
    ) internal {
        for (uint i = 0; i < allBorrowers.length; i++) {
            updateBorrowerDebtStatus(ds, allBorrowers[i], getActiveLoans);
        }
    }

    /**
     * @dev Calculate debt status for a borrower
     */
    function _calculateDebtStatus(
        address borrower,
        function(address) external view returns (ILoanMachine.LoanContract[] memory, uint256[] memory) getActiveLoans
    ) internal view returns (DebtStatus) {
        (ILoanMachine.LoanContract[] memory activeLoans, ) = getActiveLoans(borrower);
        
        if (activeLoans.length == 0) {
            return DebtStatus.NoDebt;
        }
        
        // Check if any loan has overdue parcels
        for (uint i = 0; i < activeLoans.length; i++) {
            ILoanMachine.LoanContract memory loan = activeLoans[i];
            
            if (loan.parcelsPending > 0) {
                uint32 currentParcelIndex = loan.parcelsCount - loan.parcelsPending;
                uint256 nextPaymentDueDate = loan.paymentDates[currentParcelIndex];
                
                if (block.timestamp > nextPaymentDueDate) {
                    return DebtStatus.HasOverdueDebt;
                }
            }
        }
        
        return DebtStatus.HasActiveDebt;
    }

    /**
     * @dev Update borrower lists based on status change
     */
    function _updateBorrowerLists(
        DebtStorage storage ds,
        address borrower,
        DebtStatus oldStatus,
        DebtStatus newStatus
    ) internal {
        // Remove from previous list
        if (oldStatus == DebtStatus.NoDebt) {
            _removeFromArray(ds.borrowersWithoutOpenDebt, borrower);
        } else if (oldStatus != DebtStatus.NoDebt) {
            _removeFromArray(ds.borrowersWithOpenDebt, borrower);
            ds.walletHasOpenDebtNotRepaid[borrower] = false;
            emit DebtorRemoved(borrower);
        }
        
        // Add to new list
        if (newStatus == DebtStatus.NoDebt) {
            ds.borrowersWithoutOpenDebt.push(borrower);
        } else {
            ds.borrowersWithOpenDebt.push(borrower);
            ds.walletHasOpenDebtNotRepaid[borrower] = true;
            emit DebtorAdded(borrower);
        }
    }

    /**
     * @dev Remove address from array
     */
    function _removeFromArray(address[] storage array, address element) internal {
        for (uint i = 0; i < array.length; i++) {
            if (array[i] == element) {
                array[i] = array[array.length - 1];
                array.pop();
                break;
            }
        }
    }

    // View functions
    function getBorrowerStatus(DebtStorage storage ds, address borrower) public view returns (DebtStatus) {
        return ds.borrowerDebtStatus[borrower];
    }

    function hasOverdueDebt(DebtStorage storage ds, address borrower) public view returns (bool) {
        return ds.borrowerDebtStatus[borrower] == DebtStatus.HasOverdueDebt;
    }

    function hasActiveDebt(DebtStorage storage ds, address borrower) public view returns (bool) {
        DebtStatus status = ds.borrowerDebtStatus[borrower];
        return status == DebtStatus.HasActiveDebt || status == DebtStatus.HasOverdueDebt;
    }

    function getBorrowersWithDebt(DebtStorage storage ds) public view returns (address[] memory) {
        return ds.borrowersWithOpenDebt;
    }

    function getBorrowersWithoutDebt(DebtStorage storage ds) public view returns (address[] memory) {
        return ds.borrowersWithoutOpenDebt;
    }

    function nextMonthlyUpdate(DebtStorage storage ds) public view returns (uint256) {
        return ds.lastMonthlyUpdate + ds.updateInterval;
    }

    function shouldTriggerUpdate(DebtStorage storage ds) public view returns (bool) {
        return block.timestamp >= ds.lastMonthlyUpdate + ds.updateInterval;
    }
}