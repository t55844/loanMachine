// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "./interfaces/IReputationSystem.sol";

contract ReputationSystem is IReputationSystem, ReentrancyGuard, Ownable {
    
    int32 public constant REPUTATION_GAIN_BY_REPAYNG_DEBT = 1;
    int32 public constant REPUTATION_LOSS_BY_DEBT_NOT_PAYD = 3;
    int32 public constant REPUTATION_GAIN_BY_COVERING_LOAN = 2;
    
    // Map of authorized contracts that can call reputationChange
    mapping(address => bool) public override authorizedCallers;

    // User system mappings
    mapping(address => uint32) private memberWallet;
    mapping(uint32 => int32) private memberReputation;

    // Election and moderation mappings - REMOVE OVERRIDE
    mapping(uint32 => bool) public isModerator;
    mapping(uint32 => int32) public moderatorVotesReceived;
    mapping(uint32 => mapping(uint32 => bool)) public hasVotedInElection;


    uint32 private electionCounter;
    ElectionStatus[] public elections; 

    // Custom errors
    error MemberIdOrWalletInvalid();
    error WalletAlreadyVinculated();
    error UnauthorizedCaller();
    error ActiveElectionExists();
    error ElectionNotActive();
    error MemberAlreadyVoted();
    error InvalidCandidate();
    error NoCandidates();
    error ElectionExpired();
    
    modifier onlyAuthorized() {
        if (!authorizedCallers[msg.sender]) revert UnauthorizedCaller();
        _;
    }

    modifier registerMemberData(uint32 memberId, address wallet) {
        if(memberId == 0 || wallet == address(0)) revert MemberIdOrWalletInvalid();
        if(memberWallet[wallet] > 0) revert WalletAlreadyVinculated();
        _;
    }

    modifier validMember(uint32 memberId, address wallet) {
        if(memberId == 0 || memberWallet[wallet] != memberId) revert MemberIdOrWalletInvalid();
        _;
    }

    modifier hasActiveElection() {
        if(elections.length > 0 && elections[elections.length - 1].active) {
            revert ActiveElectionExists();
        }
        _;
    }

    modifier electionExists(uint32 electionId) {
        if(electionId >= elections.length) revert ElectionNotActive();
        _;
    }

    modifier electionActive(uint32 electionId) {
        ElectionStatus storage election = elections[electionId];
        if(block.timestamp > election.endTime || !election.active) revert ElectionNotActive();
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
        memberWallet[wallet] = memberId;
        emit MemberToWalletVinculation(memberId, wallet, block.timestamp);
    }

    function getMemberId(address wallet) external view override returns (uint32) {
        return memberWallet[wallet];
    }

    function isWalletVinculated(address wallet) external view override returns (bool) {
        return memberWallet[wallet] > 0;
    }

    function reputationChange(uint32 memberId, int32 points, bool increase) 
        external 
        override
        onlyAuthorized
        validMember(memberId, msg.sender) 
    {
        int32 currentReputation = memberReputation[memberId];
        int32 newReputation;
        
        if(increase) {
            newReputation = currentReputation + points;
        } else {
            newReputation = currentReputation - points;
        }
        
        memberReputation[memberId] = newReputation;
        emit ReputationChanged(memberId, points, increase, newReputation, block.timestamp);
    }

    function getReputation(uint32 memberId) external view override returns (int32) {
        return memberReputation[memberId];
    }

    function openElection(uint32 candidateId) external override hasActiveElection {
        ElectionStatus memory newElection;
        newElection.id = electionCounter++;
        newElection.candidates = new uint32[](1);
        newElection.candidates[0] = candidateId;
        newElection.startTime = block.timestamp;
        newElection.endTime = block.timestamp + 30 days;
        newElection.active = true;
        newElection.totalVotesCast = 0;
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
        
        if(hasVotedInElection[electionId][memberId]) revert MemberAlreadyVoted();
        
        bool validCandidate = false;
        for(uint i = 0; i < election.candidates.length; i++) {
            if(election.candidates[i] == candidateId) {
                validCandidate = true;
                break;
            }
        }
        if(!validCandidate) revert InvalidCandidate();
        
        int32 voteWeight = memberReputation[memberId];
        if(voteWeight < 0) voteWeight = 0;
        
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
        if(!election.active) revert ElectionNotActive();
        if(election.candidates.length == 0) revert NoCandidates();
        
        election.active = false;
        
        uint32 winnerId = election.candidates[0];
        int32 winningVotes = moderatorVotesReceived[winnerId];
        
        for(uint i = 1; i < election.candidates.length; i++) {
            uint32 candidateId = election.candidates[i];
            int32 candidateVotes = moderatorVotesReceived[candidateId];
            
            if(candidateVotes > winningVotes) {
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

    // Internal functions

    function _calculateTotalPotentialVotes() internal view returns (int32) {
        int32 totalPotential = 0;
        for (uint32 i = 1; i <= type(uint32).max; i++) {
            if (memberReputation[i] > 0) {
                totalPotential += memberReputation[i];
            }
        }
        return totalPotential;
    }

    function _updatePotentialRemainingVotes(uint32 electionId, uint32 memberId) internal {
        ElectionStatus storage election = elections[electionId];
        int32 memberRep = memberReputation[memberId];
        if (memberRep > 0) {
            election.potentialRemainingVotes -= memberRep;
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
        
        for(uint i = 1; i < election.candidates.length; i++) {
            uint32 candidateId = election.candidates[i];
            int32 candidateVotes = moderatorVotesReceived[candidateId];
            
            if(candidateVotes > firstPlaceVotes) {
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