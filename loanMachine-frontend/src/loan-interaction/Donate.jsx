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
      console.error("❌ Error fetching USDT balance:", err);
      setUsdtBalance("0");
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

  async function confirmTransaction(transactionData) {
    try {
      const amountInWei = ethers.BigNumber.from(transactionData.params[0]);
      const memberId = transactionData.params[1];
      
      const tx = await contract.donate(amountInWei, memberId);
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

  async function handleDonate() {
    if (!account || !amount) {
      showWarning("Please connect wallet and enter amount");
      return;
    }

    if (!member || !member.id) {
      showError("Member data not available. Please check your wallet connection.");
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
    const memberId = member.id;
    
    // Show the gas cost modal with the confirmation function
    showTransactionModal(
      {
        method: "donate",
        params: [amountInWei, memberId],
        value: "0"
      },
      {
        type: 'donate',
        amount: amount,
        token: 'USDT',
        from: account,
        memberId: memberId
      }
    );
  }

  const hasSufficientBalance = parseFloat(usdtBalance) >= parseFloat(amount);
  const hasMemberData = member && member.id;
  const canDonate = account && amount && hasSufficientBalance && !needsApproval && hasMemberData;

  return (
    <div className="donate-block">
      <div className="balance-info">
        <p>Your USDT Balance: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        {loading && <p>Loading balance...</p>}
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
        {!hasMemberData ? "Maybe wallet not vinculated" : "Donate USDT"}
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Donate;