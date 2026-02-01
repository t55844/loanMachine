import { useState } from "react";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";
import { eventSystem } from "../handlers/EventSystem";

export default function TransactionPendingRequisition({
  requisition,
  donationBalances,
  customPercentage,
  setCustomPercentage,
  quickPercentages,
  contract,
  account,
  member,
  onCoverLoan,
  onRefresh,
  stopPropagation,
  isPercentageValid,
  formatUSDT
}) {
  const [covering, setCovering] = useState(false);
  const [needsApproval, setNeedsApproval] = useState(false);
  const [approving, setApproving] = useState(false);
  const [currentCoverageAmount, setCurrentCoverageAmount] = useState("0");

  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { needsUSDTApproval, approveUSDT } = useWeb3();

  // Helper function to show errors
  function showError(error) {
    eventSystem.emit('showToast', {
      message: error.message || error,
      isError: true
    });
  }

  // Helper function to show success
  function showSuccess(message) {
    eventSystem.emit('showToast', {
      message: message,
      type: 'success'
    });
  }

  // Helper function to show warning
  function showWarning(message) {
    eventSystem.emit('showToast', {
      message: message,
      type: 'warning'
    });
  }

  const checkApproval = async (coverageAmount) => {
    try {
      const approvalNeeded = await needsUSDTApproval(coverageAmount);
      return approvalNeeded;
    } catch (err) {
      //console.error("Erro ao verificar aprovação:", err);
      return true;
    }
  };

  const handleApprove = async () => {
    if (!currentCoverageAmount) return;

    setApproving(true);
    try {
      await approveUSDT(currentCoverageAmount);
      showSuccess("USDT aprovado com sucesso!");
      setNeedsApproval(false);
      setCurrentCoverageAmount("0");
    } catch (err) {
      showError("Erro ao aprovar USDT");
    } finally {
      setApproving(false);
    }
  };

  const handleCoverLoan = async (requisitionId, percentage) => {
    if (!contract || !account) {
      showWarning("Por favor, conecte sua carteira primeiro");
      return;
    }

    // Check member data
    if (!member || !member.id) {
      showError("Dados do membro não disponíveis. Por favor, verifique sua conexão com a carteira.");
      return;
    }


    // Convert to uint32 compatible values with validation
    const requisitionIdUint32 = Number(requisitionId);
    const percentageUint32 = Number(percentage);
    const memberIdUint32 = Number(member.id);

    // Validate parameters
    if (isNaN(requisitionIdUint32) || requisitionIdUint32 < 0) {
      showError("ID de requisição inválido");
      return;
    }

    if (isNaN(percentageUint32) || percentageUint32 < 1 || percentageUint32 > 100) {
      showError("Porcentagem de cobertura inválida");
      return;
    }

    if (isNaN(memberIdUint32) || memberIdUint32 < 1) {
      showError("ID do membro inválido");
      return;
    }

    const coverageAmount = parseFloat(requisition.amount) * percentage / 100;
    
    if (parseFloat(donationBalances.free) < coverageAmount) {
      showError(`Saldo de doação livre insuficiente. Você tem ${parseFloat(donationBalances.free).toFixed(2)} USDT livre mas precisa de ${coverageAmount.toFixed(2)} USDT`);
      return;
    }

    // Check if approval is needed
    const approvalNeeded = await checkApproval(coverageAmount.toString());
    if (approvalNeeded) {
      setCurrentCoverageAmount(coverageAmount.toString());
      setNeedsApproval(true);
      showWarning("Por favor, aprove USDT primeiro antes de cobrir este empréstimo");
      return;
    }

    // CRITICAL: Check if the current wallet is properly vinculated to the member ID
    try {
      const isVinculated = await contract.isWalletVinculated(account);
      
      if (!isVinculated) {
        showError("Sua carteira não está vinculada a nenhum membro. Por favor, registre-se primeiro.");
        return;
      }

      // Check if the member ID matches what's registered for this wallet
      const contractMemberId = await contract.getMemberId(account);

      if (Number(contractMemberId.toString()) !== memberIdUint32) {
        showError(`ID do membro não corresponde! Carteira ${account} está vinculada ao membro ${contractMemberId.toString()} mas você está tentando usar o membro ${memberIdUint32}. Por favor, use a carteira correta.`);
        return;
      }


    } catch (err) {
      //console.error("Erro durante a validação do membro:", err);
      showError("Erro ao verificar registro do membro. Por favor, tente novamente.");
      return;
    }

    showTransactionModal(
      {
        method: "coverLoan",
        params: [requisitionIdUint32, percentageUint32, memberIdUint32],
        value: "0"
      },
      {
        type: 'coverLoan',
        requisitionId: requisitionIdUint32,
        percentage: percentageUint32,
        coverageAmount: coverageAmount.toFixed(2),
        loanAmount: requisition.amount,
        borrower: requisition.borrower,
        memberId: memberIdUint32
      }
    );
  };

  // Also update the confirmCoverLoanTransaction function:
  const confirmCoverLoanTransaction = async (transactionData) => {
    setCovering(true);
    try {
      const { params } = transactionData;
      const [requisitionId, percentage, memberId] = params;

      const tx = await contract.coverLoan(requisitionId, percentage, memberId);
      await tx.wait();

      showSuccess(`Cobertura de ${percentage}% do empréstimo #${requisitionId} bem-sucedida`);
      
      if (onCoverLoan) {
        onCoverLoan();
      }
      
      setCustomPercentage("");
      
      await onRefresh();
    } catch (err) {
      showError(err.message || "Falha ao cobrir empréstimo");
    } finally {
      setCovering(false);
    }
  };

  const handlePercentageClick = async (percentage) => {
    const coverageAmount = parseFloat(requisition.amount) * percentage / 100;
    const remainingCoverage = 100 - requisition.currentCoverage;
    const canCover = parseFloat(donationBalances.free) >= coverageAmount;
    const isValid = percentage <= remainingCoverage && percentage > 0;

    if (isValid && canCover) {
      await handleCoverLoan(requisition.id, percentage);
    }
  };

  const handleCustomPercentageCover = async () => {
    const percentage = parseInt(customPercentage);
    if (percentage >= 10 && percentage <= 100) {
      await handleCoverLoan(requisition.id, percentage);
    }
  };

  return (
    <div className="cover-loan-section" onClick={stopPropagation}>
      <h4>Cobrir Este Empréstimo</h4>
      
      {/* Approval Section */}
      {needsApproval && (
        <div className="approval-section" style={{
          padding: '16px',
          backgroundColor: 'rgba(255, 152, 0, 0.1)',
          borderRadius: '8px',
          border: '1px solid var(--accent-orange)',
          marginBottom: '16px'
        }}>
          <h4>Aprovação Necessária</h4>
          <p>Você precisa aprovar {formatUSDT(currentCoverageAmount)} USDT antes de cobrir empréstimos.</p>
          <button 
            onClick={handleApprove}
            disabled={approving}
            className="approve-button"
          >
            {approving ? "Aprovando..." : `Aprovar ${formatUSDT(currentCoverageAmount)} USDT`}
          </button>
        </div>
      )}
      
      <div className="quick-percentages">
        <p>Seleção rápida:</p>
        <div className="percentage-grid">
          {quickPercentages.map((percentage) => {
            const coverageAmount = parseFloat(requisition.amount) * percentage / 100;
            const canCover = parseFloat(donationBalances.free) >= coverageAmount;
            const remainingCoverage = 100 - requisition.currentCoverage;
            const isValid = percentage <= remainingCoverage && percentage > 0;

            return (
              <button
                key={percentage}
                onClick={() => handlePercentageClick(percentage)}
                className={`percentage-button ${!isValid ? 'disabled' : ''} ${!canCover ? 'insufficient' : ''}`}
                disabled={covering || !isValid || !canCover || !member?.id}
                title={
                  !member?.id 
                    ? "Dados do membro não disponíveis"
                    : !isValid 
                      ? `Não é possível cobrir mais de ${remainingCoverage}%` 
                      : !canCover 
                        ? `Precisa de ${coverageAmount.toFixed(2)} USDT (Você tem ${formatUSDT(donationBalances.free)} USDT disponível)` 
                        : `Cobrir ${percentage}% (${coverageAmount.toFixed(2)} USDT)`
                }
              >
                {percentage}%
              </button>
            );
          })}
        </div>
      </div>

      <div className="custom-percentage">
        <p>Ou insira a porcentagem personalizada (10-100):</p>
        <div className="custom-input-group">
          <input
            type="number"
            min="10"
            max="100"
            value={customPercentage}
            onChange={(e) => setCustomPercentage(e.target.value)}
            onClick={stopPropagation}
            placeholder="Insira a porcentagem"
            className="custom-percentage-input"
            disabled={!member?.id}
          />
          <span>%</span>
          <button
            onClick={handleCustomPercentageCover}
            disabled={covering || !customPercentage || !isPercentageValid(requisition, parseInt(customPercentage)) || !member?.id}
            className="custom-cover-button"
          >
            {!member?.id ? "Sem Dados do Membro" : "Cobrir"}
          </button>
        </div>
        {customPercentage && (
          <div className="custom-amount">
            Valor de cobertura: {(parseFloat(requisition.amount) * parseInt(customPercentage) / 100).toFixed(2)} USDT
            {!isPercentageValid(requisition, parseInt(customPercentage)) && (
              <span className="error-text"> 
                {parseFloat(donationBalances.free) < (parseFloat(requisition.amount) * parseInt(customPercentage) / 100) 
                  ? ` - Precisa de ${(parseFloat(requisition.amount) * parseInt(customPercentage) / 100).toFixed(2)} USDT (Você tem ${formatUSDT(donationBalances.free)} USDT disponível)`
                  : ' - Porcentagem inválida'
                }
              </span>
            )}
          </div>
        )}
      </div>

      {/* Gas Cost Modal */}
      <ModalWrapper onConfirm={confirmCoverLoanTransaction} />
    </div>
  );
}