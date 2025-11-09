import { useState, useEffect } from "react";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";
import { eventSystem } from "../handlers/EventSystem";

function Withdraw() {
  const [amount, setAmount] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  const [withdrawableBalance, setWithdrawableBalance] = useState("0");
  const [loading, setLoading] = useState(false);
  
  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { 
    account, 
    contract, 
    getUSDTBalance,
    member,
    provider
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
      //console.error("❌ Erro ao buscar saldos:", err);
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
      return ethers.utils.formatUnits(balance, 6); // USDT has 6 decimals
    } catch (err) {
      //console.error("Erro ao buscar saldo sacável:", err);
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

  async function confirmTransaction(transactionData) {
    try {
      const amountInWei = ethers.utils.parseUnits(transactionData.params[0], 6); // FIXED: Use utils.parseUnits
      const memberId = transactionData.params[1];
      
      const tx = await contract.withdraw(amountInWei, memberId);
      const receipt = await tx.wait();
      
      if (receipt.status === 1) {
        showSuccess(`Saque de ${amount} USDT bem-sucedido!`);
        setAmount("");
        fetchBalances(); // Refresh balances after withdrawal
      } else {
        throw new Error("Transação falhou");
      }
    } catch (err) {
      showError(err.message || "Falha no saque");
      throw err;
    }
  }

  async function handleWithdraw() {
    if (!account || !amount) {
      showWarning("Por favor, conecte a carteira e insira o valor");
      return;
    }

    if (!member || !member.id) {
      showError("Dados do membro não disponíveis. Por favor, verifique sua conexão com a carteira.");
      return;
    }

    // Check withdrawable balance
    if (parseFloat(amount) > parseFloat(withdrawableBalance)) {
      showError(`Saldo sacável insuficiente. Você pode sacar até ${parseFloat(withdrawableBalance).toFixed(2)} USDT`);
      return;
    }

    // Check if amount is positive
    if (parseFloat(amount) <= 0) {
      showError("Por favor, insira um valor válido");
      return;
    }

    const amountInWei = ethers.utils.parseUnits(amount, 6); // FIXED: Use utils.parseUnits
    const memberId = member.id;
    
    // Show the gas cost modal with the confirmation function
    showTransactionModal(
      {
        method: "withdraw",
        params: [amountInWei.toString(), memberId], // UPDATED: String for params if needed
        value: "0"
      },
      {
        type: 'withdraw',
        amount: amount,
        token: 'USDT',
        from: account,
        memberId: memberId
      }
    );
  }

  const hasSufficientWithdrawable = parseFloat(amount) <= parseFloat(withdrawableBalance);
  const hasMemberData = member && member.id;
  const canWithdraw = account && amount && hasSufficientWithdrawable && hasMemberData && parseFloat(amount) > 0;

  return (
    <div className="donate-block withdraw-block">
      <div className="balance-info">
        <p>Seu Saldo USDT: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        <p className="withdrawable-info">
          Saldo Disponível para Saque: <strong>{parseFloat(withdrawableBalance).toFixed(2)} USDT</strong>
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
      />

      {!hasSufficientWithdrawable && amount && (
        <p className="error-text">
          ❌ Você só pode sacar até {parseFloat(withdrawableBalance).toFixed(2)} USDT
        </p>
      )}

      <button 
        onClick={handleWithdraw} 
        className="donate-button withdraw-button" 
        disabled={!canWithdraw}
      >
        {!hasMemberData ? "Carteira não vinculada" : "Sacar USDT"}
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Withdraw;