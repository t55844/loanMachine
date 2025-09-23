// Borrow.jsx
import { useState } from "react";
import { ethers } from "ethers";

function Borrow({ account, contract }) {
  const [amount, setAmount] = useState("");

  async function handleBorrow() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    try {
      const tx = await contract.borrow(ethers.utils.parseEther(amount));
      await tx.wait();
      alert(`Loan of ${amount} ETH borrowed by ${account}!`);
      setAmount("");
    } catch (err) {
      console.error(err);
      alert("Error borrowing");
    }
  }

  return (
    <div className="borrow-block">
      <input
        type="number"
        min={0}
        step="0.01"
        placeholder="Amount in ETH"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        className="borrow-input"
      />

      <button onClick={handleBorrow} className="borrow-button" disabled={!account}>
        Borrow
      </button>
    </div>
  );
}

export default Borrow;