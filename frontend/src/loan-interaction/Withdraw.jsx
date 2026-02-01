import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useWeb3 } from "../Web3Context";
import { eventSystem } from "../handlers/EventSystem";

function Withdraw() {
  const [amount, setAmount] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  const [withdrawableBalance, setWithdrawableBalance] = useState("0");
  const [loading, setLoading] = useState(false);
  const [withdrawing, setWithdrawing] = useState(false);
  
  const { 
    account, 
    contract, 
    getUSDTBalance,
    member
  } = useWeb3();

  // Fetch USDT balance and withdrawable balance
  useEffect(() => {
    if (account) {
      fetchBalances();
    } else {
      setUsdtBalance("0");
      setWithdrawableBalance("0");
    }
  }, [account]);

  async function fetchBalances() {
    if (!account || !contract) return;
    
    setLoading(true);
    try {
      const [balance, withdrawable] = await Promise.all([
        getUSDTBalance(),
        getWithdrawableBalance()
      ]);
      
      setUsdtBalance(balance || "0");
      setWithdrawableBalance(withdrawable || "0");
    } catch (err) {
      setUsdtBalance("0");
      setWithdrawableBalance("0");
      showError("Falha ao carregar saldos");
    } finally {
      setLoading(false);
    }
  }

  async function getWithdrawableBalance() {
    if (!contract || !account) return "0";
    
    try {
      const balance = await contract.getWithdrawableBalance(account);
      const formatted = ethers.utils.formatUnits(balance, 6);
      return formatted;
    } catch (err) {
      return "0";
    }
  }

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

  // ✅ FIXED: Safe display formatting
  const formatDisplayAmount = (amount) => {
    const num = Number(amount);
    if (isNaN(num)) return "0.00";
    return num.toFixed(2);
  };

  // ✅ FIXED: Safe parse for user input
  const parseUserAmount = (input) => {
    const num = Number(input);
    return isNaN(num) ? 0 : num;
  };

  async function handleWithdraw() {
    if (!account || !amount) {
      showWarning("Por favor, conecte a carteira e insira o valor");
      return;
    }

    if (!member || !member.id) {
      showError("Dados do membro não disponíveis. Por favor, verifique sua conexão com a carteira.");
      return;
    }

    const userAmount = parseUserAmount(amount);
    const availableAmount = Number(withdrawableBalance);

    // Check withdrawable balance
    if (userAmount > availableAmount) {
      showError(`Saldo sacável insuficiente. Você pode sacar até ${formatDisplayAmount(withdrawableBalance)} USDT`);
      return;
    }

    // Check if amount is positive
    if (userAmount <= 0) {
      showError("Por favor, insira um valor válido");
      return;
    }

    const amountInWei = ethers.utils.parseUnits(amount, 6);
    const memberId = member.id;

    // ✅ FIXED: Direct transaction - no gas estimation modal
    setWithdrawing(true);
    
    eventSystem.emit('showToast', {
      message: "Iniciando saque...",
      type: 'info',
      duration: 3000
    });

    try {
      const tx = await contract.withdraw(amountInWei, memberId);
      
      eventSystem.emit('showToast', {
        message: "Transação de saque enviada. Aguardando confirmação...",
        type: 'info',
        duration: 5000
      });

      const receipt = await tx.wait();
      
      if (receipt.status === 1) {
        showSuccess(`Saque de ${amount} USDT bem-sucedido!`);
        setAmount("");
        fetchBalances(); // Refresh balances after withdrawal
      } else {
        throw new Error("Transação falhou");
      }
    } catch (err) {
      // Handle specific error cases
      if (err.message.includes("user rejected transaction")) {
        showError("Transação rejeitada pelo usuário");
      } else if (err.message.includes("execution reverted")) {
        showError("Transação revertida. Verifique se você tem saldo suficiente e permissões adequadas.");
      } else {
        showError(err.message || "Falha no saque");
      }
    } finally {
      setWithdrawing(false);
    }
  }

  // ✅ FIXED: Use safe comparison and parsing
  const userAmount = parseUserAmount(amount);
  const availableAmount = Number(withdrawableBalance);
  const hasSufficientWithdrawable = userAmount <= availableAmount;
  const hasMemberData = member && member.id;
  const canWithdraw = account && amount && hasSufficientWithdrawable && hasMemberData && userAmount > 0 && !withdrawing;

  return (
    <div className="donate-block withdraw-block">
      <div className="balance-info">
        <p>Seu Saldo USDT: {formatDisplayAmount(usdtBalance)} USDT</p>
        <p className="withdrawable-info">
          Saldo Disponível para Saque: <strong>{formatDisplayAmount(withdrawableBalance)} USDT</strong>
        </p>
        {loading && <p>Carregando saldos...</p>}
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
        min={0}
        step="0.01"
        placeholder="Quantidade em USDT para sacar"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="donate-input withdraw-input"
        disabled={withdrawing}
      />

      {!hasSufficientWithdrawable && amount && (
        <p className="error-text">
          ❌ Você só pode sacar até {formatDisplayAmount(withdrawableBalance)} USDT
        </p>
      )}

      <button 
        onClick={handleWithdraw} 
        className="donate-button withdraw-button" 
        disabled={!canWithdraw}
      >
        {withdrawing ? "Processando Saque..." : 
         !hasMemberData ? "Carteira não vinculada" : "Sacar USDT"}
      </button>

      {/* ✅ FIXED: Removed ModalWrapper - no modal for Withdraw */}
    </div>
  );
}

export default Withdraw;