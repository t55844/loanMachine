// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./IReputationSystem.sol";

/**
 * @title  ReputationLib (v3)
 * @notice Internal library — inlined into LoanMachine.
 *
 * v3 changes:
 * ───────────
 * • memberId is now bytes32 = keccak256(abi.encodePacked(coopSalt, cpfOrCnpj))
 * • No CPF/CNPJ validation on-chain — the hash is opaque.
 *   The server is responsible for:
 *     1. Validating CPF/CNPJ format + check digits
 *     2. Generating: memberId = keccak256(abi.encodePacked(salt, cpf))
 *     3. Storing the mapping  hash ↔ raw CPF  in its database
 *     4. Ensuring the same CPF always produces the same hash (same salt per coop)
 *
 * The salt should be:
 *   - Per cooperative (so the same CPF in two different coops produces different hashes)
 *   - Stored securely in the server, NEVER on-chain
 *   - A random 32-byte value generated at cooperative creation time
 *
 * If the server is compromised, the attacker gets the salt and can brute-force
 * CPFs (only ~10^11 possibilities).  For higher security, use a slow hash
 * (bcrypt/argon2) off-chain and store the result, but that means the hash
 * can't be recomputed from Solidity's keccak256 — which is fine since the
 * contract never needs to verify the preimage.
 */
library ReputationLib {

    // ─── STORAGE LAYOUT ──────────────────────────────────────

    struct ReputationStorage {
        // member ↔ wallet registry
        mapping(bytes32  => address)   memberToWallet;      // first/primary wallet
        mapping(address  => bytes32)   walletToMemberId;
        mapping(bytes32  => address[]) walletsOfMember;

        // reputation
        mapping(bytes32 => int32) memberReputation;
        int32                     totalPotentialVotes;
        mapping(bytes32 => bool)  activeMembers;
        bytes32[]                 membersWithPositiveReputation;

        // moderator elections
        mapping(bytes32 => bool)  isModerator;
        mapping(bytes32 => int32) moderatorVotesReceived;
        mapping(uint32  => mapping(bytes32 => bool)) hasVotedInElection;   // electionId → memberId → voted
        uint32                         electionCounter;
        IReputationSystem.ElectionStatus[] elections;
    }

    // ─── EVENTS ──────────────────────────────────────────────
    event MemberRegistered( bytes32 indexed memberId, address indexed wallet, address[] allWallets, bool isFirstWallet, uint256 timestamp);
    event ReputationChanged(bytes32 indexed memberId, int32 points, bool increase, int32 newReputation, uint256 timestamp);
    event ElectionOpened(uint32 indexed electionId, bytes32 indexed candidateId, uint256 startTime, uint256 endTime);
    event ElectionClosed(uint32 indexed electionId, bytes32 indexed winnerId, int32 winningVotes);
    event CandidateAdded(uint32 indexed electionId, bytes32 indexed candidateId);
    event VoteCast(uint32 indexed electionId, bytes32 indexed candidateId, bytes32 indexed memberId, int32 voteWeight);
    event NewModerator(bytes32 indexed memberId, uint32 indexed electionId);
    event UnbeatableMajorityReached(uint32 indexed electionId, bytes32 indexed leadingCandidate, int32 leadingVotes);

    // ─── ERRORS ──────────────────────────────────────────────

    error RS_MemberIdOrWalletInvalid();
    error RS_WalletAlreadyVinculated();
    error RS_WalletAlreadyLinkedToAnotherMember();
    error RS_ActiveElectionExists();
    error RS_ElectionNotActive();
    error RS_MemberAlreadyVoted();
    error RS_InvalidCandidate();
    error RS_NoCandidates();

    // ─── CONSTANTS ───────────────────────────────────────────

    int32 internal constant GAIN_REPAY   = 1;
    int32 internal constant GAIN_COVER   = 2;
    int32 internal constant LOSS_OVERDUE = 3;

    // =========================================================================
    //                        WALLET REGISTRATION
    // =========================================================================

    function registerMemberWallet(
        ReputationStorage storage rs,
        bytes32 memberId,
        address wallet
    ) internal {
        if (memberId == bytes32(0) || wallet == address(0))
            revert RS_MemberIdOrWalletInvalid();

        bytes32 existingId = rs.walletToMemberId[wallet];
        if (existingId != bytes32(0) && existingId != memberId)
            revert RS_WalletAlreadyLinkedToAnotherMember();

        address[] storage wallets = rs.walletsOfMember[memberId];
        for (uint256 i = 0; i < wallets.length; i++) {
            if (wallets[i] == wallet) revert RS_WalletAlreadyVinculated();
        }

        bool isFirstWallet = (rs.memberToWallet[memberId] == address(0));
        if (isFirstWallet) {
            rs.memberToWallet[memberId] = wallet;
        }
        rs.walletToMemberId[wallet] = memberId;
        rs.walletsOfMember[memberId].push(wallet);

        emit MemberRegistered(
            memberId, wallet, rs.walletsOfMember[memberId], isFirstWallet, block.timestamp
        );
    }

    // =========================================================================
    //                        REPUTATION MUTATIONS
    // =========================================================================

    function reputationChange(
        ReputationStorage storage rs,
        bytes32 memberId,
        int32   points,
        bool    increase
    ) internal {
        int32 current = rs.memberReputation[memberId];
        int32 next    = increase ? current + points : current - points;

        _updateTotalPotentialVotes(rs, memberId, current, next);
        rs.memberReputation[memberId] = next;

        emit ReputationChanged(memberId, points, increase, next, block.timestamp);
    }

    function _updateTotalPotentialVotes(
        ReputationStorage storage rs,
        bytes32 memberId,
        int32   oldRep,
        int32   newRep
    ) private {
        int32 oldPos = oldRep > 0 ? oldRep : int32(0);
        int32 newPos = newRep > 0 ? newRep : int32(0);
        rs.totalPotentialVotes += (newPos - oldPos);

        if (oldRep <= 0 && newRep > 0) {
            rs.activeMembers[memberId] = true;
            rs.membersWithPositiveReputation.push(memberId);
        } else if (oldRep > 0 && newRep <= 0) {
            rs.activeMembers[memberId] = false;
            _removeMemberFromActiveList(rs, memberId);
        }
    }

    function _removeMemberFromActiveList(
        ReputationStorage storage rs,
        bytes32 memberId
    ) private {
        bytes32[] storage list = rs.membersWithPositiveReputation;
        uint256 len = list.length;
        for (uint256 i = 0; i < len; i++) {
            if (list[i] == memberId) {
                list[i] = list[len - 1];
                list.pop();
                break;
            }
        }
    }

    // =========================================================================
    //                           ELECTIONS
    // =========================================================================

    function openElection(
        ReputationStorage storage rs,
        bytes32 candidateId,
        bytes32 opponent
    ) internal {
        uint256 len = rs.elections.length;
        if (len > 0 && rs.elections[len - 1].active)
            revert RS_ActiveElectionExists();

        if (
            rs.memberToWallet[candidateId] == address(0) ||
            rs.memberToWallet[opponent]    == address(0)
        ) revert RS_InvalidCandidate();

        bytes32[] memory candidates = new bytes32[](2);
        candidates[0] = candidateId;
        candidates[1] = opponent;

        rs.elections.push(IReputationSystem.ElectionStatus({
            id:                      rs.electionCounter,
            candidates:              candidates,
            startTime:               block.timestamp,
            endTime:                 block.timestamp + 30 days,
            active:                  true,
            totalVotesCast:          0,
            potentialRemainingVotes: rs.totalPotentialVotes,
            winnerId:                bytes32(0),
            winningVotes:            0
        }));

        emit ElectionOpened(rs.electionCounter, candidateId, block.timestamp, block.timestamp + 30 days);
        rs.electionCounter++;
    }

    function addCandidate(
        ReputationStorage storage rs,
        uint32  electionId,
        bytes32 candidateId
    ) internal {
        _requireElectionActive(rs, electionId);
        rs.elections[electionId].candidates.push(candidateId);
        emit CandidateAdded(electionId, candidateId);
    }

    function voteForModerator(
        ReputationStorage storage rs,
        uint32  electionId,
        bytes32 candidateId,
        bytes32 memberId
    ) internal {
        _requireElectionActive(rs, electionId);

        if (rs.hasVotedInElection[electionId][memberId])
            revert RS_MemberAlreadyVoted();

        IReputationSystem.ElectionStatus storage e = rs.elections[electionId];

        bool valid = false;
        for (uint256 i = 0; i < e.candidates.length; i++) {
            if (e.candidates[i] == candidateId) { valid = true; break; }
        }
        if (!valid) revert RS_InvalidCandidate();

        int32 weight = rs.memberReputation[memberId];
        if (weight < 0) weight = 0;

        rs.hasVotedInElection[electionId][memberId] = true;
        rs.moderatorVotesReceived[candidateId] += weight;
        e.totalVotesCast += weight;

        int32 rep = rs.memberReputation[memberId];
        if (rep > 0) e.potentialRemainingVotes -= rep;

        emit VoteCast(electionId, candidateId, memberId, weight);
        _checkAndCloseElectionIfUnbeatable(rs, electionId);
    }

    function closeElection(
        ReputationStorage storage rs,
        uint32 electionId
    ) internal {
        if (electionId >= rs.elections.length) revert RS_ElectionNotActive();

        IReputationSystem.ElectionStatus storage e = rs.elections[electionId];
        if (!e.active)                revert RS_ElectionNotActive();
        if (e.candidates.length == 0) revert RS_NoCandidates();

        e.active = false;

        (bytes32 winnerId, int32 winVotes,,) = _getTopTwoCandidates(rs, electionId);
        rs.isModerator[winnerId] = true;
        e.winnerId     = winnerId;
        e.winningVotes = winVotes;

        emit ElectionClosed(electionId, winnerId, winVotes);
    }

    // ─── ELECTION INTERNALS ──────────────────────────────────

    function _requireElectionActive(
        ReputationStorage storage rs,
        uint32 electionId
    ) private view {
        if (electionId >= rs.elections.length) revert RS_ElectionNotActive();
        IReputationSystem.ElectionStatus storage e = rs.elections[electionId];
        if (block.timestamp > e.endTime || !e.active) revert RS_ElectionNotActive();
    }

    function _checkAndCloseElectionIfUnbeatable(
        ReputationStorage storage rs,
        uint32 electionId
    ) private {
        IReputationSystem.ElectionStatus storage e = rs.elections[electionId];
        if (e.candidates.length < 2) return;

        (bytes32 first, int32 fVotes,, int32 sVotes) = _getTopTwoCandidates(rs, electionId);
        if (first == bytes32(0)) return;

        if (fVotes > sVotes + e.potentialRemainingVotes) {
            e.active       = false;
            e.winnerId     = first;
            e.winningVotes = fVotes;
            rs.isModerator[first] = true;

            emit NewModerator(first, electionId);
            emit UnbeatableMajorityReached(electionId, first, fVotes);
            emit ElectionClosed(electionId, first, fVotes);
        }
    }

    function _getTopTwoCandidates(
        ReputationStorage storage rs,
        uint32 electionId
    ) private view returns (
        bytes32 firstId,  int32 firstVotes,
        bytes32 secondId, int32 secondVotes
    ) {
        IReputationSystem.ElectionStatus storage e = rs.elections[electionId];
        if (e.candidates.length == 0) return (bytes32(0), 0, bytes32(0), 0);

        firstId    = e.candidates[0];
        firstVotes = rs.moderatorVotesReceived[firstId];

        for (uint256 i = 1; i < e.candidates.length; i++) {
            bytes32 cId    = e.candidates[i];
            int32   cVotes = rs.moderatorVotesReceived[cId];
            if (cVotes > firstVotes) {
                secondId = firstId;  secondVotes = firstVotes;
                firstId  = cId;      firstVotes  = cVotes;
            } else if (cVotes > secondVotes) {
                secondId = cId;      secondVotes = cVotes;
            }
        }
    }

    // =========================================================================
    //                        VIEW HELPERS
    // =========================================================================

    function getTopTwoCandidatesView(
        ReputationStorage storage rs,
        uint32 electionId
    ) internal view returns (
        bytes32 firstId,  int32 firstVotes,
        bytes32 secondId, int32 secondVotes
    ) {
        return _getTopTwoCandidates(rs, electionId);
    }

    function electionIsActive(
        ReputationStorage storage rs,
        uint32 electionId
    ) internal view returns (bool) {
        if (electionId >= rs.elections.length) return false;
        IReputationSystem.ElectionStatus storage e = rs.elections[electionId];
        return e.active && block.timestamp <= e.endTime;
    }

    function electionCount(ReputationStorage storage rs)
        internal view returns (uint256)
    {
        return rs.elections.length;
    }
}
