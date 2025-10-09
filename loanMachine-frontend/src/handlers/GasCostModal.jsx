// GasCostModal.jsx
import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useWeb3 } from "../Web3Context";
import { extractErrorMessage } from "../handlers/errorMapping";

function GasCostModal({ 
  isOpen, 
  onClose, 
  onConfirm, 
  transactionData,
  transactionContext,
  transactionStatus = 'pending' // 'pending', 'processing', 'success', 'error'
}) {
  const [gasCost, setGasCost] = useState(null);
  const [loading, setLoading] = useState(false);
  const [gasError, setGasError] = useState("");
  
  const { contract, provider } = useWeb3();

  useEffect(() => {
    if (isOpen && contract && transactionData && transactionStatus === 'pending') {
      estimateGasCost();
    }
  }, [isOpen, transactionData, transactionStatus]);

  async function estimateGasCost() {
    try {
      setLoading(true);
      setGasError("");
      setGasCost(null);

      const { method, params = [], value = "0" } = transactionData;
      
      if (!contract[method]) {
        throw new Error(`Method ${method} not found on contract`);
      }

      const gasEstimate = await contract.estimateGas[method](...params, {
        value: ethers.BigNumber.from(value)
      });

      const gasPrice = await provider.getGasPrice();
      const gasCostWei = gasEstimate.mul(gasPrice);
      const gasCostEth = ethers.utils.formatEther(gasCostWei);

      setGasCost(gasCostEth);
    } catch (err) {
      console.error("Error estimating gas:", err);
      const userFriendlyError = extractErrorMessage(err);
      setGasError(userFriendlyError);
    } finally {
      setLoading(false);
    }
  }

  const handleRetry = () => {
    estimateGasCost();
  };

  if (!isOpen) return null;

  const transactionValue = transactionData?.value ? 
    parseFloat(ethers.utils.formatEther(transactionData.value)) : 0;
  const totalCost = (parseFloat(gasCost || 0) + transactionValue).toFixed(6);

  const formatUSDTAmount = (amount) => {
    if (!amount) return '0';
    try {
      return parseFloat(ethers.utils.formatUnits(amount, 6)).toFixed(2);
    } catch {
      return amount;
    }
  };

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

  const renderContent = () => {
    // Show transaction status if processing, success, or error
    if (transactionStatus === 'processing') {
      return (
        <div className="transaction-status processing">
          <h4>Processing Transaction...</h4>
          <p>Your transaction is being processed on the blockchain.</p>
          <div className="loading-spinner">⏳</div>
        </div>
      );
    }

    if (transactionStatus === 'success') {
      return (
        <div className="transaction-status success">
          <h4>✅ Transaction Successful!</h4>
          <p>Your transaction was completed successfully.</p>
          <button onClick={onClose} className="close-button">
            Close
          </button>
        </div>
      );
    }

    if (transactionStatus === 'error') {
      return (
        <div className="transaction-status error">
          <h4>❌ Transaction Failed</h4>
          <p>There was an error processing your transaction.</p>
          <p className="error-note">Check the toast notification for details.</p>
          <div className="error-actions">
            <button onClick={onClose} className="close-button">
              Close
            </button>
            <button onClick={() => onConfirm(transactionData)} className="retry-button">
              Try Again
            </button>
          </div>
        </div>
      );
    }

    // Default: show gas estimation and confirmation
    return (
      <>
        {renderTransactionDetails()}
        
        <div className="gas-info">
          {loading ? (
            <p>Calculating gas cost...</p>
          ) : gasError ? (
            <div>
              <p className="error-message">⚠️ {gasError}</p>
              <button onClick={handleRetry} className="refresh-button">
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
            disabled={loading || gasError}
          >
            {loading ? "Calculating..." : "Confirm Transaction"}
          </button>
        </div>
      </>
    );
  };

  return (
    <div className="confirmation-modal-overlay">
      <div className="confirmation-modal">
        <h3>Transaction Confirmation</h3>
        {renderContent()}
      </div>
    </div>
  );
}

export default GasCostModal;