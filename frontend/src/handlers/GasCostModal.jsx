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
  
  const { contract, provider } = useWeb3();

  useEffect(() => {
    if (isOpen && contract && transactionData) {
      estimateGasCost();
    }
  }, [isOpen, transactionData]);

  // Updated: async, await extractErrorMessage
  async function estimateGasCost() {
    try {
      setLoading(true);
      setGasError("");
      setGasCost(null);

      const { method, params = [], value = "0" } = transactionData;
      
      if (!contract[method]) {
        throw new Error(`Método ${method} não encontrado no contrato`);
      }

      const gasEstimate = await contract.estimateGas[method](...params, {
        value: ethers.parseEther(value) // v6: parseEther
      });

      const gasPrice = await provider.getGasPrice();
      const gasCostWei = gasEstimate * gasPrice;
      const gasCostEth = ethers.formatEther(gasCostWei);

      setGasCost(gasCostEth);
    } catch (err) {
      console.error("Erro de estimativa de gas:", err);
      
      // NEW: Await the decoded message
      const userFriendlyError = await extractErrorMessage(err, provider, contract.interface);
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
    parseFloat(ethers.formatEther(transactionData.value)) : 0;
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
            disabled={loading || gasError}
          >
            {loading ? "Calculando..." : "Confirmar Transação"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default GasCostModal;