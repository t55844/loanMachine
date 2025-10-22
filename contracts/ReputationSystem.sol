// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./interfaces/IReputationSystem.sol";

contract ReputationSystem is IReputationSystem, ReentrancyGuard, Ownable {
    
    int32 public constant override REPUTATION_GAIN_BY_REPAYNG_DEBT = 1;
    int32 public constant override REPUTATION_GAIN_BY_COVERING_LOAN = 2;
    int32 public constant override REPUTATION_LOSS_BY_DEBT_NOT_PAYD = 3;
    
    // Map of authorized contracts that can call reputationChange
    mapping(address => bool) public override authorizedCallers;

    // User system mappings - WITH REVERSE MAPPING
    mapping(uint32 => address) private memberToWallet; // Primary wallet (first vinculated)
    mapping(address => uint32) public walletToMemberId; // REVERSE MAPPING
    mapping(uint32 => address[]) public walletsOfMember;
    mapping(uint32 => int32) public memberReputation;

    // INCREMENTAL TRACKING FOR POTENTIAL VOTES - SOLUTION 1
    int32 public totalPotentialVotes;
    mapping(uint32 => bool) private activeMembers; // Track members with positive reputation
    uint32[] private membersWithPositiveReputation; // Array of active member IDs

    // Election and moderation mappings
    mapping(uint32 => bool) public override isModerator;
    mapping(uint32 => int32) public moderatorVotesReceived;
    mapping(uint32 => mapping(uint32 => bool)) public hasVotedInElection; 

    uint32 private electionCounter;
    ElectionStatus[] public elections; 

    // Custom errors
    error ReputationSystem_MemberIdOrWalletInvalid();
    error ReputationSystem_WalletAlreadyVinculated();
    error ReputationSystem_UnauthorizedCaller();
    error ReputationSystem_ActiveElectionExists();
    error ReputationSystem_ElectionNotActive();
    error ReputationSystem_MemberAlreadyVoted();
    error ReputationSystem_InvalidCandidate();
    error ReputationSystem_NoCandidates();
    error ReputationSystem_ElectionExpired();
    error ReputationSystem_WalletAlreadyLinkedToAnotherMember();
    
    modifier onlyAuthorized() {
        if (!authorizedCallers[msg.sender]) revert ReputationSystem_UnauthorizedCaller();
        _;
    }

    modifier registerMemberData(uint32 memberId, address wallet) {
        if (memberId == 0 || wallet == address(0)) revert ReputationSystem_MemberIdOrWalletInvalid();
        // Removed: if (memberToWallet[memberId] > address(0)) revert ReputationSystem_WalletAlreadyVinculated(); // Now allows multiple
        if (walletToMemberId[wallet] != 0 && walletToMemberId[wallet] != memberId) {
            revert ReputationSystem_WalletAlreadyLinkedToAnotherMember();
        }
        _;
    }

    modifier validMember(uint32 memberId, address wallet) {
        if (memberId == 0 || memberToWallet[memberId] == address(0)) revert ReputationSystem_MemberIdOrWalletInvalid();
        if (memberId == 0 || walletToMemberId[wallet] == 0) revert ReputationSystem_MemberIdOrWalletInvalid();
        _;
    }

    modifier hasActiveElection() {
        if (elections.length > 0 && elections[elections.length - 1].active) {
            revert ReputationSystem_ActiveElectionExists();
        }
        _;
    }

    modifier electionExists(uint32 electionId) {
        if (electionId >= elections.length) revert ReputationSystem_ElectionNotActive();
        _;
    }

    modifier electionActive(uint32 electionId) {
        ElectionStatus storage election = elections[electionId];
        if (block.timestamp > election.endTime || !election.active) revert ReputationSystem_ElectionNotActive();
        _;
    }

    constructor() {
        authorizedCallers[msg.sender] = true;
    }

    // Interface function implementations

    function vinculationMemberToWallet(uint32 memberId, address wallet) 
        external 
        override
        registerMemberData(memberId, wallet) 
    {
        // Check for duplicate wallet on this member
        for (uint i = 0; i < walletsOfMember[memberId].length; i++) {
            if (walletsOfMember[memberId][i] == wallet) {
                revert ReputationSystem_WalletAlreadyVinculated();
            }
        }

        if (memberToWallet[memberId] == address(0)) {
            memberToWallet[memberId] = wallet; // Set primary only if not set
        }
        walletToMemberId[wallet] = memberId;
        walletsOfMember[memberId].push(wallet);
        address[] memory wallets = walletsOfMember[memberId];

        emit MemberToWalletVinculation(memberId, wallet, wallets, block.timestamp);
    }

    function getMemberId(address wallet) external view override returns (uint32) {
        return walletToMemberId[wallet];
    }

    function isWalletVinculated(address wallet) external view override returns (bool) {
        return walletToMemberId[wallet] != 0;
    }

    // UPDATED: reputationChange with incremental potential votes tracking
    function reputationChange(uint32 memberId, int32 points, bool increase) 
        external 
        onlyAuthorized
    {
        int32 currentReputation = memberReputation[memberId];
        int32 newReputation;
        
        if (increase) {
            newReputation = currentReputation + points;
        } else {
            newReputation = currentReputation - points;
        }
        
        // Update total potential votes incrementally
        _updateTotalPotentialVotes(memberId, currentReputation, newReputation);
        
        memberReputation[memberId] = newReputation;
        emit ReputationChanged(memberId, points, increase, newReputation, block.timestamp);
    }

    function getReputation(uint32 memberId) external view override returns (int32) {
        return memberReputation[memberId];
    }

    function openElection(uint32 candidateId,uint32 oponent) external override hasActiveElection {
        if(memberToWallet[candidateId] == address(0) || memberToWallet[oponent] == address(0) ) revert ReputationSystem_InvalidCandidate();

        ElectionStatus memory newElection;
        newElection.id = electionCounter++;
        newElection.candidates = new uint32[](2);
        newElection.candidates[0] = candidateId;
        newElection.candidates[1] = oponent;
        newElection.startTime = block.timestamp;
        newElection.endTime = block.timestamp + 30 days;
        newElection.active = true;
        newElection.totalVotesCast = 0;
        
        // OPTIMIZED: Use pre-calculated total potential votes
        newElection.potentialRemainingVotes = _calculateTotalPotentialVotes();
        
        elections.push(newElection);
        emit ElectionOpened(newElection.id, candidateId, newElection.startTime, newElection.endTime);
    }

    function addCandidate(uint32 electionId, uint32 candidateId) 
        external 
        override
        electionExists(electionId) 
        electionActive(electionId)
    {
        ElectionStatus storage election = elections[electionId];
        election.candidates.push(candidateId);
        emit CandidateAdded(electionId, candidateId);
    }

    function voteForModerator(uint32 electionId, uint32 candidateId, uint32 memberId) 
        external 
        override
        validMember(memberId, msg.sender)
        electionExists(electionId)
        electionActive(electionId)
    {
        ElectionStatus storage election = elections[electionId];
        
        if (hasVotedInElection[electionId][memberId]) revert ReputationSystem_MemberAlreadyVoted();
        
        bool validCandidate = false;
        for (uint i = 0; i < election.candidates.length; i++) {
            if (election.candidates[i] == candidateId) {
                validCandidate = true;
                break;
            }
        }
        if (!validCandidate) revert ReputationSystem_InvalidCandidate();
        
        int32 voteWeight = memberReputation[memberId];
        if (voteWeight < 0) voteWeight = 0;
        
        hasVotedInElection[electionId][memberId] = true;
        moderatorVotesReceived[candidateId] += voteWeight;
        election.totalVotesCast += voteWeight;
        _updatePotentialRemainingVotes(electionId, memberId);
        
        emit VoteCast(electionId, candidateId, memberId, voteWeight);
        _checkAndCloseElectionIfUnbeatable(electionId);
    }

    function closeElection(uint32 electionId) 
        external 
        override
        electionExists(electionId)
        onlyOwner
    {
        ElectionStatus storage election = elections[electionId];
        if (!election.active) revert ReputationSystem_ElectionNotActive();
        if (election.candidates.length == 0) revert ReputationSystem_NoCandidates();
        
        election.active = false;
        
        uint32 winnerId = election.candidates[0];
        int32 winningVotes = moderatorVotesReceived[winnerId];
        
        for (uint i = 1; i < election.candidates.length; i++) {
            uint32 candidateId = election.candidates[i];
            int32 candidateVotes = moderatorVotesReceived[candidateId];
            
            if (candidateVotes > winningVotes) {
                winnerId = candidateId;
                winningVotes = candidateVotes;
            }
        }
        
        isModerator[winnerId] = true;
        election.winnerId = winnerId;
        election.winningVotes = winningVotes;
        
        emit ElectionClosed(electionId, winnerId, winningVotes);
    }

    function setAuthorizedCaller(address caller, bool authorized) external override onlyOwner {
        authorizedCallers[caller] = authorized;
        emit AuthorizedCallerUpdated(caller, authorized);
    }

    function getCandidateVotes(uint32 candidateId) external view override returns (int32) {
        return moderatorVotesReceived[candidateId];
    }

    function hasMemberVoted(uint32 electionId, uint32 memberId) external view override returns (bool) {
        return hasVotedInElection[electionId][memberId];
    }

    function getCurrentElectionId() external view override returns (int32) {
        if (elections.length == 0) return -1;
        ElectionStatus storage lastElection = elections[elections.length - 1];
        return lastElection.active ? int32(lastElection.id) : -1;
    }

    function getElectionInfo(uint32 electionId) 
        external 
        view 
        override
        electionExists(electionId)
        returns (
            uint32 id,
            uint32[] memory candidates,
            uint256 startTime,
            uint256 endTime,
            bool active,
            uint32 winnerId,
            int32 winningVotes,
            int32 totalVotesCast
        ) 
    {
        ElectionStatus storage election = elections[electionId];
        return (
            election.id,
            election.candidates,
            election.startTime,
            election.endTime,
            election.active,
            election.winnerId,
            election.winningVotes,
            election.totalVotesCast
        );
    }

    function getActiveMemberCount() external view returns (uint32) {
        return uint32(membersWithPositiveReputation.length);
    }

    function getAverageReputation() external view returns (int32) {
        uint32 activeCount = uint32(membersWithPositiveReputation.length);
        if (activeCount == 0) return 0;
        return totalPotentialVotes / int32(activeCount);
    }

    function isMemberActive(uint32 memberId) external view returns (bool) {
        return activeMembers[memberId];
    }

    // Internal functions - OPTIMIZED

    function _calculateTotalPotentialVotes() internal view returns (int32) {
        // OPTIMIZED: Now O(1) - returns pre-calculated total
        return totalPotentialVotes;
    }

    function _updatePotentialRemainingVotes(uint32 electionId, uint32 memberId) internal {
        ElectionStatus storage election = elections[electionId];
        int32 memberRep = memberReputation[memberId];
        if (memberRep > 0) {
            election.potentialRemainingVotes -= memberRep;
        }
    }

    function _updateTotalPotentialVotes(uint32 memberId, int32 oldRep, int32 newRep) internal {

        int32 oldPositive = oldRep > 0 ? oldRep : int32(0);
        int32 newPositive = newRep > 0 ? newRep : int32(0);
        int32 difference = newPositive - oldPositive;
        
        totalPotentialVotes += difference;
        
        // Update active members tracking
        if (oldRep <= 0 && newRep > 0) {
            // Member becomes active
            activeMembers[memberId] = true;
            membersWithPositiveReputation.push(memberId);
        } else if (oldRep > 0 && newRep <= 0) {
            // Member becomes inactive
            activeMembers[memberId] = false;
            _removeMemberFromActiveList(memberId);
        }
    }

    function _removeMemberFromActiveList(uint32 memberId) internal {
        for (uint i = 0; i < membersWithPositiveReputation.length; i++) {
            if (membersWithPositiveReputation[i] == memberId) {
                // Swap with last element and pop
                membersWithPositiveReputation[i] = membersWithPositiveReputation[membersWithPositiveReputation.length - 1];
                membersWithPositiveReputation.pop();
                break;
            }
        }
    }

    function _checkAndCloseElectionIfUnbeatable(uint32 electionId) internal {
        ElectionStatus storage election = elections[electionId];
        
        if (election.candidates.length < 2) return;
        
        (uint32 firstPlaceId, int32 firstPlaceVotes, , int32 secondPlaceVotes) = _getTopTwoCandidates(electionId);
        
        if (firstPlaceId == 0) return;
        
        if (firstPlaceVotes > secondPlaceVotes + election.potentialRemainingVotes) {
            election.active = false;
            election.winnerId = firstPlaceId;
            election.winningVotes = firstPlaceVotes;
            isModerator[firstPlaceId] = true;
            
            emit NewModerator(election.winnerId,electionId);
            emit UnbeatableMajorityReached(electionId, firstPlaceId, firstPlaceVotes);
            emit ElectionClosed(electionId, firstPlaceId, firstPlaceVotes);
        }
    }

    function _getTopTwoCandidates(uint32 electionId) internal view returns (
        uint32 firstPlaceId,
        int32 firstPlaceVotes,
        uint32 secondPlaceId, 
        int32 secondPlaceVotes
    ) {
        ElectionStatus storage election = elections[electionId];
        
        if (election.candidates.length == 0) return (0, 0, 0, 0);
        
        firstPlaceId = election.candidates[0];
        firstPlaceVotes = moderatorVotesReceived[firstPlaceId];
        secondPlaceId = 0;
        secondPlaceVotes = 0;
        
        for (uint i = 1; i < election.candidates.length; i++) {
            uint32 candidateId = election.candidates[i];
            int32 candidateVotes = moderatorVotesReceived[candidateId];
            
            if (candidateVotes > firstPlaceVotes) {
                secondPlaceId = firstPlaceId;
                secondPlaceVotes = firstPlaceVotes;
                firstPlaceId = candidateId;
                firstPlaceVotes = candidateVotes;
            } else if (candidateVotes > secondPlaceVotes) {
                secondPlaceId = candidateId;
                secondPlaceVotes = candidateVotes;
            }
        }
        
        return (firstPlaceId, firstPlaceVotes, secondPlaceId, secondPlaceVotes);
    }

    // Additional view function for checking majority
    function checkUnbeatableMajority(uint32 electionId) 
        external 
        view 
        electionExists(electionId)
        returns (
            bool isUnbeatable,
            uint32 leadingCandidate,
            int32 leadingVotes,
            uint32 secondCandidate,
            int32 secondVotes
        ) 
    {
        ElectionStatus storage election = elections[electionId];
        if (election.candidates.length < 2) return (false, 0, 0, 0, 0);
        
        (leadingCandidate, leadingVotes, secondCandidate, secondVotes) = _getTopTwoCandidates(electionId);
        isUnbeatable = (leadingVotes > secondVotes + election.potentialRemainingVotes);
        
        return (isUnbeatable, leadingCandidate, leadingVotes, secondCandidate, secondVotes);
    }
}