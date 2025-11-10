import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useWeb3 } from "../Web3Context";
import { eventSystem } from "../handlers/EventSystem";

function Donate() {
  const [amount, setAmount] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  const [needsApproval, setNeedsApproval] = useState(false);
  const [approving, setApproving] = useState(false);
  const [loading, setLoading] = useState(false);
  
  const { 
    account, 
    contract, 
    getUSDTBalance, 
    approveUSDT,
    needsUSDTApproval,
    member
  } = useWeb3();

  // Fetch USDT balance
  useEffect(() => {
    if (account) {
      fetchUSDTBalance();
    } else {
      setUsdtBalance("0");
    }
  }, [account]);

  async function fetchUSDTBalance() {
    if (!account) return;
    
    setLoading(true);
    try {
      const balance = await getUSDTBalance();
      setUsdtBalance(balance || "0");
    } catch (err) {
      //console.error("❌ Erro ao buscar saldo USDT:", err);
      setUsdtBalance("0");
      eventSystem.emit('showToast', {
        message: "Falha ao carregar saldo USDT",
        isError: true
      });
    } finally {
      setLoading(false);
    }
  }

  // Check approval when amount changes
  useEffect(() => {
    if (amount && account) {
      checkApproval();
    } else {
      setNeedsApproval(false);
    }
  }, [amount, account]);

  async function checkApproval() {
    if (!amount || !account) return;
    
    try {
      const approvalNeeded = await needsUSDTApproval(amount);
      setNeedsApproval(approvalNeeded);
    } catch (err) {
      //console.error("Erro ao verificar aprovação:", err);
    }
  }

  // Helper functions (unchanged)
  function showError(error) {
    eventSystem.emit('showToast', {
      message: error.message || error,
      isError: true
    });
  }

  function showSuccess(message) {
    eventSystem.emit('showToast', {
      message: message,
      type: 'success'
    });
  }

  function showWarning(message) {
    eventSystem.emit('showToast', {
      message: message,
      type: 'warning'
    });
  }

  async function handleApprove() {
    if (!amount) {
      showWarning("Por favor, insira um valor primeiro");
      return;
    }

    setApproving(true);
    try {
      const tx = await approveUSDT(amount);
      await tx.wait();
      
      showSuccess(`${amount} USDT aprovados com sucesso!`);
      setNeedsApproval(false);
      
    } catch (err) {
      showError(err.message || "Falha na aprovação");
    } finally {
      setApproving(false);
    }
  }

  async function confirmTransaction(amountInWei, memberId) {
    try {
      const tx = await contract.donate(amountInWei, memberId);
      const receipt = await tx.wait();
      
      if (receipt.status === 1) {
        showSuccess(`Doação de ${amount} USDT bem-sucedida!`);
        setAmount("");
        fetchUSDTBalance();
      } else {
        throw new Error("Transação falhou");
      }
    } catch (err) {
      showError(err.message || "Falha na doação");
      throw err;
    }
  }

  async function handleDonate() {
    if (!account || !amount) {
      showWarning("Por favor, conecte a carteira e insira o valor");
      return;
    }

    if (!member || !member.id) {
      showError("Dados do membro não disponíveis. Por favor, verifique sua conexão com a carteira.");
      return;
    }

    // Check balance (use parseFloat for consistency)
    if (parseFloat(usdtBalance) < parseFloat(amount)) {
      showError(`Saldo USDT insuficiente. Você tem ${parseFloat(usdtBalance).toFixed(2)} USDT`);
      return;
    }

    // Final approval check
    try {
      const currentApprovalNeeded = await needsUSDTApproval(amount);
      if (currentApprovalNeeded) {
        showWarning("Por favor, aprove USDT primeiro");
        setNeedsApproval(true);
        return;
      }
    } catch (err) {
      showError("Erro ao verificar status de aprovação");
      return;
    }

    const amountInWei = ethers.utils.parseUnits(amount, 6);
    const memberId = member.id;

    // FIXED: Direct tx—no modal to avoid revert. Show loading toast
    eventSystem.emit('showToast', {
      message: "Iniciando doação...",
      type: 'info',
      duration: 3000
    });

    await confirmTransaction(amountInWei, memberId);
  }

  const hasSufficientBalance = parseFloat(usdtBalance) >= parseFloat(amount);
  const hasMemberData = member && member.id;
  const canDonate = account && amount && hasSufficientBalance && !needsApproval && hasMemberData;

  return (
    <div className="donate-block">
      <div className="balance-info">
        <p>Seu Saldo USDT: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        {loading && <p>Carregando saldo...</p>}
        {member && (
          <p className="member-info">
            ID do Membro: {member.id} {member.name && `- ${member.name}`}
          </p>
        )}
        {!member && account && (
          <p className="warning-text">⚠️ Dados do membro não carregados</p>
        )}
      </div>

      <input
        type="number"
        min="0"
        step="0.01"
        placeholder="Quantidade em USDT"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="donate-input"
      />

      {needsApproval && (
        <button 
          onClick={handleApprove} 
          className="approve-button"
          disabled={approving || !amount}
        >
          {approving ? "Aprovando..." : `Aprovar ${amount} USDT`}
        </button>
      )}

      <button 
        onClick={handleDonate} 
        className="donate-button" 
        disabled={!canDonate}
      >
        {!hasMemberData ? "Talvez a carteira não esteja vinculada" : "Doar USDT"}
      </button>

      {/* FIXED: Removed ModalWrapper—no modal for Donate */}
    </div>
  );
}

export default Donate;