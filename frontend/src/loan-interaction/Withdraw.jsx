import { useState, useEffect } from "react";
import { ethers } from "ethers";
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
      console.error("❌ Error fetching balances:", err);
      setUsdtBalance("0");
      setWithdrawableBalance("0");
      showError("Failed to load balances");
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
      console.error("Error fetching withdrawable balance:", err);
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
      const amountInWei = ethers.BigNumber.from(transactionData.params[0]);
      const memberId = transactionData.params[1];
      
      const tx = await contract.withdraw(amountInWei, memberId);
      const receipt = await tx.wait();
      
      if (receipt.status === 1) {
        showSuccess(`Successfully withdrawn ${amount} USDT!`);
        setAmount("");
        fetchBalances(); // Refresh balances after withdrawal
      } else {
        throw new Error("Transaction failed");
      }
    } catch (err) {
      showError(err.message || "Withdrawal failed");
      throw err;
    }
  }

  async function handleWithdraw() {
    if (!account || !amount) {
      showWarning("Please connect wallet and enter amount");
      return;
    }

    if (!member || !member.id) {
      showError("Member data not available. Please check your wallet connection.");
      return;
    }

    // Check withdrawable balance
    if (parseFloat(amount) > parseFloat(withdrawableBalance)) {
      showError(`Insufficient withdrawable balance. You can withdraw up to ${parseFloat(withdrawableBalance).toFixed(2)} USDT`);
      return;
    }

    // Check if amount is positive
    if (parseFloat(amount) <= 0) {
      showError("Please enter a valid amount");
      return;
    }

    const amountInWei = ethers.utils.parseUnits(amount, 6);
    const memberId = member.id;
    
    // Show the gas cost modal with the confirmation function
    showTransactionModal(
      {
        method: "withdraw",
        params: [amountInWei, memberId],
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
        <p>Your USDT Balance: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        <p className="withdrawable-info">
          Withdrawable Balance: <strong>{parseFloat(withdrawableBalance).toFixed(2)} USDT</strong>
        </p>
        {loading && <p>Loading balances...</p>}
        {member && (
          <p className="member-info">
            Member ID: {member.id} {member.name && `- ${member.name}`}
          </p>
        )}
        {!member && account && (
          <p className="warning-text">⚠️ Member data not loaded</p>
        )}
      </div>

      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Amount in USDT to withdraw"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="donate-input withdraw-input"
      />

      {!hasSufficientWithdrawable && amount && (
        <p className="error-text">
          ❌ You can only withdraw up to {parseFloat(withdrawableBalance).toFixed(2)} USDT
        </p>
      )}

      <button 
        onClick={handleWithdraw} 
        className="donate-button withdraw-button" 
        disabled={!canWithdraw}
      >
        {!hasMemberData ? "Wallet not vinculated" : "Withdraw USDT"}
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Withdraw;