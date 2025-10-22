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
  transactionContext
}) {
  const [gasCost, setGasCost] = useState(null);
  const [loading, setLoading] = useState(false);
  const [gasError, setGasError] = useState("");
  
  const { contract, provider } = useWeb3();

  useEffect(() => {
    if (isOpen && contract && transactionData) {
      estimateGasCost();
    }
  }, [isOpen, transactionData]);

  // GasCostModal.jsx - Update the estimateGasCost function
async function estimateGasCost() {
  try {
    setLoading(true);
    setGasError("");
    setGasCost(null);

    const { method, params = [], value = "0" } = transactionData;
    
    if (!contract[method]) {
      throw new Error(`Method ${method} not found on contract`);
    }

    console.log(`Estimating gas for ${method} with params:`, params);
    console.log("Transaction context:", transactionContext);

    // Try to estimate gas
    const gasEstimate = await contract.estimateGas[method](...params, {
      value: ethers.BigNumber.from(value)
    });

    const gasPrice = await provider.getGasPrice();
    const gasCostWei = gasEstimate.mul(gasPrice);
    const gasCostEth = ethers.utils.formatEther(gasCostWei);

    setGasCost(gasCostEth);
  } catch (err) {
    console.error("Gas estimation error:", err);
    
    // Use extractErrorMessage but don't emit toast here
    let userFriendlyError = extractErrorMessage(err);
    
    // If it's a gas estimation error with underlying contract error, 
    // show the contract error, not the gas estimation wrapper
    if (err.code === 'UNPREDICTABLE_GAS_LIMIT') {
      // The extractErrorMessage should have already extracted the underlying error
      console.log('Showing contract error instead of gas estimation error');
    }
    
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

  return (
    <div className="confirmation-modal-overlay">
      <div className="confirmation-modal">
        <h3>Gas Cost Estimation</h3>
        
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
      </div>
    </div>
  );
}

export default GasCostModal;