import React, { useState, useEffect } from 'react';
import { fetchLastElection } from '../graphql-frontend-query';
import { useWeb3 } from '../Web3Context'; // NEW: Import useWeb3 for provider/contract
import { useToast } from '../handlers/useToast'; // NEW: Import useToast

const VoteAndCandidate = ({ contract, currentAccount, member }) => {
  const [electionInfo, setElectionInfo] = useState(null);
  const [lastElection, setLastElection] = useState(null);
  const [selectedCandidate, setSelectedCandidate] = useState('');
  const [newCandidateId, setNewCandidateId] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isAddingCandidate, setIsAddingCandidate] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  const { provider } = useWeb3(); // NEW: Get provider
  const { showError, showSuccess } = useToast(provider, contract); // NEW: Use toast with provider/contract

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
      //console.error('Erro ao carregar informações da eleição:', err);
      await showError(err); // UPDATED: Use showError
    }
  };

  const loadLastElection = async () => {
    try {
      const lastElectionData = await fetchLastElection();
      if (lastElectionData) {
        setLastElection(lastElectionData);
      
      }
    } catch (err) {
      //console.error('Erro ao carregar última eleição:', err);
      await showError(err); // UPDATED: Use showError
    }
  };

  const handleVote = async () => {
    if (!selectedCandidate) {
      setError('Por favor, selecione um candidato para votar');
      return;
    }

    if (!hasVinculation || !memberId || memberId === 0) {
      setError('Você precisa vincular sua carteira a um membro antes de poder votar. Por favor, use a seção "Vincular Membro" no menu lateral.');
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
      showSuccess('Voto emitido com sucesso!'); // UPDATED: Use showSuccess
      setSelectedCandidate('');
      loadElectionInfo();
    } catch (err) {
      //console.error('Erro ao votar:', err);
      await showError(err); // UPDATED: Use showError
    } finally {
      setIsLoading(false);
    }
  };

  const handleAddCandidate = async () => {
    if (!newCandidateId) {
      setError('Por favor, insira um ID de membro candidato');
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
      showSuccess('Candidato adicionado com sucesso!'); // UPDATED: Use showSuccess
      setNewCandidateId('');
      loadElectionInfo();
    } catch (err) {
      //console.error('Erro ao adicionar candidato:', err);
      await showError(err); // UPDATED: Use showError
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
    <div pw-test-id="election-section">
      <h2>Gerenciamento de Eleição</h2>

      {/* Show member status */}
      {!hasVinculation && (
        <div className="error-message">
          <strong>Carteira Não Vinculada</strong><br />
          Você precisa vincular sua carteira a um membro antes de poder participar das eleições.
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
            <h3>Eleição Ativa #{electionInfo.id.toString()}</h3>
            <div className="stats-grid">
              <div>
                <strong>Status:</strong> 
                <span style={{ 
                  color: 'var(--accent-green)',
                  marginLeft: '8px'
                }}>
                  Ativa
                </span>
              </div>
              <div>
                <strong>Total de Votos:</strong> {electionInfo.totalVotesCast.toString()}
              </div>
              <div>
                <strong>Início:</strong> {formatTime(electionInfo.startTime)}
              </div>
              <div>
                <strong>Término:</strong> {formatTime(electionInfo.endTime)}
              </div>
            </div>
          </div>

          {/* Add Candidate Section */}
          {hasVinculation && (
            <div className="vinculate-section">
              <h3>Adicionar Candidato</h3>
              <div className="wallet-input-row">
                <input
                  type="number"
                  placeholder="Novo ID de Membro Candidato"
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
                  {isAddingCandidate ? 'Adicionando...' : 'Adicionar Candidato'}
                </button>
              </div>
            </div>
          )}

          {/* Voting Section */}
          {electionInfo.candidates && electionInfo.candidates.length > 0 && hasVinculation && (
            <div pw-test-id="voting-section" className="vinculate-section">
              <h3>Emitir Seu Voto</h3>
              
              <div style={{ marginBottom: '16px' }}>
                <strong>Candidatos:</strong>
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
                        Membro #{candidate.toString()}
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
                  {isLoading ? 'Votando...' : 'Emitir Voto'}
                </button>
              </div>

              <div className="vinculation-info">
                <strong>Poder de Voto:</strong> Seu peso de voto é igual à sua pontuação de reputação. 
                Reputação mais alta significa mais influência na eleição.
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
                <h3 style={{
                  textAlign: 'center',
                  padding: '20px',
                  background: 'linear-gradient(135deg, var(--accent-green) 0%, rgba(0, 192, 135, 0.1) 100%)',
                  borderRadius: '12px',
                  border: '2px solid var(--accent-green)',
                  marginBottom: '20px'
                }}>
                  <div style={{ fontSize: '18px', fontWeight: 'bold', marginBottom: '8px', color: 'white' }}>
                    🏆 VENCEDOR DA ÚLTIMA ELEIÇÃO
                  </div>
                  <h4 style={{ fontSize: '24px', fontWeight: 'bold', color: 'white', marginBottom: '8px' }}>
                    Membro #{lastElection.winnerId.toString()}
                  </h4>
                  <h4 style={{ fontSize: '16px', color: 'white' }}>
                    com {lastElection.winningVotes.toString()} votos
                  </h4>
                  <div style={{ fontSize: '14px', color: 'rgba(255,255,255,0.8)', marginTop: '8px' }}>
                    Eleição #{lastElection.electionId.toString()} • {lastElection.blockTimestamp}
                  </div>
                </h3>
              )}

              <h3>Resultados da Eleição #{lastElection.electionId.toString()}</h3>
              
              {/* Election Summary */}
              <div className="stats-grid">
                <div>
                  <strong>Vencedor:</strong> Membro #{lastElection.winnerId.toString()}
                </div>
                <div>
                  <strong>Votos Vencedores:</strong> {lastElection.winningVotes.toString()}
                </div>
                <div>
                  <strong>ID da Eleição:</strong> {lastElection.electionId.toString()}
                </div>
                <div>
                  <strong>Encerrada Em:</strong> {lastElection.blockTimestamp}
                </div>
              </div>
            </div>
          ) : (
            <div className="vinculate-section">
              <h3>Nenhum Histórico de Eleição</h3>
              <p style={{ color: 'var(--text-secondary)', textAlign: 'center' }}>
                Não há eleições passadas para exibir. Crie uma nova eleição para iniciar o processo de votação.
              </p>
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default VoteAndCandidate;