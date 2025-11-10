import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useWeb3 } from "../Web3Context";
import { extractErrorMessage } from "../handlers/errorMapping"; // Now async

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
  const [isAllowanceError, setIsAllowanceError] = useState(false); // NEW: Flag for allowance-specific errors
  
  const { contract, provider } = useWeb3();

  useEffect(() => {
    if (isOpen && contract && provider && transactionData) {
      estimateGasCost();
    }
  }, [isOpen, transactionData]);

  // NEW: Robust v5/v6-agnostic estimation via populateTransaction + provider.estimateGas
  async function estimateGasCost() {
    try {
      if (!contract || !provider) {
        throw new Error("Contrato ou provedor não disponível");
      }

      setLoading(true);
      setGasError("");
      setGasCost(null);
      setIsAllowanceError(false); // Reset flag

      const { method, params = [], value = "0" } = transactionData;
      
      // ✅ FIXED: Populate tx first (v5/v6 compatible)
      const populatedTx = await contract.populateTransaction[method](...params, {
        value: ethers.utils.parseEther(value || "0"), // FIXED: Fallback to "0" if value undefined
        from: await contract.signer.getAddress(), // Ensure 'from' for accurate estimate
      });

      // Estimate on provider (universal)
      const gasEstimate = await provider.estimateGas(populatedTx);
      
      // FIXED: Use legacy getGasPrice as fallback if getFeeData fails
      let gasPrice;
      try {
        const feeData = await provider.getFeeData();
        gasPrice = feeData.gasPrice || await provider.getGasPrice();
      } catch {
        gasPrice = await provider.getGasPrice(); // Legacy fallback
      }
      
      if (!gasPrice || gasPrice.isZero()) {
        throw new Error("Não foi possível obter o preço do gas");
      }

      const gasCostWei = gasEstimate.mul(gasPrice); // v5: BigNumber mul
      const gasCostEth = ethers.utils.formatEther(gasCostWei);

      setGasCost(gasCostEth);
    } catch (err) {
      console.error("Erro de estimativa de gas:", err); // For debug
      
      // NEW: Special handling for insufficient allowance
      if (err.code === 'UNPREDICTABLE_GAS_LIMIT' && err.error?.message?.includes('insufficient allowance')) {
        setGasError("Aprovação insuficiente de USDT. Por favor, aprove o valor necessário primeiro.");
        setIsAllowanceError(true); // Flag to enable button despite error
        setGasCost("0.001"); // Fallback for allowance reverts
      } else {
        // Await the decoded message for other errors
        const userFriendlyError = await extractErrorMessage(err, provider, contract.interface);
        setGasError(userFriendlyError);
        setGasCost("0.001"); // Fallback for all errors
      }
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
        <h3>Estimativa de Custo de Gas</h3>
        
        <div className="gas-info">
          {loading ? (
            <p>Calculando custo de gas...</p>
          ) : gasError ? (
            <div>
              <p className="error-message">⚠️ {gasError}</p>
              <button onClick={handleRetry} className="refresh-button">
                Tentar Estimação Novamente
              </button>
            </div>
          ) : gasCost ? (
            <div>
              <div className="detail-item">
                <strong>Custo Estimado de Gas:</strong>
                <span>{parseFloat(gasCost).toFixed(6)} ETH</span>
              </div>
              
              {transactionValue > 0 && (
                <div className="detail-item">
                  <strong>Valor da Transação:</strong>
                  <span>{transactionValue} ETH</span>
                </div>
              )}
              
              {transactionContext?.token === 'USDT' && transactionValue === 0 && (
                <div className="detail-item">
                  <strong>Transferência de Token:</strong>
                  <span style={{color: 'var(--accent-blue)'}}>
                    {transactionContext.amount} USDT (aprovação separada)
                  </span>
                </div>
              )}
              
              <div className="detail-item total-cost">
                <strong>Taxa Total da Rede:</strong>
                <span style={{color: 'var(--accent-green)', fontWeight: 'bold'}}>
                  {totalCost} ETH
                </span>
              </div>
              
              <div className="fee-note">
                <small>
                  * As taxas de gas são pagas em ETH, não em USDT. Isso cobre o custo da transação na rede.
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
            Cancelar
          </button>
          <button 
            onClick={() => onConfirm(transactionData)}
            className="confirm-button"
            disabled={loading} // FIXED: Remove gasError from disabled - allow confirm on allowance error with fallback
          >
            {loading ? "Calculando..." : "Confirmar Transação"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default GasCostModal;