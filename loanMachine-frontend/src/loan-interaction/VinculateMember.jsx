// VinculateMember.jsx
import { useState } from "react";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";

function VinculateMember() {
  const [memberId, setMemberId] = useState("");
  const [loading, setLoading] = useState(false);
  
  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { account, contract } = useWeb3();

  async function handleVinculate() {
    if (!account || !contract || !memberId) {
      alert("Please connect wallet and enter a Member ID");
      return;
    }

    // Validate member ID is a positive integer
    const id = parseInt(memberId);
    if (isNaN(id) || id <= 0) {
      alert("Please enter a valid positive Member ID");
      return;
    }

    showTransactionModal(
      {
        method: "vinculationMemberToWallet",
        params: [id, account], // Using account as the wallet parameter
        value: "0"
      },
      {
        type: 'vinculateMember',
        memberId: id,
        wallet: account
      }
    );
  }

  async function confirmTransaction(transactionData) {
    setLoading(true);
    try {
      const memberId = transactionData.params[0];
      
      const tx = await contract.vinculationMemberToWallet(memberId, account);
      await tx.wait();
      
      alert(`Member ID ${memberId} successfully vinculated to your wallet!`);
      setMemberId("");
    } catch (err) {
      console.error("Error vinculating member:", err);
      alert("Error vinculating member to wallet");
      throw err;
    } finally {
      setLoading(false);
    }
  }

  const canVinculate = account && memberId && parseInt(memberId) > 0;

  return (
    <div className="stats-box">
      <h2>Vinculate Member to Wallet</h2>
      
      <div className="user-address">
        <strong>Current Wallet:</strong> {account ? `${account.substring(0, 6)}...${account.substring(38)}` : "Not connected"}
      </div>

      <div className="donate-section">
        <h3>Enter Member ID</h3>
        <div className="donate-block">
          <input
            type="number"
            min="1"
            step="1"
            placeholder="Enter Member ID"
            value={memberId}
            onChange={(e) => setMemberId(e.target.value)}
            className="donate-input"
            disabled={!account}
          />

          <button 
            onClick={handleVinculate} 
            className="donate-button" 
            disabled={!canVinculate || loading}
          >
            {loading ? "Processing..." : "Vinculate Member"}
          </button>
        </div>
        <p style={{ fontSize: '14px', color: 'var(--text-secondary)', marginTop: '10px' }}>
          This will link Member ID {memberId || '___'} to your currently connected wallet
        </p>
      </div>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default VinculateMember;