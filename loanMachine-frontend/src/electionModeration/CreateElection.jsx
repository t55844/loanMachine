import React, { useState } from 'react';

const CreateElection = ({ contract, currentAccount, member, onElectionCreated }) => {
  const [candidateId, setCandidateId] = useState('');
  const [opponentId, setOpponentId] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  // Get memberId from member object with proper null checking
  const memberId = member?.memberId || 0;
  const hasVinculation = member?.hasVinculation || false;

  const handleCreateElection = async () => {
    if (!candidateId || !opponentId) {
      setError('Please enter both candidate and opponent member IDs');
      return;
    }

    if (!hasVinculation) {
      setError('You need to vinculate your wallet to a member before you can create elections.');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const tx = await contract.openElection(parseInt(candidateId), parseInt(opponentId));
      await tx.wait();
      setCandidateId('');
      setOpponentId('');
      if (onElectionCreated) onElectionCreated();
    } catch (err) {
      console.error('Error creating election:', err);
      setError(err.reason || 'Failed to create election');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div>
      <h2>Create New Election</h2>
      
      {/* Show member status */}
      {!hasVinculation && (
        <div className="error-message">
          <strong>Wallet Not Vinculated</strong><br />
          You need to vinculate your wallet to a member before you can create elections.
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

      <div className="vinculate-section">
        <h3>Start a Moderator Election</h3>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '16px', fontSize: '14px' }}>
          Create a new election by providing both candidate and opponent member IDs.
        </p>
        
        <div className="wallet-input-row">
          <input
            type="number"
            placeholder="Candidate Member ID"
            value={candidateId}
            onChange={(e) => setCandidateId(e.target.value)}
            className="member-id-input"
            disabled={isLoading || !hasVinculation}
          />
          <input
            type="number"
            placeholder="Opponent Member ID"
            value={opponentId}
            onChange={(e) => setOpponentId(e.target.value)}
            className="member-id-input"
            disabled={isLoading || !hasVinculation}
          />
          <button
            onClick={handleCreateElection}
            disabled={isLoading || !candidateId || !opponentId || !hasVinculation}
            className="vinculate-button"
          >
            {isLoading ? 'Creating...' : 'Create Election'}
          </button>
        </div>
        
        <div className="vinculation-info">
          <strong>Note:</strong> Only one active election can exist at a time. 
          Elections run for 30 days unless a candidate reaches an unbeatable majority.
        </div>
      </div>
    </div>
  );
};

export default CreateElection;