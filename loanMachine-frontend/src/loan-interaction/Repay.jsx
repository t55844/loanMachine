// Repay.jsx
import { useState } from "react";
import { ethers } from "ethers";

function Repay({ account, contract }) {
  const [amount, setAmount] = useState("");

  async function handleRepay() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    try {
      const tx = await contract.repay({
        value: ethers.utils.parseEther(amount)
      });
      await tx.wait();
      alert(`Repayment of ${amount} ETH sent from ${account}!`);
      setAmount("");
    } catch (err) {
      console.error(err);
      alert("Error repaying loan");
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

      <button onClick={handleRepay} className="repay-button" disabled={!account}>
        Repay
      </button>
    </div>
  );
}

export default Repay;