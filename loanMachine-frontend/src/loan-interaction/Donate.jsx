// Donate.jsx
import { useState } from "react";
import { ethers } from "ethers";
import { useGasCostModal } from "../handlers/useGasCostModal";

function Donate({ account, contract }) {
  const [amount, setAmount] = useState("");
  const { showTransactionModal, ModalWrapper } = useGasCostModal();

  async function handleDonate() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    const value = ethers.utils.parseEther(amount);
    
    showTransactionModal({
      method: "donate",
      params: [],
      value: value.toString()
    });
  }

  async function confirmTransaction(transactionData) {
    try {
      const tx = await contract.donate({
        value: ethers.BigNumber.from(transactionData.value)
      });
      
      await tx.wait();
      alert(`Donation of ${amount} ETH sent from ${account} to the contract!`);
      setAmount("");
    } catch (err) {
      console.error(err);
      alert("Error sending donation");
    }
  }

  return (
    <div className="donate-block">
      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Amount in ETH"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="donate-input"
      />

      <button 
        onClick={handleDonate} 
        className="donate-button" 
        disabled={!account}
      >
        Donate
      </button>

      <ModalWrapper 
        account={account} 
        contract={contract} 
        onConfirm={confirmTransaction} 
      />
    </div>
  );
}

export default Donate;