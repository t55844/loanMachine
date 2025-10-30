import React, { useState } from 'react';
import CreateElection from './CreateElection';
import VoteAndCandidate from './VoteAndCandidate';

const ElectionManagement = ({ contract, currentAccount, member }) => {
  const [activeTab, setActiveTab] = useState('vote');

  // Show message if contract is not available
  if (!contract) {
    return (
      <div className="card">
        <h2>Election Management</h2>
        <div className="error-message">
          Reputation contract not available. Please check your configuration.
        </div>
      </div>
    );
  }

  return (
    <div className="interaction-tabs">
      <div className="tab-header">
        <button 
          className={`tab-button ${activeTab === 'vote' ? 'active' : ''}`}
          onClick={() => setActiveTab('vote')}
        >
          Vote & Candidates
        </button>
        <button 
          className={`tab-button ${activeTab === 'create' ? 'active' : ''}`}
          onClick={() => setActiveTab('create')}
        >
          Create Election
        </button>
      </div>
      
      <div className="tab-content">
        {activeTab === 'create' ? (
          <CreateElection 
            contract={contract}
            currentAccount={currentAccount}
            member={member}
            onElectionCreated={() => setActiveTab('vote')}
          />
        ) : (
          <VoteAndCandidate 
            contract={contract}
            currentAccount={currentAccount}
            member={member}
          />
        )}
      </div>
    </div>
  );
};

export default ElectionManagement;