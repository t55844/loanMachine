import React, { useState } from 'react';

const CreateElection = ({ contract, currentAccount, memberId, onElectionCreated }) => {
  const [candidateId, setCandidateId] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const handleCreateElection = async () => {
    if (!candidateId) {
      setError('Please enter a candidate member ID');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const tx = await contract.openElection(parseInt(candidateId));
      await tx.wait();
      setCandidateId('');
      if (onElectionCreated) onElectionCreated();
    } catch (err) {
      console.error('Error creating election:', err);
      setError(err.reason || 'Failed to create election');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="card">
      <h2>Create New Election</h2>
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      <div className="vinculate-section">
        <h3>Start a Moderator Election</h3>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '16px', fontSize: '14px' }}>
          Create a new election by providing the initial candidate's member ID. 
          Additional candidates can be added after creation.
        </p>
        
        <div className="wallet-input-row">
          <input
            type="number"
            placeholder="Candidate Member ID"
            value={candidateId}
            onChange={(e) => setCandidateId(e.target.value)}
            className="member-id-input"
            disabled={isLoading}
          />
          <button
            onClick={handleCreateElection}
            disabled={isLoading || !candidateId}
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