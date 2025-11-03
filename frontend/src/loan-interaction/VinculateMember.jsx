import { useState } from "react";
import { useGasCostModal } from "../handlers/useGasCostModal";
import { useWeb3 } from "../Web3Context";

function VinculateMember() {
  const [memberId, setMemberId] = useState("");
  const [loading, setLoading] = useState(false);

  const { showTransactionModal, ModalWrapper } = useGasCostModal();
  const { account, contract, member, refreshMemberData } = useWeb3();

  const memberData = member?.hasVinculation ? member : null;

  async function handleVinculate() {
    if (!account || !contract || !memberId) return alert("Please connect wallet and enter a Member ID");
    
    const id = parseInt(memberId);
    if (isNaN(id) || id <= 0) return alert("Please enter a valid positive Member ID");

    showTransactionModal(
      { method: "vinculationMemberToWallet", params: [id, account], value: "0" },
      { type: 'vinculateMember', memberId: id, wallet: account }
    );
  }

  async function confirmTransaction(transactionData) {
    setLoading(true);
    try {
      const memberId = transactionData.params[0];
      const tx = await contract.vinculationMemberToWallet(memberId, account);
      await tx.wait();
      alert(`Member ID ${memberId} vinculated to your wallet!`);
      setMemberId("");
      // Simply refresh the member data
      await refreshMemberData();
    } catch (err) {
      alert("Error vinculating member to wallet");
      throw err;
    } finally {
      setLoading(false);
    }
  }

  const canVinculate = account && memberId && parseInt(memberId) > 0 && !memberData;

  if (memberData) {
    return (
      <div className="vinculate-member-block">
        <h2>Wallet Vinculation Status</h2>
        
        <div className="user-address">
          <strong>Current Wallet:</strong> {account ? `${account.substring(0, 6)}...${account.substring(38)}` : "Not connected"}
        </div>

        <div className="vinculate-section">
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px', marginBottom: '15px' }}>
            <div style={{ color: 'var(--accent-green)' }}>✅</div>
            <strong style={{ color: 'var(--accent-green)' }}>Wallet Already Vinculated</strong>
          </div>
          
          {/* Member Information */}
          <div style={{ 
            background: 'var(--bg-primary)', 
            padding: '15px', 
            borderRadius: '8px',
            border: '1px solid var(--border-color)',
            marginBottom: '15px'
          }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
              <span><strong>Member ID:</strong></span>
              <span style={{ fontWeight: 'bold', color: 'var(--accent-blue)' }}>#{memberData.memberId}</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
              <span><strong>Reputation Score:</strong></span>
              <span style={{ color: 'var(--accent-green)', fontWeight: 'bold' }}>
                {memberData.currentReputation} points
              </span>
            </div>
          </div>

          {/* Wallets List */}
          <div style={{ marginBottom: '15px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
              <strong>Vinculated Wallets:</strong>
              <span style={{ color: 'var(--text-secondary)' }}>({memberData.wallets ? memberData.wallets.length : 0})</span>
            </div>
            <div style={{ 
              background: 'var(--bg-primary)', 
              padding: '10px', 
              borderRadius: '6px',
              border: '1px solid var(--border-color)',
              maxHeight: '120px',
              overflowY: 'auto'
            }}>
              {memberData.wallets && memberData.wallets.length > 0 
                ? memberData.wallets.map((wallet, index) => (
                    <div key={index} style={{ 
                      padding: '4px 0',
                      fontFamily: 'monospace', 
                      fontSize: '12px',
                      color: wallet.toLowerCase() === account.toLowerCase() ? 'var(--accent-green)' : 'var(--text-primary)',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px'
                    }}>
                      <span>
                        {wallet.toLowerCase() === account.toLowerCase() ? '✓ ' : '• '}
                      </span>
                      <span>
                        {wallet.toLowerCase() === account.toLowerCase() 
                          ? `${wallet.substring(0, 8)}...${wallet.substring(36)} (Current)` 
                          : `${wallet.substring(0, 8)}...${wallet.substring(36)}`
                        }
                      </span>
                    </div>
                  ))
                : <div style={{ color: 'var(--text-secondary)', textAlign: 'center' }}>No wallets vinculated</div>
              }
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="vinculate-member-block">
      <h2>Vinculate Member to Wallet</h2>
      
      <div className="user-address">
        <strong>Current Wallet:</strong> {account ? `${account.substring(0, 6)}...${account.substring(38)}` : "Not connected"}
      </div>

      <div className="vinculate-section">
        <h3>Enter Member ID</h3>
        <div className="wallet-input-row">
          <input
            type="number"
            min="1"
            step="1"
            placeholder="Enter Member ID"
            value={memberId}
            onChange={(e) => setMemberId(e.target.value)}
            className="member-id-input"
            disabled={!account}
            style={{
              flex: 1,
              padding: '10px',
              border: '1px solid var(--border-color)',
              borderRadius: '6px',
              background: 'var(--bg-primary)',
              color: 'var(--text-primary)'
            }}
          />
          <button 
            onClick={handleVinculate} 
            className="vinculate-button" 
            disabled={!canVinculate || loading}
            style={{
              padding: '10px 20px',
              background: !canVinculate ? 'var(--bg-secondary)' : 'var(--accent-green)',
              color: 'white',
              border: 'none',
              borderRadius: '6px',
              cursor: !canVinculate ? 'not-allowed' : 'pointer',
              marginLeft: '10px'
            }}
          >
            {loading ? "Processing..." : "Vinculate Member"}
          </button>
        </div>
        <p className="vinculation-info" style={{ marginTop: '10px', color: 'var(--text-secondary)' }}>
          This will link Member ID {memberId || '___'} to your wallet
        </p>
      </div>

      <ModalWrapper onConfirm={confirmTransaction} />
    </div>
  );
}

export default VinculateMember;