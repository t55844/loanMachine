// Donate.jsx
import { useState } from "react";
import { ethers } from "ethers";

function Donate({ account, contract }) {
  const [amount, setAmount] = useState("");

  async function handleDonate() {
    if (!account || !contract || !amount) {
      alert("Please connect and enter an amount");
      return;
    }

    try {
      const tx = await contract.donate({
        value: ethers.utils.parseEther(amount)
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

      <button onClick={handleDonate} className="donate-button" disabled={!account}>
        Donate
      </button>
    </div>
  );
}

export default Donate;