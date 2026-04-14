// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title  IReputationSystem (v3)
 * @notice memberId is now bytes32 = keccak256(abi.encodePacked(coopSalt, cpfOrCnpj))
 *
 *         The raw CPF/CNPJ NEVER touches the blockchain.
 *         The server holds:   salt  +  cpf  →  hash
 *         The contract holds: hash  →  wallet, reputation, etc.
 *
 *         LGPD-compliant: no personal identifier is stored on-chain.
 */
interface IReputationSystem {
    struct ElectionStatus {
        uint32    id;
        bytes32[] candidates;
        uint256   startTime;
        uint256   endTime;
        bool      active;
        bytes32   winnerId;
        int32     winningVotes;
        int32     totalVotesCast;
        int32     potentialRemainingVotes;
    }

    // Events
    event MemberToWalletVinculation(bytes32 indexed memberId, address indexed wallet, address[] walletVinculated, uint256 timestamp);
    event ReputationChanged(bytes32 indexed memberId, int32 points, bool increase, int32 newReputation, uint256 timestamp);
    event AuthorizedCallerUpdated(address indexed caller, bool authorized);
    event ElectionOpened(uint32 indexed electionId, bytes32 indexed candidateId, uint256 startTime, uint256 endTime);
    event CandidateAdded(uint32 indexed electionId, bytes32 indexed candidateId);
    event VoteCast(uint32 indexed electionId, bytes32 indexed candidateId, bytes32 indexed memberId, int32 voteWeight);
    event ElectionClosed(uint32 indexed electionId, bytes32 indexed winnerId, int32 winningVotes);
    event UnbeatableMajorityReached(uint32 indexed electionId, bytes32 indexed winnerId, int32 winningVotes);
    event NewModerator(bytes32 indexed memberId, uint32 electionId);

    // Constants
    function REPUTATION_GAIN_BY_REPAYNG_DEBT() external view returns (int32);
    function REPUTATION_LOSS_BY_DEBT_NOT_PAYD() external view returns (int32);
    function REPUTATION_GAIN_BY_COVERING_LOAN() external view returns (int32);

    // Member management
    function vinculationMemberToWallet(bytes32 memberId, address wallet) external;
    function getMemberId(address wallet) external view returns (bytes32);
    function isWalletVinculated(address wallet) external view returns (bool);

    // Reputation
    function getReputation(bytes32 memberId) external view returns (int32);

    // Elections
    function openElection(bytes32 candidateId, bytes32 opponent) external;
    function addCandidate(uint32 electionId, bytes32 candidateId) external;
    function voteForModerator(uint32 electionId, bytes32 candidateId, bytes32 memberId) external;
    function closeElection(uint32 electionId) external;

    // Views
    function isModerator(bytes32 memberId) external view returns (bool);
    function getCandidateVotes(bytes32 candidateId) external view returns (int32);
    function hasMemberVoted(uint32 electionId, bytes32 memberId) external view returns (bool);
    function getCurrentElectionId() external view returns (int32);
    function getElectionInfo(uint32 electionId) external view returns (
        uint32 id, bytes32[] memory candidates, uint256 startTime, uint256 endTime,
        bool active, bytes32 winnerId, int32 winningVotes, int32 totalVotesCast
    );

    function authorizedCallers(address caller) external view returns (bool);
}
