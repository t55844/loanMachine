// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

interface IReputationSystem {
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
    function removeModerator(uint32 memberId) external;
    
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