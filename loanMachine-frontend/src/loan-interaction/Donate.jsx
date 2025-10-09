// Donate.jsx
import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";
import { eventSystem } from "../handlers/EventSystem";

function Donate() {
  const [amount, setAmount] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  const [needsApproval, setNeedsApproval] = useState(false);
  const [approving, setApproving] = useState(false);
  const [loading, setLoading] = useState(false);
  
  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { 
    account, 
    contract, 
    getUSDTBalance, 
    approveUSDT,
    needsUSDTApproval 
  } = useWeb3();

  // Fetch USDT balance - FIXED
  useEffect(() => {
    if (account) {
      fetchUSDTBalance();
    } else {
      setUsdtBalance("0"); // Reset when no account
    }
  }, [account]);

  async function fetchUSDTBalance() {
    if (!account) return;
    
    setLoading(true);
    try {
      const balance = await getUSDTBalance();
      setUsdtBalance(balance || "0"); // Ensure it's never undefined
    } catch (err) {
      console.error("❌ Error fetching USDT balance:", err);
      setUsdtBalance("0"); // Set to 0 on error
      showError("Failed to load USDT balance");
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
      console.error("Error checking approval:", err);
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

  async function handleApprove() {
    if (!amount) {
      showWarning("Please enter an amount first");
      return;
    }

    setApproving(true);
    try {
      const tx = await approveUSDT(amount);
      await tx.wait();
      
      showSuccess(`Approved ${amount} USDT successfully!`);
      setNeedsApproval(false);
      
    } catch (err) {
      showError(err.message || "Approval failed");
    } finally {
      setApproving(false);
    }
  }

  async function handleDonate() {
    if (!account || !amount) {
      showWarning("Please connect wallet and enter amount");
      return;
    }

    // Check balance
    if (parseFloat(usdtBalance) < parseFloat(amount)) {
      showError(`Insufficient USDT balance. You have ${parseFloat(usdtBalance).toFixed(2)} USDT`);
      return;
    }

    // Final approval check
    try {
      const currentApprovalNeeded = await needsUSDTApproval(amount);
      if (currentApprovalNeeded) {
        showWarning("Please approve USDT first");
        setNeedsApproval(true);
        return;
      }
    } catch (err) {
      showError("Error checking approval status");
      return;
    }

    const amountInWei = ethers.utils.parseUnits(amount, 6);
    
    showTransactionModal(
      {
        method: "donate",
        params: [amountInWei.toString()],
        value: "0"
      },
      {
        type: 'donate',
        amount: amount,
        token: 'USDT',
        from: account
      }
    );
  }

  async function confirmTransaction(transactionData) {
    try {
      const amountInWei = ethers.BigNumber.from(transactionData.params[0]);
      const tx = await contract.donate(amountInWei);
      const receipt = await tx.wait();
      
      if (receipt.status === 1) {
        showSuccess(`Successfully donated ${amount} USDT!`);
        setAmount("");
        fetchUSDTBalance();
      } else {
        throw new Error("Transaction failed");
      }
    } catch (err) {
      showError(err.message || "Donation failed");
      throw err;
    }
  }

  const hasSufficientBalance = parseFloat(usdtBalance) >= parseFloat(amount);
  const canDonate = account && amount && hasSufficientBalance && !needsApproval;

  return (
    <div className="donate-block">
      <div className="balance-info">
        <p>Your USDT Balance: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        {loading && <p>Loading balance...</p>}
      </div>

      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Amount in USDT"
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
          {approving ? "Approving..." : `Approve ${amount} USDT`}
        </button>
      )}

      <button 
        onClick={handleDonate} 
        className="donate-button" 
        disabled={!canDonate}
      >
        Donate USDT
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Donate;