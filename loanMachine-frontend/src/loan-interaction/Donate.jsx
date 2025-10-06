// Donate.jsx
import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";

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

  // Fetch USDT balance when account changes
  useEffect(() => {
    if (account && contract) {
      fetchUSDTBalance();
    }
  }, [account, contract]);

  async function fetchUSDTBalance() {
    if (!account) return;
    
    setLoading(true);
    try {
      const balance = await getUSDTBalance();
      setUsdtBalance(balance);
    } catch (err) {
      console.error("Error fetching USDT balance:", err);
    } finally {
      setLoading(false);
    }
  }

  // Check approval when amount changes
  useEffect(() => {
    if (amount && account && contract) {
      checkApproval();
    } else {
      setNeedsApproval(false);
    }
  }, [amount]);

  async function checkApproval() {
    try {
      const approvalNeeded = await needsUSDTApproval(amount);
      setNeedsApproval(approvalNeeded);
    } catch (err) {
      console.error("Error checking approval:", err);
    }
  }

  async function handleApprove() {
    if (!amount) {
      alert("Please enter an amount first");
      return;
    }

    setApproving(true);
    try {
      // Approve the specific amount for donation
      await approveUSDT(amount);
      alert("USDT approved successfully!");
      setNeedsApproval(false);
    } catch (err) {
      console.error("Error approving USDT:", err);
      alert("Error approving USDT");
    } finally {
      setApproving(false);
    }
  }

  async function handleDonate() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    // Double-check approval status before proceeding
    const currentApprovalNeeded = await needsUSDTApproval(amount);
    if (currentApprovalNeeded) {
      alert("Please approve USDT first. Click the 'Approve USDT' button.");
      setNeedsApproval(true);
      return;
    }

    // Check balance
    if (parseFloat(usdtBalance) < parseFloat(amount)) {
      alert("Insufficient USDT balance");
      return;
    }

    const amountInWei = ethers.utils.parseUnits(amount, 6);
    
    // Show gas cost modal before proceeding
    showTransactionModal(
      {
        method: "donate",
        params: [amountInWei.toString()],
        value: "0" // No ETH value for USDT transactions
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
      await tx.wait();
      
      alert(`Donation of ${amount} USDT sent from ${account} to the contract!`);
      setAmount("");
      fetchUSDTBalance(); // Refresh balance
    } catch (err) {
      console.error(err);
      alert("Error sending donation");
      throw err;
    }
  }

  const hasSufficientBalance = parseFloat(usdtBalance) >= parseFloat(amount);
  const canDonate = account && amount && hasSufficientBalance && !needsApproval;

  return (
    <div className="donate-block">
      <div className="balance-info">
        <p>Your USDT Balance: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        {loading && <p>Loading...</p>}
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
          {approving ? "Approving..." : `Approve USDT (${amount} USDT)`}
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