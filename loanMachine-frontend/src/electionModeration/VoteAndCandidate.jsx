import React, { useState, useEffect } from 'react';
import { fetchLastElection } from '../graphql-frontend-query';

const VoteAndCandidate = ({ contract, currentAccount, member }) => {
  const [electionInfo, setElectionInfo] = useState(null);
  const [lastElection, setLastElection] = useState(null);
  const [selectedCandidate, setSelectedCandidate] = useState('');
  const [newCandidateId, setNewCandidateId] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isAddingCandidate, setIsAddingCandidate] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  // Get memberId from member object with proper null checking
  const memberId = member?.memberId || 0;
  const hasVinculation = member?.hasVinculation || false;

  useEffect(() => {
    loadElectionInfo();
    loadLastElection();
  }, [contract]);

  const loadElectionInfo = async () => {
    if (!contract) return;

    try {
      const currentElectionId = await contract.getCurrentElectionId();
      
      if (currentElectionId > -1) {
        const election = await contract.getElectionInfo(currentElectionId);
        
        setElectionInfo({
          id: election.id,
          candidates: election.candidates,
          startTime: election.startTime,
          endTime: election.endTime,
          active: election.active,
          winnerId: election.winnerId,
          winningVotes: election.winningVotes,
          totalVotesCast: election.totalVotesCast
        });
      } else {
        setElectionInfo(null);
      }
    } catch (err) {
      console.error('Error loading election info:', err);
    }
  };

  const loadLastElection = async () => {
    try {
      const lastElectionData = await fetchLastElection();
      if (lastElectionData) {
        setLastElection(lastElectionData);
      
      }
    } catch (err) {
      console.error('Error loading last election:', err);
    }
  };

  const handleVote = async () => {
    if (!selectedCandidate) {
      setError('Please select a candidate to vote for');
      return;
    }

    if (!hasVinculation || !memberId || memberId === 0) {
      setError('You need to vinculate your wallet to a member before you can vote. Please use the "Vinculate Member" section in the side menu.');
      return;
    }

    setIsLoading(true);
    setError('');
    setSuccess('');

    try {
      const electionId = Number(electionInfo.id);
      const candidateId = Number(selectedCandidate);
      const memberIdNum = Number(memberId);

      const tx = await contract.voteForModerator(
        electionId,
        candidateId,
        memberIdNum
      );
      await tx.wait();
      setSuccess('Vote cast successfully!');
      setSelectedCandidate('');
      loadElectionInfo();
    } catch (err) {
      console.error('Error voting:', err);
      setError(err.reason || 'Failed to cast vote');
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddCandidate = async () => {
    if (!newCandidateId) {
      setError('Please enter a candidate member ID');
      return;
    }

    setIsAddingCandidate(true);
    setError('');
    setSuccess('');

    try {
      const tx = await contract.addCandidate(
        electionInfo.id,
        parseInt(newCandidateId)
      );
      await tx.wait();
      setSuccess('Candidate added successfully!');
      setNewCandidateId('');
      loadElectionInfo();
    } catch (err) {
      console.error('Error adding candidate:', err);
      setError(err.reason || 'Failed to add candidate');
    } finally {
      setIsAddingCandidate(false);
    }
  };

  const formatTime = (timestamp) => {
    return new Date(parseInt(timestamp) * 1000).toLocaleString();
  };

  const formatBlockTimestamp = (blockTimestamp) => {
    return new Date(parseInt(blockTimestamp) * 1000).toLocaleString();
  };

  const isElectionActive = electionInfo && electionInfo.active;
  const hasWinner = lastElection && lastElection.winnerId && lastElection.winnerId !== 0;

  return (
    <div>
      <h2>Election Management</h2>

      {/* Show member status */}
      {!hasVinculation && (
        <div className="error-message">
          <strong>Wallet Not Vinculated</strong><br />
          You need to vinculate your wallet to a member before you can participate in elections.
          Please use the "Vinculate Member" section in the side menu.
        </div>
      )}

      {hasVinculation && (
        <div style={{
          background: 'var(--bg-tertiary)',
          padding: '12px 16px',
          borderRadius: '8px',
          marginBottom: '16px',
          border: '1px solid var(--border-color)'
        }}>
          <strong>Member Status:</strong> Vinculated as Member #{memberId}
        </div>
      )}

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      {success && (
        <div style={{
          color: 'var(--accent-green)',
          marginBottom: '16px',
          padding: '12px',
          backgroundColor: 'rgba(0, 192, 135, 0.1)',
          borderRadius: '6px',
          border: '1px solid var(--accent-green)',
          textAlign: 'center'
        }}>
          {success}
        </div>
      )}

      {/* Active Election Section */}
      {electionInfo ? (
        <>
          {/* Election Info */}
          <div className="stats-box">
            <h3>Active Election #{electionInfo.id.toString()}</h3>
            <div className="stats-grid">
              <div>
                <strong>Status:</strong> 
                <span style={{ 
                  color: 'var(--accent-green)',
                  marginLeft: '8px'
                }}>
                  Active
                </span>
              </div>
              <div>
                <strong>Total Votes Cast:</strong> {electionInfo.totalVotesCast.toString()}
              </div>
              <div>
                <strong>Start Time:</strong> {formatTime(electionInfo.startTime)}
              </div>
              <div>
                <strong>End Time:</strong> {formatTime(electionInfo.endTime)}
              </div>
            </div>
          </div>

          {/* Add Candidate Section */}
          {hasVinculation && (
            <div className="vinculate-section">
              <h3>Add Candidate</h3>
              <div className="wallet-input-row">
                <input
                  type="number"
                  placeholder="New Candidate Member ID"
                  value={newCandidateId}
                  onChange={(e) => setNewCandidateId(e.target.value)}
                  className="member-id-input"
                  disabled={isAddingCandidate}
                />
                <button
                  onClick={handleAddCandidate}
                  disabled={isAddingCandidate || !newCandidateId}
                  className="vinculate-button"
                >
                  {isAddingCandidate ? 'Adding...' : 'Add Candidate'}
                </button>
              </div>
            </div>
          )}

          {/* Voting Section */}
          {electionInfo.candidates && electionInfo.candidates.length > 0 && hasVinculation && (
            <div className="vinculate-section">
              <h3>Cast Your Vote</h3>
              
              <div style={{ marginBottom: '16px' }}>
                <strong>Candidates:</strong>
                <div style={{ marginTop: '8px' }}>
                  {electionInfo.candidates.map((candidate, index) => (
                    <div key={index} style={{
                      padding: '8px 12px',
                      margin: '4px 0',
                      background: 'var(--bg-primary)',
                      borderRadius: '6px',
                      border: '1px solid var(--border-color)'
                    }}>
                      <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                        <input
                          type="radio"
                          name="candidate"
                          value={candidate}
                          checked={selectedCandidate === candidate.toString()}
                          onChange={(e) => setSelectedCandidate(e.target.value)}
                          style={{ marginRight: '8px' }}
                          disabled={isLoading}
                        />
                        Member #{candidate.toString()}
                      </label>
                    </div>
                  ))}
                </div>
              </div>

              <div className="wallet-input-row">
                <button
                  onClick={handleVote}
                  disabled={isLoading || !selectedCandidate}
                  className="confirm-button"
                  style={{ width: '100%' }}
                >
                  {isLoading ? 'Voting...' : 'Cast Vote'}
                </button>
              </div>

              <div className="vinculation-info">
                <strong>Voting Power:</strong> Your vote weight equals your reputation score. 
                Higher reputation means more influence in the election.
              </div>
            </div>
          )}
        </>
      ) : (
        /* Last Election Results Section */
        <>
          {lastElection ? (
            <div className="stats-box">
              {/* Winner Display - More Prominent */}
              {hasWinner && (
                <div style={{
                  textAlign: 'center',
                  padding: '20px',
                  background: 'linear-gradient(135deg, var(--accent-green) 0%, rgba(0, 192, 135, 0.1) 100%)',
                  borderRadius: '12px',
                  border: '2px solid var(--accent-green)',
                  marginBottom: '20px'
                }}>
                  <div style={{ fontSize: '18px', fontWeight: 'bold', marginBottom: '8px', color: 'white' }}>
                    🏆 LAST ELECTION WINNER
                  </div>
                  <div style={{ fontSize: '24px', fontWeight: 'bold', color: 'white', marginBottom: '8px' }}>
                    Member #{lastElection.winnerId.toString()}
                  </div>
                  <div style={{ fontSize: '16px', color: 'white' }}>
                    with {lastElection.winningVotes.toString()} votes
                  </div>
                  <div style={{ fontSize: '14px', color: 'rgba(255,255,255,0.8)', marginTop: '8px' }}>
                    Election #{lastElection.electionId.toString()} • {lastElection.blockTimestamp}
                  </div>
                </div>
              )}

              <h3>Election #{lastElection.electionId.toString()} Results</h3>
              
              {/* Election Summary */}
              <div className="stats-grid">
                <div>
                  <strong>Winner:</strong> Member #{lastElection.winnerId.toString()}
                </div>
                <div>
                  <strong>Winning Votes:</strong> {lastElection.winningVotes.toString()}
                </div>
                <div>
                  <strong>Election ID:</strong> {lastElection.electionId.toString()}
                </div>
                <div>
                  <strong>Closed On:</strong> {lastElection.blockTimestamp}
                </div>
              </div>
            </div>
          ) : (
            <div className="vinculate-section">
              <h3>No Election History</h3>
              <p style={{ color: 'var(--text-secondary)', textAlign: 'center' }}>
                There are no past elections to display. Create a new election to start the voting process.
              </p>
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default VoteAndCandidate;