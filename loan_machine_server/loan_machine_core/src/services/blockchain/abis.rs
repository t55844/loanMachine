// src/services/blockchain/abis.rs
//
// All sol! macro definitions live here.
// When you add a new contract, add its ABI here and import
// the generated types in the file that uses them.
//
// Rule: one sol! block per contract. No business logic here.
//
// v3 CHANGES:
// - memberId: uint32 → bytes32 (keccak256(COOP_SALT ++ wallet_bytes))
// - Single admin → multisig (proposeAction/confirmProposal)
// - Wallet approval: admin + moderator co-signature
// - Withdrawal: immediate, single-step withdraw()
// - Views: consolidated into getCoopStats() and getUserFinancials()
// - LoanMachineFactory → CoopRegistry (lightweight)

use alloy::sol;

// ── LOAN MACHINE ─────────────────────────────────────────────
// The cooperative savings/lending contract.
// One instance per cooperative, deployed directly by server.
// ReputationLib is inlined — all events emit from this address.

sol! {
    #[sol(rpc)]
    contract LoanMachine {

        // ── INITIALIZATION (called once after deploy) ────────
        function initializeMultisig(
            address[] calldata admins,
            uint256   threshold,
            bytes32   founderMemberId
        ) external;

        // ── MULTISIG ADMIN ───────────────────────────────────
        // ProposalType enum: 0=TransferAdmin, 1=AddAdmin, 2=ApproveWallet,
        // 3=RemoveAdmin, 4=RevokeWallet, 5=Deactivate,
        // 6=Reactivate, 7=SetAuthorizedCaller, 8=ChangeThreshold

        function proposeAction(
            uint8 pType,
            bytes calldata data
        ) external returns (uint256 proposalId);

        function confirmProposal(uint256 proposalId) external;

        // ── WALLET APPROVAL (admin/moderator invite) ─────────
        function proposeApproveWalletAsAdmin(address wallet) external returns (uint256);
        // ── MEMBER JOIN ──────────────────────────────────────
        function joinCoop(
            bytes32 memberId,
            address wallet
        ) external;

        function vinculationMemberToWallet(
            bytes32 memberId,
            address wallet
        ) external;

        // ── DONATIONS ────────────────────────────────────────
        function donate(uint256 amount, bytes32 memberId) external;

        // ── WITHDRAWAL ────────────────────────────────────────
        function withdraw(
            uint256 amount,
            bytes32 memberId
        ) external;

        // ── LOAN LIFECYCLE ───────────────────────────────────
        function createLoanRequisition(
            uint256 amount,
            uint32  parcelscount,
            bytes32 memberId,
            uint32  daysIntervalOfPayment
        ) external returns (uint256);

        function coverLoan(
            uint256 requisitionId,
            uint32  coveragePercentage,
            bytes32 memberId
        ) external;

        function repay(
            uint256 requisitionId,
            uint256 amount,
            bytes32 memberId
        ) external;

        function cancelLoanRequisition(
            uint256 requisitionId,
            bytes32 memberId
        ) external returns (uint256 totalUncoveredAmount);

        // ── ELECTIONS ────────────────────────────────────────
        function openElection(bytes32 candidateId, bytes32 opponent) external;
        function addCandidate(uint32 electionId, bytes32 candidateId) external;
        function voteForModerator(
            uint32  electionId,
            bytes32 candidateId,
            bytes32 memberId
        ) external;
        function closeElection(uint32 electionId) external;

        // ── CONSOLIDATED VIEWS ───────────────────────────────
        function getCoopStats() external view returns (
            uint256 totalDonations,
            uint256 totalBorrowed,
            uint256 availableBalance,
            uint256 contractBalance,
            bool    isActive,
            uint32  activeMemberCount,
            int32   averageReputation
        );

        function getUserFinancials(address user) external view returns (
            uint256 donation,
            uint256 borrowing,
            uint256 lastBorrowTime,
            uint256 inCoverage,
            uint256 withdrawable,
            uint256 allowance
        );

        // ── INDIVIDUAL VIEWS ─────────────────────────────────
        function isWalletApproved(address wallet) external view returns (bool);
        function isMember(address wallet)         external view returns (bool);
        function getReputation(bytes32 memberId)  external view returns (int32);
        function getMemberId(address wallet)      external view returns (bytes32);
        function isWalletVinculated(address wallet) external view returns (bool);
        function isModerator(bytes32 memberId)    external view returns (bool);
        function canUserBorrow(address user, uint256 amount) external view returns (bool);

        // Admin views
        function getAdmins()           external view returns (address[] memory);
        function isAdmin(address a)    external view returns (bool);
        function adminThreshold()      external view returns (uint256);

        function getProposal(uint256 proposalId) external view returns (
            uint8   pType,
            uint256 confirmations,
            bool    executed,
            uint256 createdAt,
            bool    requiresUnanimous
        );

        // Loan views
        function getRequisitionInfo(uint256 requisitionId) external view returns (
            uint256   requisitionId_,
            address   borrower,
            uint256   amount,
            uint32    minimumCoverage,
            uint32    currentCoverage,
            uint8     status,
            uint256   creationTime,
            address[] coveringLenders,
            uint32    parcelsCount
        );

        function getLoanContract(uint256 requisitionId) external view returns (
            address   walletAddress,
            uint256   requisitionId_,
            uint8     status,
            uint32    parcelsCount,
            uint32    parcelsPending,
            uint256   parcelsValues,
            uint256[] paymentDates,
            uint256[] parcelsAmounts,
            uint256   creationTime
        );

        function getNextPaymentAmount(uint256 requisitionId) external view returns (uint256 paymentAmount, bool canPay);
        function canPayRequisition(uint256 requisitionId, address borrower) external view returns (bool);
        function getPaymentDates(uint256 requisitionId) external view returns (uint256[] memory);
        function getCoveringLenders(uint256 id)         external view returns (address[] memory);
        function getLenderCoverage(uint256 id, address l) external view returns (uint256);
        function getBorrowerRequisitions(address b)     external view returns (uint256[] memory);
        function isBorrowerOverdue(uint256 requisitionId) external view returns (bool);

        function getRepaymentSummary(uint256 requisitionId) external view returns (
            uint256 totalRemainingDebt,
            uint256 nextPaymentAmount,
            uint256 parcelsRemaining,
            uint256 totalParcels,
            bool    isActive
        );

        // Election views
        function getCurrentElectionId() external view returns (int32);
        function getElectionInfo(uint32 electionId) external view returns (
            uint32    id,
            bytes32[] candidates,
            uint256   startTime,
            uint256   endTime,
            bool      isActive,
            bytes32   winnerId,
            int32     winningVotes,
            int32     totalVotesCast
        );
        function getCandidateVotes(bytes32 candidateId) external view returns (int32);
        function hasMemberVoted(uint32 electionId, bytes32 memberId) external view returns (bool);
    }
}

// ── ERC20 ────────────────────────────────────────────────────
// Minimal interface for the USDT/USDC stand-in token.  Only what
// the donation flow needs to build an `approve` call.

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
        function allowance(address owner, address spender) external view returns (uint256);
    }
}

// ── COOP REGISTRY ────────────────────────────────────────────
// Lightweight on-chain registry.  Does NOT deploy LoanMachine.
// Server deploys LoanMachine directly, then registers here.

sol! {
    #[sol(rpc)]
    contract CoopRegistry {
        function platformAdmin() external view returns (address);  // ← add this

        function registerCoop(string calldata name, address loanMachine)
            external returns (bytes32 coopId);
        function getCoopInstance(bytes32 coopId) external view returns (address);
        function getAllCoops() external view returns (bytes32[] memory);
        function getCoopCount() external view returns (uint256);
        function transferPlatformAdmin(address newAdmin) external;

        function coops(bytes32 coopId) external view returns (
            address loanMachine,
            string  name,
            uint256 registeredAt,
            bool    exists
        );
    }
}