import React, { useState } from 'react';
import CreateElection from './createElection';
import VoteAndCandidate from './VoteAndCandidate';

const ElectionManagement = ({ contract, currentAccount, memberId }) => {
  const [activeTab, setActiveTab] = useState('vote');

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
            memberId={memberId}
            onElectionCreated={() => setActiveTab('vote')}
          />
        ) : (
          <VoteAndCandidate 
            contract={contract}
            currentAccount={currentAccount}
            memberId={memberId}
          />
        )}
      </div>
    </div>
  );
};

export default ElectionManagement;