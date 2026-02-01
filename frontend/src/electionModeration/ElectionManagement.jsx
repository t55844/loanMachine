import React, { useState } from 'react';
import CreateElection from './CreateElection';
import VoteAndCandidate from './VoteAndCandidate';
import { useWeb3 } from '../Web3Context'; // NEW: Import useWeb3

const ElectionManagement = ({ contract, currentAccount, member }) => {
  const [activeTab, setActiveTab] = useState('vote');

  const { provider } = useWeb3(); // NEW: Get provider if needed, but not used here

  // Show message if contract is not available
  if (!contract) {
    return (
      <div className="card">
        <h2>Gerenciamento de Eleição</h2>
        <div className="error-message">
          Contrato de reputação não disponível. Por favor, verifique sua configuração.
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
          Votar & Candidatos
        </button>
        <button 
          className={`tab-button ${activeTab === 'create' ? 'active' : ''}`}
          onClick={() => setActiveTab('create')}
        >
          Criar Eleição
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