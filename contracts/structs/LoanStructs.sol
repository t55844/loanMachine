// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "../interfaces/ILoanMachine.sol";

struct LoanRequisitionData {
    address borrower;
    uint256 amount;
    uint32 minimumCoverage;
    uint32 currentCoverage;
    ILoanMachine.BorrowStatus status;
    uint256 durationDays;
    uint256 creationTime;
    address[] coveringLenders;
}

library LoanStructs {
    // This library can contain functions to manipulate structs if needed
}
