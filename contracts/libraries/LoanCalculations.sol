// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

library LoanCalculations {
    error CoverageExceeds100();
    error InvalidCoveragePercentage();

    function calculateCoverageAmount(
        uint256 loanAmount,
        uint32 coveragePercentage
    ) internal pure returns (uint256) {
        if (coveragePercentage > 100) {
            revert InvalidCoveragePercentage();
        }
        return (loanAmount * uint256(coveragePercentage)) / 100;
    }

    function calculateNewCoverage(
        uint32 currentCoverage,
        uint32 additionalCoverage
    ) internal pure returns (uint32) {
        uint256 newCoverage = uint256(currentCoverage) +
            uint256(additionalCoverage);
        if (newCoverage > 100) {
            revert CoverageExceeds100();
        }
        return uint32(newCoverage);
    }

    function canUserBorrow(
        uint256 userDonation,
        uint256 userBorrowing,
        uint256 lastBorrowTime,
        uint256 availableBalance,
        uint256 minDonation,
        uint256 borrowDuration
    ) internal view returns (bool) {
        return (availableBalance >= minDonation &&
            (userDonation >= minDonation || userBorrowing == 0) &&
            (lastBorrowTime + borrowDuration < block.timestamp ||
                userBorrowing == 0) &&
            userBorrowing == 0);
    }
}
