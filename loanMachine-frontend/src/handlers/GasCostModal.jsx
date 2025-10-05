// GasCostModal.jsx
import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useWeb3 } from "../Web3Context"; // Import the web3 hook

function GasCostModal({ 
  isOpen, 
  onClose, 
  onConfirm, 
  transactionData,
  transactionContext
}) {
  const [gasCost, setGasCost] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  
  const { contract, provider } = useWeb3(); // Get contract and provider from context

  useEffect(() => {
    if (isOpen && contract && transactionData) {
      estimateGasCost();
    }
  }, [isOpen, transactionData]);

  async function estimateGasCost() {
    try {
      setLoading(true);
      setError("");
      setGasCost(null);

      const { method, params = [], value = "0" } = transactionData;
      
      if (!contract[method]) {
        throw new Error(`Method ${method} not found on contract`);
      }

      // Estimate gas - for USDT transactions, value is 0
      const gasEstimate = await contract.estimateGas[method](...params, {
        value: ethers.BigNumber.from(value)
      });

      // Get gas price
      const gasPrice = await provider.getGasPrice();

      // Calculate total gas cost in ETH (gas is still paid in ETH)
      const gasCostWei = gasEstimate.mul(gasPrice);
      const gasCostEth = ethers.utils.formatEther(gasCostWei);

      setGasCost(gasCostEth);
    } catch (err) {
      console.error("Error estimating gas:", err);
      setError(err.message || "Failed to estimate gas cost");
    } finally {
      setLoading(false);
    }
  }

  if (!isOpen) return null;

  // For USDT transactions, transaction value is 0 (no ETH sent)
  const transactionValue = transactionData?.value ? 
    parseFloat(ethers.utils.formatEther(transactionData.value)) : 0;
  const totalCost = (parseFloat(gasCost || 0) + transactionValue).toFixed(6);

  // Format USDT amount for display
  const formatUSDTAmount = (amount) => {
    if (!amount) return '0';
    try {
      return parseFloat(ethers.utils.formatUnits(amount, 6)).toFixed(2);
    } catch {
      return amount;
    }
  };

  // Render transaction-specific details based on context
  const renderTransactionDetails = () => {
    if (!transactionContext) return null;

    switch (transactionContext.type) {
      case 'coverLoan':
        return (
          <div className="transaction-details">
            <div><strong>Covering Loan:</strong> Requisition #{transactionContext.requisitionId}</div>
            <div><strong>Coverage Percentage:</strong> {transactionContext.percentage}%</div>
            <div><strong>Coverage Amount:</strong> {formatUSDTAmount(transactionContext.coverageAmount)} USDT</div>
          </div>
        );
      case 'repay':
        return (
          <div className="transaction-details">
            <div><strong>Repayment Amount:</strong> {formatUSDTAmount(transactionContext.amount)} USDT</div>
            <div><strong>Requisition ID:</strong> #{transactionContext.requisitionId}</div>
          </div>
        );
      case 'donate':
        return (
          <div className="transaction-details">
            <div><strong>Donation Amount:</strong> {transactionContext.amount} USDT</div>
            <div><strong>Token:</strong> USDT</div>
            <div><strong>From:</strong> {transactionContext.from}</div>
          </div>
        );
      case 'borrow':
        return (
          <div className="transaction-details">
            <div><strong>Loan Amount:</strong> {formatUSDTAmount(transactionContext.amount)} USDT</div>
            <div><strong>Requisition ID:</strong> #{transactionContext.requisitionId}</div>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="confirmation-modal-overlay">
      <div className="confirmation-modal">
        <h3>Transaction Confirmation</h3>
        
        {/* Transaction-specific details */}
        {renderTransactionDetails()}
        
        <div className="gas-info">
          {loading ? (
            <p>Calculating gas cost...</p>
          ) : error ? (
            <div>
              <p className="error-message">⚠️ {error}</p>
              <button onClick={estimateGasCost} className="refresh-button">
                Retry Estimation
              </button>
            </div>
          ) : gasCost ? (
            <div>
              <div className="detail-item">
                <strong>Estimated Gas Cost:</strong>
                <span>{parseFloat(gasCost).toFixed(6)} ETH</span>
              </div>
              
              {/* Show transaction value only for ETH transactions */}
              {transactionValue > 0 && (
                <div className="detail-item">
                  <strong>Transaction Value:</strong>
                  <span>{transactionValue} ETH</span>
                </div>
              )}
              
              {/* For USDT transactions, show note */}
              {transactionContext?.token === 'USDT' && transactionValue === 0 && (
                <div className="detail-item">
                  <strong>Token Transfer:</strong>
                  <span style={{color: 'var(--accent-blue)'}}>
                    {transactionContext.amount} USDT (separate approval)
                  </span>
                </div>
              )}
              
              <div className="detail-item total-cost">
                <strong>Total Network Fee:</strong>
                <span style={{color: 'var(--accent-green)', fontWeight: 'bold'}}>
                  {totalCost} ETH
                </span>
              </div>
              
              <div className="fee-note">
                <small>
                  * Gas fees are paid in ETH, not USDT. This covers the network transaction cost.
                </small>
              </div>
            </div>
          ) : null}
        </div>

        <div className="confirmation-buttons">
          <button 
            onClick={onClose} 
            className="cancel-button"
            disabled={loading}
          >
            Cancel
          </button>
          <button 
            onClick={() => onConfirm(transactionData)}
            className="confirm-button"
            disabled={loading || error}
          >
            {loading ? "Calculating..." : "Confirm Transaction"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default GasCostModal;