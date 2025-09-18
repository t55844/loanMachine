import { useState } from "react";

export default function WalletConnect({ setAccount }) {
  const [hash, setHash] = useState("");

  function handleSubmit(e) {
    e.preventDefault();
    if (hash.trim()) {
      setAccount(hash.trim());
    }
  }

  return (
    <form onSubmit={handleSubmit} className="wallet-connect">
      <input
        type="text"
        placeholder="Enter wallet hash"
        value={hash}
        onChange={(e) => setHash(e.target.value)}
        className="wallet-input"
      />
      <button type="submit" className="wallet-button">
        Set Wallet
      </button>
    </form>
  );
}
