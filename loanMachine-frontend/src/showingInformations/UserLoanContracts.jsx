import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useToast } from "../handlers/useToast";
import Toast from "../handlers/Toast";
import { useWeb3 } from "../Web3Context";
import { useGasCostModal } from "../handlers/useGasCostModal";

export default function UserLoanContracts({ contract, account, onLoanUpdate }) {
  const [activeLoans, setActiveLoans] = useState([]);
  const [expandedLoan, setExpandedLoan] = useState(null);
  const [loading, setLoading] = useState(true);
  const [paying, setPaying] = useState(false);
  const [approving, setApproving] = useState(false);
  const [needsApproval, setNeedsApproval] = useState({});

  const { toast, showToast, hideToast, handleContractError } = useToast();
  const { needsUSDTApproval, approveUSDT } = useWeb3();
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
      console.error("Error loading user loans:", err);
      handleContractError(err, "loadUserActiveLoans");
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
      console.error("Error checking approval:", err);
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
      showToast("USDT approved successfully!", "success");
      
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
      console.error("Error approving USDT:", err);
      handleContractError(err, "approveUSDT");
      
      // Re-check approval status in case of error
      await checkApprovalForLoan(loan.requisitionId);
    } finally {
      setApproving(false);
    }
  };

  const handlePayInstallment = async (loan) => {
    if (!contract || !account) {
      showToast("Please connect your wallet first");
      return;
    }

    // Check if payment is available
    let canPay;
    try {
      canPay = await contract.canPayRequisition(loan.requisitionId, account);
      if (!canPay) {
        showToast("Payment is not available at this time", "warning");
        return;
      }
    } catch (err) {
      console.error("Error checking payment availability:", err);
      showToast("Error checking payment availability", "error");
      return;
    }

    // Final approval check with the exact Wei amount
    try {

      const currentApprovalNeeded = await needsUSDTApproval(loan.nextPaymentAmount);
      
      if (currentApprovalNeeded) {
        showToast("Please approve USDT first before making payment", "error");
        setNeedsApproval(prev => ({
          ...prev,
          [loan.requisitionId]: true
        }));
        return;
      }


    } catch (err) {
      console.error("Error in final approval check:", err);
      
      // If there's an error checking approval, assume approval is needed
      showToast("Error checking USDT approval. Please try approving again.", "error");
      setNeedsApproval(prev => ({
        ...prev,
        [loan.requisitionId]: true
      }));
      return;
    }

    // Show gas cost modal for payment - using the exact Wei amount from the loan data
    showTransactionModal(
      {
        method: "repay",
        params: [loan.requisitionId, loan.nextPaymentAmountWei.toString()],
        value: "0"
      },
      {
        type: 'repay',
        requisitionId: loan.requisitionId,
        amount: loan.nextPaymentAmount,
        token: 'USDT'
      }
    );
  };

  const confirmRepayTransaction = async (transactionData) => {
    setPaying(true);
    try {
      const { params } = transactionData;
      const [requisitionId, amount] = params;


      const tx = await contract.repay(requisitionId, amount);
      await tx.wait();

      showToast(`Payment successful for loan #${requisitionId}`, "success");
      await loadUserActiveLoans();
      setExpandedLoan(null);
      
      if (onLoanUpdate) {
        onLoanUpdate();
      }
    } catch (err) {
      console.error("Transaction failed:", err);
      
      // If transaction fails due to approval, update the approval status
      if (err.message?.includes('insufficient allowance') || err.reason?.includes('ERC20: insufficient allowance')) {
        setNeedsApproval(prev => ({
          ...prev,
          [requisitionId]: true
        }));
        showToast("USDT approval required. Please approve USDT first.", "error");
      } else {
        handleContractError(err, "payInstallment");
      }
      throw err;
    } finally {
      setPaying(false);
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 0: return "Active";
      case 1: return "Completed";
      case 2: return "Defaulted";
      default: return "Unknown";
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
    <>
      <Toast toast={toast} onClose={hideToast} />
      
      <div style={{
        background: 'var(--bg-tertiary)',
        padding: '24px',
        borderRadius: '12px',
        border: '1px solid var(--border-color)',
        marginTop: '16px'
      }}>
        <h2>My Loan Contracts</h2>

        {loading ? (
          <p>Loading your active loans...</p>
        ) : activeLoans.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '20px', color: 'var(--text-secondary)' }}>
            <p>No active loan contracts found.</p>
            <p style={{ fontSize: '0.9em', marginTop: '8px' }}>
              Your active loan contracts will appear here once your requisitions are fully covered.
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
                  <h3>Loan Contract #{loan.requisitionId}</h3>
                  <span 
                    className="status-badge"
                    style={{ color: getStatusColor(loan.status) }}
                  >
                    {getStatusText(loan.status)}
                  </span>
                </div>

                <div className="requisition-details">
                  <div><strong>Remaining Debt:</strong> {formatUSDT(loan.totalRemainingDebt)} USDT</div>
                  <div><strong>Next Payment:</strong> {formatUSDT(loan.nextPaymentAmount)} USDT</div>
                  <div><strong>Progress:</strong> {loan.totalParcels - loan.parcelsRemaining}/{loan.totalParcels} parcels paid</div>
                </div>

                {expandedLoan === loan.requisitionId && (
                  <div className="cover-loan-section">
                    <h4>Contract Details</h4>
                    
                    <div className="requisition-details" style={{ gridTemplateColumns: '1fr 1fr' }}>
                      <div><strong>Contract Address:</strong> {formatAddress(loan.walletAddress)}</div>
                      <div><strong>Parcel Value:</strong> {formatUSDT(loan.parcelsValues)} USDT</div>
                      <div><strong>Total Paid:</strong> 
                        {formatUSDT(((loan.totalParcels - loan.parcelsRemaining) * parseFloat(loan.parcelsValues)))} USDT
                      </div>
                      <div><strong>Completion:</strong> 
                        {Math.round((loan.totalParcels - loan.parcelsRemaining) / loan.totalParcels * 100)}%
                      </div>
                    </div>

                    <div style={{ marginTop: '16px' }}>
                      <strong>Payment Schedule:</strong>
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
                            <span>Parcel {index + 1}:</span>
                            <span>{formatDate(date)}</span>
                            <span>
                              {index < loan.totalParcels - loan.parcelsRemaining ? '✅ Paid' : '⏳ Pending'}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>

                    {loan.canPay && loan.status === 0 && (
                      <div style={{ marginTop: '16px' }}>
                        {/* Approval button - shown when approval is needed */}
                        {needsApproval[loan.requisitionId] && (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              handleApprove(loan);
                            }}
                            disabled={approving}
                            className="approve-button"
                            style={{ 
                              width: '100%', 
                              marginBottom: '12px',
                              background: 'var(--accent-orange)'
                            }}
                          >
                            {approving ? "Approving..." : `Approve USDT (${formatUSDT(loan.nextPaymentAmount)} USDT)`}
                          </button>
                        )}

                        {/* Pay button - only enabled when no approval needed and not currently paying */}
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handlePayInstallment(loan);
                          }}
                          disabled={paying || needsApproval[loan.requisitionId]}
                          className="repay-button"
                          style={{ width: '100%' }}
                        >
                          {paying ? "Processing Payment..." : `Pay Installment (${formatUSDT(loan.nextPaymentAmount)} USDT)`}
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
                        Next payment will be available on the scheduled date
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
                        ✅ Loan fully repaid
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
                        ⚠️ Loan defaulted
                      </div>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        <button 
          onClick={loadUserActiveLoans}
          className="wallet-button refresh-button"
          style={{ marginTop: '16px', width: '100%' }}
        >
          Refresh Contracts
        </button>
      </div>

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
    </>
  );
} 