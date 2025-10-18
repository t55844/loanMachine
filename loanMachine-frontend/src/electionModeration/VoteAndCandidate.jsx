import React, { useState, useEffect } from 'react';

const VoteAndCandidate = ({ contract, currentAccount, memberId }) => {
  const [electionInfo, setElectionInfo] = useState(null);
  const [selectedCandidate, setSelectedCandidate] = useState('');
  const [newCandidateId, setNewCandidateId] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isAddingCandidate, setIsAddingCandidate] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  useEffect(() => {
    loadElectionInfo();
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

  const hasVoted = async () => {
    if (!contract || !memberId || !electionInfo) return false;
    try {
      return await contract.hasMemberVoted(electionInfo.id, memberId);
    } catch (err) {
      return false;
    }
  };

  const handleVote = async () => {
    if (!selectedCandidate) {
      setError('Please select a candidate to vote for');
      return;
    }

    setIsLoading(true);
    setError('');
    setSuccess('');

    try {
      const tx = await contract.voteForModerator(
        electionInfo.id,
        parseInt(selectedCandidate),
        memberId
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

  const isElectionActive = electionInfo && electionInfo.active;

  return (
    <div className="card">
      <h2>Election Management</h2>

      {!electionInfo ? (
        <div className="vinculate-section">
          <h3>No Active Election</h3>
          <p style={{ color: 'var(--text-secondary)', textAlign: 'center' }}>
            There is currently no active election. Create a new election to start the voting process.
          </p>
        </div>
      ) : (
        <>
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

          {/* Election Info */}
          <div className="stats-box">
            <h3>Election #{electionInfo.id.toString()} Details</h3>
            <div className="stats-grid">
              <div>
                <strong>Status:</strong> 
                <span style={{ 
                  color: isElectionActive ? 'var(--accent-green)' : 'var(--accent-red)',
                  marginLeft: '8px'
                }}>
                  {isElectionActive ? 'Active' : 'Closed'}
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
              {!isElectionActive && electionInfo.winnerId && (
                <div style={{ gridColumn: '1 / -1', textAlign: 'center', padding: '12px', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
                  <strong style={{ color: 'var(--accent-green)' }}>Winner: Member #{electionInfo.winnerId.toString()} with {electionInfo.winningVotes.toString()} votes</strong>
                </div>
              )}
            </div>
          </div>

          {/* Add Candidate Section */}
          {isElectionActive && (
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
          {isElectionActive && electionInfo.candidates && electionInfo.candidates.length > 0 && (
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

          {/* Refresh Button */}
          <div className="wallet-input-row">
            <button
              onClick={loadElectionInfo}
              className="refresh-button"
              style={{ width: '100%' }}
            >
              Refresh Election Info
            </button>
          </div>
        </>
      )}
    </div>
  );
};

export default VoteAndCandidate;