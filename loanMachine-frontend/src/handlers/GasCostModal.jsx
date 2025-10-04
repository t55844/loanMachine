// GasCostModal.jsx
import { useState, useEffect } from "react";
import { ethers } from "ethers";

function GasCostModal({ 
  isOpen, 
  onClose, 
  onConfirm, 
  transactionData,
  transactionContext,
  account,
  contract 
}) {
  const [gasCost, setGasCost] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (isOpen && account && contract && transactionData) {
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

      // Estimate gas
      const gasEstimate = await contract.estimateGas[method](...params, {
        value: ethers.BigNumber.from(value)
      });

      // Get gas price
      const provider = new ethers.providers.Web3Provider(window.ethereum);
      const gasPrice = await provider.getGasPrice();

      // Calculate total gas cost in ETH
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

  const transactionValue = transactionData?.value ? 
    parseFloat(ethers.utils.formatEther(transactionData.value)) : 0;
  const totalCost = (parseFloat(gasCost || 0) + transactionValue).toFixed(6);

  // Render transaction-specific details based on context
  const renderTransactionDetails = () => {
    if (!transactionContext) return null;

    switch (transactionContext.type) {
      case 'coverLoan':
        return (
          <div style={{ marginBottom: '1rem', padding: '0.75rem', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
            <div><strong>Covering Loan:</strong> Requisition #{transactionContext.requisitionId}</div>
            <div><strong>Coverage Percentage:</strong> {transactionContext.percentage}%</div>
            <div><strong>Coverage Amount:</strong> {transactionContext.coverageAmount} ETH</div>
          </div>
        );
      case 'repay':
        return (
          <div style={{ marginBottom: '1rem', padding: '0.75rem', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
            <div><strong>Repayment Amount:</strong> {transactionContext.amount} ETH</div>
          </div>
        );
      case 'donate':
        return (
          <div style={{ marginBottom: '1rem', padding: '0.75rem', background: 'var(--bg-tertiary)', borderRadius: '6px' }}>
            <div><strong>Donation Amount:</strong> {transactionContext.amount} ETH</div>
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
              <button onClick={estimateGasCost} className="refresh-button" style={{marginTop: '10px'}}>
                Retry Estimation
              </button>
            </div>
          ) : gasCost ? (
            <div>
              <div className="detail-item">
                <strong>Estimated Gas Cost:</strong>
                <span>{parseFloat(gasCost).toFixed(6)} ETH</span>
              </div>
              {transactionValue > 0 && (
                <div className="detail-item">
                  <strong>Transaction Value:</strong>
                  <span>{transactionValue} ETH</span>
                </div>
              )}
              <div className="detail-item" style={{borderTop: '1px solid var(--border-color)', paddingTop: '12px', marginTop: '8px'}}>
                <strong style={{color: 'var(--accent-green)'}}>Total Cost:</strong>
                <span style={{color: 'var(--accent-green)', fontWeight: 'bold'}}>{totalCost} ETH</span>
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