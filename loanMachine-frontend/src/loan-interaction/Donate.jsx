// Donate.jsx
import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";

function Donate() {
  const [amount, setAmount] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  const [allowance, setAllowance] = useState("0");
  const [maxDonation, setMaxDonation] = useState("5");
  
  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { 
    account, 
    contract, 
    getUSDTBalance, 
    getUSDTAllowance,
    approveUSDT,
    needsUSDTApproval 
  } = useWeb3();

  // Fetch USDT data when account changes
  useEffect(() => {
    if (account && contract) {
      fetchUSDTData();
    }
  }, [account, contract]);

  async function fetchUSDTData() {
    try {
      const [balance, currentAllowance, remainingAllowance] = await Promise.all([
        getUSDTBalance(),
        getUSDTAllowance(),
        contract.getRemainingDonationAllowance(account)
      ]);
      
      setUsdtBalance(balance);
      setAllowance(currentAllowance);
      setMaxDonation(ethers.utils.formatUnits(remainingAllowance, 6));
    } catch (err) {
      console.error("Error fetching USDT data:", err);
    }
  }

  async function handleApprove() {
    try {
      // Approve a large amount for multiple donations
      await approveUSDT(amount); // Approve 100 USDT
      alert("USDT approved successfully!");
      fetchUSDTData(); // Refresh data
    } catch (err) {
      console.error("Error approving USDT:", err);
      alert("Error approving USDT");
    }
  }

  async function handleDonate() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    // Check if approval is needed
    const needsApproval = await needsUSDTApproval(amount);
    if (needsApproval) {
      alert("Please approve USDT first. Click the 'Approve USDT' button.");
      return;
    }

    // Check balance
    if (parseFloat(usdtBalance) < parseFloat(amount)) {
      alert("Insufficient USDT balance");
      return;
    }

    // Check donation limit
    if (parseFloat(amount) > parseFloat(maxDonation)) {
      alert(`Donation exceeds maximum limit. You can only donate ${maxDonation} more USDT.`);
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
      fetchUSDTData(); // Refresh balances
    } catch (err) {
      console.error(err);
      alert("Error sending donation");
      throw err;
    }
  }

  const needsApproval = parseFloat(allowance) < parseFloat(amount);
  const hasSufficientBalance = parseFloat(usdtBalance) >= parseFloat(amount);
  const withinDonationLimit = parseFloat(amount) <= parseFloat(maxDonation);

  return (
    <div className="donate-block">
      <div className="balance-info">
        <p>Your USDT Balance: {parseFloat(usdtBalance).toFixed(2)} USDT</p>
        <p>Approved: {parseFloat(allowance).toFixed(2)} USDT</p>
        <p>Remaining Donation Allowance: {parseFloat(maxDonation).toFixed(2)} USDT</p>
      </div>

      <input
        type="number"
        min={0}
        max={5}
        step="0.01"
        placeholder="Amount in USDT (max 5)"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="donate-input"
      />

      {needsApproval && (
        <button onClick={handleApprove} className="approve-button">
          Approve USDT First
        </button>
      )}

      <button 
        onClick={handleDonate} 
        className="donate-button" 
        disabled={!account || !amount || needsApproval || !hasSufficientBalance || !withinDonationLimit}
      >
        Donate USDT
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Donate;