// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IReputationSystem {
    struct ElectionStatus {
        uint32 id;
        uint32[] candidates;
        uint256 startTime;
        uint256 endTime;
        bool active;
        uint32 winnerId;
        int32 winningVotes;
        int32 totalVotesCast;
        int32 potentialRemainingVotes;
    }
    

    // Events
    event MemberToWalletVinculation(uint32 indexed memberId, address indexed wallet, uint256 timestamp);
    event ReputationChanged(uint32 indexed memberId, int32 points, bool increase, int32 newReputation, uint256 timestamp);
    event AuthorizedCallerUpdated(address indexed caller, bool authorized);
    event ElectionOpened(uint32 indexed electionId, uint32 indexed candidateId, uint256 startTime, uint256 endTime);
    event CandidateAdded(uint32 indexed electionId, uint32 indexed candidateId);
    event VoteCast(uint32 indexed electionId, uint32 indexed candidateId, uint32 indexed memberId, int32 voteWeight);
    event ElectionClosed(uint32 indexed electionId, uint32 indexed winnerId, int32 winningVotes);
    event UnbeatableMajorityReached(uint32 indexed electionId, uint32 indexed winnerId, int32 winningVotes);


    // Constants
    function REPUTATION_GAIN_BY_REPAYNG_DEBT() external view returns (int32);
    function REPUTATION_LOSS_BY_DEBT_NOT_PAYD() external view returns (int32);
    function REPUTATION_GAIN_BY_COVERING_LOAN() external view returns (int32);

    // Member management
    function vinculationMemberToWallet(uint32 memberId, address wallet) external;
    function getMemberId(address wallet) external view returns (uint32);
    function isWalletVinculated(address wallet) external view returns (bool);
    
    // Reputation management
    function reputationChange(uint32 memberId, int32 points, bool increase) external;
    function getReputation(uint32 memberId) external view returns (int32);
    
    // Election functions
    function openElection(uint32 candidateId) external;
    function addCandidate(uint32 electionId, uint32 candidateId) external;
    function voteForModerator(uint32 electionId, uint32 candidateId, uint32 memberId) external;
    function closeElection(uint32 electionId) external;
    
    // View functions - Replace mappings with getter functions
    function isModerator(uint32 memberId) external view returns (bool);
    function getCandidateVotes(uint32 candidateId) external view returns (int32);
    function hasMemberVoted(uint32 electionId, uint32 memberId) external view returns (bool);
    function getCurrentElectionId() external view returns (int32);
    function getElectionInfo(uint32 electionId) external view returns (
        uint32 id,
        uint32[] memory candidates,
        uint256 startTime,
        uint256 endTime,
        bool active,
        uint32 winnerId,
        int32 winningVotes,
        int32 totalVotesCast
    );
    
    // Authorization
    function setAuthorizedCaller(address caller, bool authorized) external;
    function authorizedCallers(address caller) external view returns (bool);
}