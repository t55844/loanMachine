import React, { useState } from 'react';
import { useWeb3 } from '../Web3Context'; // NEW: Import useWeb3
import { useToast } from '../handlers/useToast'; // NEW: Import useToast

const CreateElection = ({ contract, currentAccount, member, onElectionCreated }) => {
  const [candidateId, setCandidateId] = useState('');
  const [opponentId, setOpponentId] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const { provider } = useWeb3(); // NEW: Get provider
  const { showError, showSuccess } = useToast(provider, contract); // NEW: Use toast

  // Get memberId from member object with proper null checking
  const memberId = member?.memberId || 0;
  const hasVinculation = member?.hasVinculation || false;

  const handleCreateElection = async () => {
    if (!candidateId || !opponentId) {
      setError('Por favor, insira os IDs de membro do candidato e do oponente');
      return;
    }

    if (!hasVinculation) {
      setError('Você precisa vincular sua carteira a um membro antes de poder criar eleições.');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const tx = await contract.openElection(parseInt(candidateId), parseInt(opponentId));
      await tx.wait();
      showSuccess('Eleição criada com sucesso!'); // UPDATED: Use showSuccess
      setCandidateId('');
      setOpponentId('');
      if (onElectionCreated) onElectionCreated();
    } catch (err) {
      //console.error('Erro ao criar eleição:', err);
      await showError(err); // UPDATED: Use showError
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div>
      <h2>Criar Nova Eleição</h2>
      
      {/* Show member status */}
      {!hasVinculation && (
        <div className="error-message">
          <strong>Carteira Não Vinculada</strong><br />
          Você precisa vincular sua carteira a um membro antes de poder criar eleições.
          Por favor, use a seção "Vincular Membro" no menu lateral.
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
          <strong>Status do Membro:</strong> Vinculado como Membro #{memberId}
        </div>
      )}
      
      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      <div className="vinculate-section">
        <h3>Iniciar uma Eleição de Moderador</h3>
        <p style={{ color: 'var(--text-secondary)', marginBottom: '16px', fontSize: '14px' }}>
          Crie uma nova eleição fornecendo os IDs de membro do candidato e do oponente.
        </p>
        
        <div className="wallet-input-row">
          <input
            type="number"
            placeholder="ID do Membro Candidato"
            value={candidateId}
            onChange={(e) => setCandidateId(e.target.value)}
            className="member-id-input"
            disabled={isLoading || !hasVinculation}
          />
          <input
            type="number"
            placeholder="ID do Membro Oponente"
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
            {isLoading ? 'Criando...' : 'Criar Eleição'}
          </button>
        </div>
        
        <div className="vinculation-info">
          <strong>Nota:</strong> Apenas uma eleição ativa pode existir por vez. 
          As eleições duram 30 dias, a menos que um candidato atinja uma maioria inatingível.
        </div>
      </div>
    </div>
  );
};

export default CreateElection;