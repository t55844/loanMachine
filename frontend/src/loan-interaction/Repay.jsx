// Repay.jsx
import { useState } from "react";
import { ethers } from "ethers";
import { useGasCostModal } from "../handlers/useGasCostModal";

function Repay({ account, contract }) {
  const [amount, setAmount] = useState("");
  const { showTransactionModal, ModalWrapper } = useGasCostModal();

  async function handleRepay() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    const value = ethers.utils.parseEther(amount);
    
    showTransactionModal(
      {
        method: "repay",
        params: [],
        value: value.toString()
      },
      {
        type: 'repay',
        amount: amount
      }
    );
  }

  async function confirmTransaction(transactionData) {
    try {
      const value = ethers.BigNumber.from(transactionData.value);
      const tx = await contract.repay({
        value: value
      });
      await tx.wait();
      
      alert(`Repayment of ${amount} ETH successful!`);
      setAmount("");
    } catch (err) {
      console.error(err);
      alert("Error processing repayment");
      throw err;
    }
  }

  return (
    <div className="repay-block">
      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Amount in ETH"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="repay-input"
      />

      <button onClick={handleRepay} className="repay-button" disabled={!account || !amount}>
        Repay
      </button>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default Repay;