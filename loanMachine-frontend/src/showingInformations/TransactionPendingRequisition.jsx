

// TransactionPendingRequisition.jsx
import { useState } from "react";
import { useToast } from "../handlers/useToast";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";

export default function TransactionPendingRequisition({
  requisition,
  donationBalances,
  customPercentage,
  setCustomPercentage,
  quickPercentages,
  contract,
  account,
  onCoverLoan,
  onRefresh,
  stopPropagation,
  isPercentageValid,
  formatUSDT,
  showToast
}) {
  const [covering, setCovering] = useState(false);
  const [needsApproval, setNeedsApproval] = useState(false);
  const [approving, setApproving] = useState(false);
  const [currentCoverageAmount, setCurrentCoverageAmount] = useState("0");

  const { handleContractError } = useToast();
  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { needsUSDTApproval, approveUSDT } = useWeb3();

  const checkApproval = async (coverageAmount) => {
    try {
      const approvalNeeded = await needsUSDTApproval(coverageAmount);
      return approvalNeeded;
    } catch (err) {
      console.error("Error checking approval:", err);
      return true;
    }
  };

  const handleApprove = async () => {
    if (!currentCoverageAmount) return;

    setApproving(true);
    try {
      await approveUSDT(currentCoverageAmount);
      showToast("USDT approved successfully!", "success");
      setNeedsApproval(false);
      setCurrentCoverageAmount("0");
    } catch (err) {
      console.error("Error approving USDT:", err);
      showToast("Error approving USDT", "error");
    } finally {
      setApproving(false);
    }
  };

  const handleCoverLoan = async (requisitionId, percentage) => {
    if (!contract || !account) {
      showToast("Please connect your wallet first");
      return;
    }

    const coverageAmount = parseFloat(requisition.amount) * percentage / 100;
    
    if (parseFloat(donationBalances.free) < coverageAmount) {
      showToast(`Insufficient free donation balance. You have ${parseFloat(donationBalances.free).toFixed(2)} USDT free but need ${coverageAmount.toFixed(2)} USDT`, "error");
      return;
    }

    // Check if approval is needed
    const approvalNeeded = await checkApproval(coverageAmount.toString());
    if (approvalNeeded) {
      setCurrentCoverageAmount(coverageAmount.toString());
      setNeedsApproval(true);
      showToast("Please approve USDT first before covering this loan", "warning");
      return;
    }

    showTransactionModal(
      {
        method: "coverLoan",
        params: [requisitionId, percentage],
        value: "0"
      },
      {
        type: 'coverLoan',
        requisitionId: requisitionId,
        percentage: percentage,
        coverageAmount: coverageAmount.toFixed(2),
        loanAmount: requisition.amount,
        borrower: requisition.borrower
      }
    );
  };

  const confirmCoverLoanTransaction = async (transactionData) => {
    setCovering(true);
    try {
      const { params } = transactionData;
      const [requisitionId, percentage] = params;

      const tx = await contract.coverLoan(requisitionId, percentage);
      await tx.wait();

      showToast(`Successfully covered ${percentage}% of loan #${requisitionId}`, "success");
      
      if (onCoverLoan) {
        onCoverLoan();
      }
      
      setCustomPercentage("");
      
      await onRefresh();
    } catch (err) {
      handleContractError(err, "coverLoan");
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
    if (percentage >= 1 && percentage <= 100) {
      await handleCoverLoan(requisition.id, percentage);
    }
  };

  return (
    <div className="cover-loan-section" onClick={stopPropagation}>
      <h4>Cover This Loan</h4>
      
      {/* Approval Section */}
      {needsApproval && (
        <div className="approval-section" style={{
          padding: '16px',
          backgroundColor: 'rgba(255, 152, 0, 0.1)',
          borderRadius: '8px',
          border: '1px solid var(--accent-orange)',
          marginBottom: '16px'
        }}>
          <h4>Approval Required</h4>
          <p>You need to approve {formatUSDT(currentCoverageAmount)} USDT before covering loans.</p>
          <button 
            onClick={handleApprove}
            disabled={approving}
            className="approve-button"
          >
            {approving ? "Approving..." : `Approve ${formatUSDT(currentCoverageAmount)} USDT`}
          </button>
        </div>
      )}
      
      <div className="quick-percentages">
        <p>Quick select:</p>
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
                disabled={covering || !isValid || !canCover}
                title={
                  !isValid 
                    ? `Cannot cover more than ${remainingCoverage}%` 
                    : !canCover 
                      ? `Need ${coverageAmount.toFixed(2)} USDT (You have ${formatUSDT(donationBalances.free)} USDT available)` 
                      : `Cover ${percentage}% (${coverageAmount.toFixed(2)} USDT)`
                }
              >
                {percentage}%
              </button>
            );
          })}
        </div>
      </div>

      <div className="custom-percentage">
        <p>Or enter custom percentage (1-100):</p>
        <div className="custom-input-group">
          <input
            type="number"
            min="1"
            max="100"
            value={customPercentage}
            onChange={(e) => setCustomPercentage(e.target.value)}
            onClick={stopPropagation}
            placeholder="Enter percentage"
            className="custom-percentage-input"
          />
          <span>%</span>
          <button
            onClick={handleCustomPercentageCover}
            disabled={covering || !customPercentage || !isPercentageValid(requisition, parseInt(customPercentage))}
            className="custom-cover-button"
          >
            Cover
          </button>
        </div>
        {customPercentage && (
          <div className="custom-amount">
            Coverage amount: {(parseFloat(requisition.amount) * parseInt(customPercentage) / 100).toFixed(2)} USDT
            {!isPercentageValid(requisition, parseInt(customPercentage)) && (
              <span className="error-text"> 
                {parseFloat(donationBalances.free) < (parseFloat(requisition.amount) * parseInt(customPercentage) / 100) 
                  ? ` - Need ${(parseFloat(requisition.amount) * parseInt(customPercentage) / 100).toFixed(2)} USDT (You have ${formatUSDT(donationBalances.free)} USDT available)`
                  : ' - Invalid percentage'
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