import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useToast } from "../handlers/useToast";
// import Toast from "../handlers/Toast"; // REMOVED: No local Toast needed (use global)
import { useWeb3 } from "../Web3Context";
import { useGasCostModal } from "../handlers/useGasCostModal";

export default function UserLoanContracts({ contract, account, onLoanUpdate }) {
  const [activeLoans, setActiveLoans] = useState([]);
  const [expandedLoan, setExpandedLoan] = useState(null);
  const [loading, setLoading] = useState(true);
  const [paying, setPaying] = useState(false);
  const [approving, setApproving] = useState(false);
  const [needsApproval, setNeedsApproval] = useState({});

  const { provider, loanInterface } = useWeb3(); // NEW: Get provider and loanInterface
  const { showToast, showSuccess, showError, handleContractError } = useToast(provider, contract); // UPDATED: Pass provider/contract
  const { needsUSDTApproval, approveUSDT, member } = useWeb3();
  const { showTransactionModal, ModalWrapper } = useGasCostModal();

  useEffect(() => {
    if (contract && account) {
      loadUserActiveLoans();
    }
  }, [contract, account]);

  // Check approval status when loans change or when expanded loan changes
  useEffect(() => {
    if (activeLoans.length > 0 && expandedLoan) {
      checkApprovalForLoan(expandedLoan);
    }
  }, [activeLoans, expandedLoan]);

  const loadUserActiveLoans = async () => {
    if (!contract || !account) return;
    setLoading(true);
    try {
      const [loans, requisitionIds] = await contract.getActiveLoans(account);
      
      const formattedLoans = await Promise.all(
        loans.map(async (loan, index) => {
          const requisitionId = requisitionIds[index];
          const repaymentSummary = await contract.getRepaymentSummary(requisitionId);
          const [nextPaymentAmount, canPay] = await contract.getNextPaymentAmount(requisitionId);
          const paymentDates = await contract.getPaymentDates(requisitionId);

          return {
            requisitionId: requisitionId.toString(),
            walletAddress: loan.walletAddress,
            status: loan.status,
            parcelsPending: loan.parcelsPending.toString(),
            parcelsValues: ethers.utils.formatUnits(loan.parcelsValues, 6),
            paymentDates: paymentDates.map(date => new Date(Number(date) * 1000)),
            nextPaymentAmount: ethers.utils.formatUnits(nextPaymentAmount, 6),
            nextPaymentAmountWei: nextPaymentAmount,
            canPay: canPay,
            totalRemainingDebt: ethers.utils.formatUnits(repaymentSummary.totalRemainingDebt, 6),
            parcelsRemaining: repaymentSummary.parcelsRemaining.toString(),
            totalParcels: repaymentSummary.totalParcels.toString(),
            isActive: repaymentSummary.isActive
          };
        })
      );

      setActiveLoans(formattedLoans);
    } catch (err) {
      //console.error("Erro ao carregar empréstimos do usuário:", err);
      await handleContractError(err, "loadUserActiveLoans"); // UPDATED: Await handleContractError
    } finally {
      setLoading(false);
    }
  };

  const checkApprovalForLoan = async (requisitionId) => {
    try {
      const loan = activeLoans.find(l => l.requisitionId === requisitionId);
      if (!loan) return;

      const approvalNeeded = await needsUSDTApproval(loan.nextPaymentAmount);
      setNeedsApproval(prev => ({
        ...prev,
        [requisitionId]: approvalNeeded
      }));
    } catch (err) {
      //console.error("Erro ao verificar aprovação:", err);
    }
  };

  const handleApprove = async (loan) => {
    if (!loan) return;

    setApproving(true);
    try {
      // Use the exact Wei amount for approval to ensure precision
      const amountInWei = loan.nextPaymentAmountWei;
      const amountForDisplay = ethers.utils.formatUnits(amountInWei, 6);
      
      await approveUSDT(amountForDisplay);
      showSuccess("USDT aprovado com sucesso!");
      
      // Update approval status immediately
      setNeedsApproval(prev => ({
        ...prev,
        [loan.requisitionId]: false
      }));
      
      // Force a re-check after a short delay to ensure blockchain state is updated
      setTimeout(() => {
        checkApprovalForLoan(loan.requisitionId);
      }, 2000);
      
    } catch (err) {
      //console.error("Erro ao aprovar USDT:", err);
      await handleContractError(err, "approveUSDT"); // UPDATED: Await
      
      // Re-check approval status in case of error
      await checkApprovalForLoan(loan.requisitionId);
    } finally {
      setApproving(false);
    }
  };

  const handlePayInstallment = async (loan) => {
    if (!contract || !account) {
      showToast("Por favor, conecte sua carteira primeiro");
      return;
    }

    // Check if member data is available
    if (!member || !member.id) {
      showToast("Dados do membro não disponíveis. Por favor, verifique sua conexão com a carteira.", "error");
      return;
    }

    // Check if payment is available
    let canPay;
    try {
      canPay = await contract.canPayRequisition(loan.requisitionId, account);
      if (!canPay) {
        showToast("Pagamento não disponível neste momento", "warning");
        return;
      }
    } catch (err) {
      //console.error("Erro ao verificar disponibilidade de pagamento:", err);
      showToast("Erro ao verificar disponibilidade de pagamento", "error");
      return;
    }

    // Final approval check with the exact Wei amount
    try {
      const currentApprovalNeeded = await needsUSDTApproval(loan.nextPaymentAmount);
      
      if (currentApprovalNeeded) {
        showToast("Por favor, aprove USDT primeiro antes de fazer o pagamento", "error");
        setNeedsApproval(prev => ({
          ...prev,
          [loan.requisitionId]: true
        }));
        return;
      }
    } catch (err) {
      //console.error("Erro na verificação final de aprovação:", err);
      
      // If there's an error checking approval, assume approval is needed
      showToast("Erro ao verificar aprovação de USDT. Por favor, tente aprovar novamente.", "error");
      setNeedsApproval(prev => ({
        ...prev,
        [loan.requisitionId]: true
      }));
      return;
    }

    // Show gas cost modal for payment - using the exact Wei amount from the loan data and member ID
    showTransactionModal(
      {
        method: "repay",
        params: [loan.requisitionId, loan.nextPaymentAmountWei.toString(), member.id],
        value: "0"
      },
      {
        type: 'repay',
        requisitionId: loan.requisitionId,
        amount: loan.nextPaymentAmount,
        token: 'USDT',
        memberId: member.id
      }
    );
  };

  const confirmRepayTransaction = async (transactionData) => {
    setPaying(true);
    try {
      const { params } = transactionData;
      const [requisitionId, amount, memberId] = params;

      const tx = await contract.repay(requisitionId, amount, memberId);
      await tx.wait();

      showSuccess(`Pagamento bem-sucedido para empréstimo #${requisitionId}`);
      await loadUserActiveLoans();
      setExpandedLoan(null);
      
      if (onLoanUpdate) {
        onLoanUpdate();
      }
    } catch (err) {
      //console.error("Transação falhou:", err);
      
      if (err.message?.includes('insufficient allowance') || err.reason?.includes('ERC20: insufficient allowance')) {
        setNeedsApproval(prev => ({
          ...prev,
          [requisitionId]: true
        }));
        showToast("Aprovação de USDT necessária. Por favor, aprove USDT primeiro.", "error");
      } else {
        await handleContractError(err, "payInstallment"); // UPDATED: Await for consistency
      }
      throw err;
    } finally {
      setPaying(false);
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 0: return "Ativo";
      case 1: return "Concluído";
      case 2: return "Inadimplente";
      default: return "Desconhecido";
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 0: return "var(--accent-blue)";
      case 1: return "var(--accent-green)";
      case 2: return "var(--accent-red)";
      default: return "var(--text-secondary)";
    }
  };

  const formatDate = (date) => {
    return date.toLocaleDateString();
  };

  const formatAddress = (address) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatUSDT = (amount) => {
    return parseFloat(amount).toFixed(2);
  };

  return (
    <div style={{
      background: 'var(--bg-tertiary)',
      padding: '24px',
      borderRadius: '12px',
      border: '1px solid var(--border-color)',
      marginTop: '16px'
    }}>
      <h2>Meus Contratos de Empréstimo</h2>

      {/* Member Info Display */}
      {member && (
        <div className="member-info-section" style={{
          marginBottom: '16px',
          padding: '12px',
          backgroundColor: 'var(--bg-secondary)',
          borderRadius: '8px',
          border: '1px solid var(--border-color)'
        }}>
          <p style={{ margin: 0, fontSize: '0.9em' }}>
            <strong>ID do Membro:</strong> {member.id} 
            {member.name && ` - ${member.name}`}
          </p>
          {!member.hasVinculation && (
            <p style={{ 
              margin: '4px 0 0 0', 
              fontSize: '0.8em', 
              color: 'var(--accent-orange)' 
            }}>
              ⚠️ Carteira não vinculada a nenhum membro
            </p>
          )}
        </div>
      )}

      {loading ? (
        <p>Carregando seus empréstimos ativos...</p>
      ) : activeLoans.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '20px', color: 'var(--text-secondary)' }}>
          <p>Nenhum contrato de empréstimo ativo encontrado.</p>
          <p style={{ fontSize: '0.9em', marginTop: '8px' }}>
            Seus contratos de empréstimo ativos aparecerão aqui assim que suas requisições estiverem totalmente cobertas.
          </p>
        </div>
      ) : (
        <div className="requisitions-list">
          {activeLoans.map((loan) => (
            <div 
              key={loan.requisitionId} 
              className="requisition-item"
              onClick={() => setExpandedLoan(expandedLoan === loan.requisitionId ? null : loan.requisitionId)}
              style={{ cursor: 'pointer' }}
            >
              <div className="requisition-header">
                <h3>Contrato de Empréstimo #{loan.requisitionId}</h3>
                <span 
                  className="status-badge"
                  style={{ color: getStatusColor(loan.status) }}
                >
                  {getStatusText(loan.status)}
                </span>
              </div>

              <div className="requisition-details">
                <div><strong>Dívida Restante:</strong> {formatUSDT(loan.totalRemainingDebt)} USDT</div>
                <div><strong>Próximo Pagamento:</strong> {formatUSDT(loan.nextPaymentAmount)} USDT</div>
                <div><strong>Progresso:</strong> {loan.totalParcels - loan.parcelsRemaining}/{loan.totalParcels} parcelas pagas</div>
              </div>

              {expandedLoan === loan.requisitionId && (
                <div className="cover-loan-section">
                  <h4>Detalhes do Contrato</h4>
                  
                  <div className="requisition-details" style={{ gridTemplateColumns: '1fr 1fr' }}>
                    <div><strong>Endereço do Contrato:</strong> {formatAddress(loan.walletAddress)}</div>
                    <div><strong>Valor da Parcela:</strong> {formatUSDT(loan.parcelsValues)} USDT</div>
                    <div><strong>Total Pago:</strong> 
                      {formatUSDT(((loan.totalParcels - loan.parcelsRemaining) * parseFloat(loan.parcelsValues)))} USDT
                    </div>
                    <div><strong>Conclusão:</strong> 
                      {Math.round((loan.totalParcels - loan.parcelsRemaining) / loan.totalParcels * 100)}%
                    </div>
                  </div>

                  <div style={{ marginTop: '16px' }}>
                    <strong>Cronograma de Pagamentos:</strong>
                    <div style={{ 
                      maxHeight: '150px', 
                      overflowY: 'auto', 
                      marginTop: '8px',
                      border: '1px solid var(--border-color)',
                      borderRadius: '4px',
                      padding: '8px'
                    }}>
                      {loan.paymentDates.map((date, index) => (
                        <div 
                          key={index} 
                          style={{ 
                            display: 'flex', 
                            justifyContent: 'space-between',
                            padding: '4px 0',
                            fontSize: '0.9em',
                            color: index < loan.totalParcels - loan.parcelsRemaining ? 'var(--accent-green)' : 'var(--text-secondary)'
                          }}
                        >
                          <span>Parcela {index + 1}:</span>
                          <span>{formatDate(date)}</span>
                          <span>
                            {index < loan.totalParcels - loan.parcelsRemaining ? '✅ Pago' : '⏳ Pendente'}
                          </span>
                        </div>
                      ))}
                    </div>
                  </div>

                  {loan.canPay && loan.status === 0 && (
                    <div style={{ marginTop: '16px' }}>
                      {/* Member validation check */}
                      {(!member || !member.id) && (
                        <div style={{ 
                          padding: '12px',
                          backgroundColor: 'rgba(242, 54, 69, 0.1)',
                          borderRadius: '6px',
                          color: 'var(--accent-red)',
                          marginBottom: '12px',
                          textAlign: 'center'
                        }}>
                          ⚠️ Dados do membro não disponíveis. Não é possível processar o pagamento.
                        </div>
                      )}

                      {/* Approval button - shown when approval is needed */}
                      {needsApproval[loan.requisitionId] && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleApprove(loan);
                          }}
                          disabled={approving || !member?.id}
                          className="approve-button"
                          style={{ 
                            width: '100%', 
                            marginBottom: '12px',
                            background: 'var(--accent-orange)'
                          }}
                        >
                          {approving ? "Aprovando..." : `Aprovar USDT (${formatUSDT(loan.nextPaymentAmount)} USDT)`}
                        </button>
                      )}

                      {/* Pay button - only enabled when no approval needed and not currently paying */}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handlePayInstallment(loan);
                        }}
                        disabled={paying || needsApproval[loan.requisitionId] || !member?.id}
                        className="repay-button"
                        style={{ width: '100%' }}
                      >
                        {paying ? "Processando Pagamento..." : `Pagar Parcela (${formatUSDT(loan.nextPaymentAmount)} USDT)`}
                      </button>
                    </div>
                  )}

                  {!loan.canPay && loan.status === 0 && (
                    <div style={{ 
                      textAlign: 'center', 
                      padding: '12px',
                      backgroundColor: 'rgba(158, 158, 158, 0.1)',
                      borderRadius: '6px',
                      color: 'var(--text-secondary)',
                      marginTop: '16px'
                    }}>
                      Próximo pagamento estará disponível na data agendada
                    </div>
                  )}

                  {loan.status === 1 && (
                    <div style={{ 
                      textAlign: 'center', 
                      padding: '12px',
                      backgroundColor: 'rgba(0, 192, 135, 0.1)',
                      borderRadius: '6px',
                      color: 'var(--accent-green)',
                      marginTop: '16px'
                    }}>
                      ✅ Empréstimo totalmente quitado
                    </div>
                  )}

                  {loan.status === 2 && (
                    <div style={{ 
                      textAlign: 'center', 
                      padding: '12px',
                      backgroundColor: 'rgba(242, 54, 69, 0.1)',
                      borderRadius: '6px',
                      color: 'var(--accent-red)',
                      marginTop: '16px'
                    }}>
                      ⚠️ Empréstimo inadimplente
                    </div>
                  )}
                </div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Gas Cost Modal for repayment */}
      <ModalWrapper onConfirm={confirmRepayTransaction} />

      <style jsx>{`
        .approve-button {
          padding: 12px 20px;
          border: none;
          border-radius: 6px;
          background: var(--accent-orange);
          color: white;
          cursor: pointer;
          font-weight: 600;
          font-size: 14px;
          transition: all 0.2s ease;
        }
        .approve-button:hover:not(:disabled) {
          background: #e67e22;
        }
        .approve-button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
        .repay-button {
          padding: 12px 20px;
          border: none;
          border-radius: 6px;
          background: var(--accent-green);
          color: white;
          cursor: pointer;
          font-weight: 600;
          font-size: 14px;
          transition: all 0.2s ease;
        }
        .repay-button:hover:not(:disabled) {
          background: #00a878;
        }
        .repay-button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
}