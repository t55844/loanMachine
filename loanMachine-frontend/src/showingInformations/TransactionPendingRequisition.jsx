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
      console.error("Error checking approval:", err);
      return true;
    }
  };

  const handleApprove = async () => {
    if (!currentCoverageAmount) return;

    setApproving(true);
    try {
      await approveUSDT(currentCoverageAmount);
      showSuccess("USDT approved successfully!");
      setNeedsApproval(false);
      setCurrentCoverageAmount("0");
    } catch (err) {
      showError("Error approving USDT");
    } finally {
      setApproving(false);
    }
  };

  const handleCoverLoan = async (requisitionId, percentage) => {
    if (!contract || !account) {
      showWarning("Please connect your wallet first");
      return;
    }

    // Check member data
    if (!member || !member.id) {
      showError("Member data not available. Please check your wallet connection.");
      return;
    }

    // Convert to uint32 compatible values with validation
    const requisitionIdUint32 = Number(requisitionId);
    const percentageUint32 = Number(percentage);
    const memberIdUint32 = Number(member.id);

    console.log("Cover loan validation:", {
      currentAccount: account,
      memberId: memberIdUint32,
      requisitionId: requisitionIdUint32,
      percentage: percentageUint32
    });

    // Validate parameters
    if (isNaN(requisitionIdUint32) || requisitionIdUint32 < 0) {
      showError("Invalid requisition ID");
      return;
    }

    if (isNaN(percentageUint32) || percentageUint32 < 1 || percentageUint32 > 100) {
      showError("Invalid coverage percentage");
      return;
    }

    if (isNaN(memberIdUint32) || memberIdUint32 < 1) {
      showError("Invalid member ID");
      return;
    }

    const coverageAmount = parseFloat(requisition.amount) * percentage / 100;
    
    if (parseFloat(donationBalances.free) < coverageAmount) {
      showError(`Insufficient free donation balance. You have ${parseFloat(donationBalances.free).toFixed(2)} USDT free but need ${coverageAmount.toFixed(2)} USDT`);
      return;
    }

    // Check if approval is needed
    const approvalNeeded = await checkApproval(coverageAmount.toString());
    if (approvalNeeded) {
      setCurrentCoverageAmount(coverageAmount.toString());
      setNeedsApproval(true);
      showWarning("Please approve USDT first before covering this loan");
      return;
    }

    // CRITICAL: Check if the current wallet is properly vinculated to the member ID
    try {
      console.log("Checking wallet vinculated status...");
      const isVinculated = await contract.isWalletVinculated(account);
      console.log("Is wallet vinculated:", isVinculated);
      
      if (!isVinculated) {
        showError("Your wallet is not vinculated to any member. Please register first.");
        return;
      }

      // Check if the member ID matches what's registered for this wallet
      const contractMemberId = await contract.getMemberId(account);
      console.log("Contract member ID for wallet:", contractMemberId.toString());
      console.log("Our context member ID:", memberIdUint32);

      if (Number(contractMemberId.toString()) !== memberIdUint32) {
        showError(`Member ID mismatch! Wallet ${account} is vinculated to member ${contractMemberId.toString()} but you're trying to use member ${memberIdUint32}. Please use the correct wallet.`);
        return;
      }

      console.log("✅ Member validation passed!");

    } catch (err) {
      console.error("Error during member validation:", err);
      showError("Error verifying member registration. Please try again.");
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

      showSuccess(`Successfully covered ${percentage}% of loan #${requisitionId}`);
      
      if (onCoverLoan) {
        onCoverLoan();
      }
      
      setCustomPercentage("");
      
      await onRefresh();
    } catch (err) {
      showError(err.message || "Cover loan failed");
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
                disabled={covering || !isValid || !canCover || !member?.id}
                title={
                  !member?.id 
                    ? "Member data not available"
                    : !isValid 
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
            disabled={!member?.id}
          />
          <span>%</span>
          <button
            onClick={handleCustomPercentageCover}
            disabled={covering || !customPercentage || !isPercentageValid(requisition, parseInt(customPercentage)) || !member?.id}
            className="custom-cover-button"
          >
            {!member?.id ? "No Member Data" : "Cover"}
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