// src/services/blockchain/abis.rs
//
// All sol! macro definitions live here.
// When you add a new contract, add its ABI here and import
// the generated types in the file that uses them.
//
// Rule: one sol! block per contract. No business logic here.

use alloy::sol;

// ── LOAN MACHINE ─────────────────────────────────────────────
// The cooperative savings/lending contract.
// One instance per cooperative, deployed by LoanMachineFactory.

sol! {
    #[sol(rpc)]
    contract LoanMachine {
        // Member vinculation
        function joinCoop(
            uint32 memberId,
            address wallet,
            string calldata accessCode
        ) external;

        // Donations
        function donate(uint256 amount, uint32 memberId) external;

        // Loan lifecycle
        function createLoanRequisition(
            uint256 amount,
            uint32  minimumCoverage,
            uint32  parcelscount,
            uint32  memberId,
            uint32  daysIntervalOfPayment
        ) external returns (uint256);

        function coverLoan(
            uint256 requisitionId,
            uint32  coveragePercentage,
            uint32  memberId
        ) external;

        function repay(
            uint256 requisitionId,
            uint256 amount,
            uint32  memberId
        ) external;

        // Admin (called by coopAdmin wallet)
        function initializeAdmin(
            address coopAdmin,
            string calldata accessCode
        ) external;

        function approveWallet(address wallet) external;
        function approveWalletBatch(address[] calldata wallets) external;
        function revokeWallet(address wallet) external;
        function transferAdmin(address newAdmin) external;
        function rotateAccessCode(string calldata newCode) external;
        function deactivateCoop() external;

        // Views
        function isWalletApproved(address wallet) external view returns (bool);
        function isMember(address wallet)         external view returns (bool);
        function isCoopActive()                   external view returns (bool);
        function coopAdmin()                      external view returns (address);
        function getReputation(uint32 memberId)   external view returns (int32);
        function getMemberId(address wallet)      external view returns (uint32);
        function isWalletVinculated(address wallet) external view returns (bool);
        function getDonation(address user)        external view returns (uint256);
        function getAvailableBalance()            external view returns (uint256);
        function getTotalDonations()              external view returns (uint256);
    }
}

// ── LOAN MACHINE FACTORY ──────────────────────────────────────
// Deploys new LoanMachine instances (one per cooperative).
// After deployment it has ZERO authority over the instance.

sol! {
    #[sol(rpc)]
    contract LoanMachineFactory {
        function deployCoop(
            string  calldata name,
            address          usdtToken,
            address          coopAdmin,
            string  calldata accessCode
        ) external returns (bytes32 coopId, address loanMachine);

        function getCoopInstance(bytes32 coopId) external view returns (address);
        function getAllCoops()                   external view returns (bytes32[] memory);

        function coops(bytes32 coopId) external view returns (
            address loanMachine,
            string  name,
            uint256 deployedAt,
            bool    exists
        );
    }
}

// ── COOP ACCOUNT ─────────────────────────────────────────────
// Smart wallet for each member (ERC-4337 compatible).
// Deployed by CoopAccountFactory, one per member.

sol! {
    #[sol(rpc)]
    contract CoopAccount {
        function initialize(
            address          owner,
            address          loanMachine,
            uint32           memberId,
            address[] calldata guardians
        ) external;

        function execute(address target, bytes calldata data) external;

        function approveRecovery(address proposedOwner) external;

        function getGuardians()                       external view returns (address[] memory);
        function getRecoveryApprovalCount(address p)  external view returns (uint256);
        function hasGuardianApproved(address g, address p) external view returns (bool);

        function owner()       external view returns (address);
        function loanMachine() external view returns (address);
        function memberId()    external view returns (uint32);
    }
}

// ── COOP ACCOUNT FACTORY ─────────────────────────────────────
// Creates CoopAccount instances for members.

sol! {
    #[sol(rpc)]
    contract CoopAccountFactory {
        function createAccount(
            address          owner,
            address          loanMachine,
            uint32           memberId,
            address[] calldata guardians
        ) external returns (address account);

        function getAccount(address owner) external view returns (address);
    }
}