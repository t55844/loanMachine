// components/WalletConnection.jsx
import { useState, useEffect } from 'react';
import { useWeb3 } from '../Web3Context'; 
import { ethers } from 'ethers'; 

const WalletConnection = ({ onContinue }) => {
  const {  
    account,  
    provider,  
    loading,  
    error,  
    switchAccount,  
    getUSDTBalance,
    usdtContract,
    connectToLocalNode,
    connectToExternalWallet, 
    disconnect, 
    connectionType 
  } = useWeb3();
  
  const [usdtBalance, setUsdtBalance] = useState('0');
  const [faucetLoading, setFaucetLoading] = useState(false);
  const [faucetError, setFaucetError] = useState('');
  const [faucetSuccess, setFaucetSuccess] = useState(false);
  const [availableAccounts, setAvailableAccounts] = useState([]);
  const [showAccountSelector, setShowAccountSelector] = useState(false);

  // ✅ Load saved connection type
  useEffect(() => {
  const savedType = localStorage.getItem('connectedWalletType');
  const savedAccount = localStorage.getItem('connectedWalletAddress');

  if (!account && savedType) {
    if (savedType === 'local') {
      connectToLocalNode(savedAccount);
    } else if (savedType === 'external') {
      connectToExternalWallet(savedAccount);
    }
  }
}, []);

  // ✅ Update balance and accounts when connected
  useEffect(() => {
    const fetchData = async () => {
      if (account && provider) {
        try {
          if (connectionType === 'local') {
            const accounts = await provider.listAccounts();
            setAvailableAccounts(accounts);
          } else {
            setAvailableAccounts([]);
          }
          const balance = await getUSDTBalance();
          setUsdtBalance(balance);
        } catch (err) {
          console.error('Error fetching data:', err);
        }
      }
    };
    fetchData();
  }, [account, provider, getUSDTBalance, connectionType]); 

  // ✅ Request faucet
  const requestFaucet = async () => {
    if (!usdtContract || !account) {
      setFaucetError('Wallet not connected or USDT contract not found');
      return;
    }

    setFaucetLoading(true);
    setFaucetError('');
    setFaucetSuccess(false);

    try {
      const amount = ethers.utils.parseUnits('1000', 6); 
      const tx = await usdtContract.mint(account, amount);
      await tx.wait();
      
      const newBalance = await getUSDTBalance();
      setUsdtBalance(newBalance);
      
      setFaucetSuccess(true);
      setTimeout(() => setFaucetSuccess(false), 3000);
    } catch (err) {
      console.error('Faucet error:', err);
      setFaucetError(err.data?.message || err.message || 'Failed to get USDT from faucet');
    } finally {
      setFaucetLoading(false);
    }
  };

  // ✅ Handle connect buttons (save to localStorage)
  const handleConnectLocal = async () => {
    await connectToLocalNode();
    localStorage.setItem('connectedWalletType', 'local');
  };

  const handleConnectExternal = async () => {
    await connectToExternalWallet();
    localStorage.setItem('connectedWalletType', 'external');
  };

  // ✅ Handle disconnect (clear localStorage)
  const handleDisconnect = () => {
    disconnect();
    localStorage.removeItem('connectedWalletType');
    window.location.reload(); // force back to initial screen
  };

  const handleAccountSwitch = async (accountIndex) => {
    if (connectionType !== 'local') return; 
    await switchAccount(accountIndex);
    setShowAccountSelector(false);
    
    const balance = await getUSDTBalance();
    setUsdtBalance(balance);
  };

  // INITIAL CONNECTION SCREEN
  if (!account && !loading) {
    return (
      <div className="wallet-connection-block initial-state">
        <h3>🔌 Connect Your Wallet</h3>
        <p>Choose how you want to connect for testing.</p>
        <div className="connection-options" style={{display: 'flex', flexDirection:'row', justifyContent: 'space-around'}}>
          <button 
            onClick={handleConnectExternal} 
            className="faucet-button primary"
          >
             Connect External Wallet
          </button>
          <button 
            onClick={handleConnectLocal} 
            className="faucet-button secondary"
          >
             Connect to Local Node
          </button>
        </div>
        {error && (
          <div className="error-message" style={{marginTop: '1rem'}}>
            Error: {error}
          </div>
        )}
      </div>
    );
  }

  if (loading) {
    return (
      <div className="wallet-connection loading">
        <div className="loading-spinner">Connecting to wallet...</div>
      </div>
    );
  }

  if (error && !account) {
    return (
      <div className="wallet-connection error">
        <p>Error: {error}</p>
        <button onClick={handleDisconnect} className="retry-button">
          Go Back
        </button>
      </div>
    );
  }

  // CONNECTED VIEW
  return (
    <div className="wallet-connection-block">
      <div className="wallet-info">
        <h3 style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span>💰 Wallet Connected</span>
            <span className={`connection-badge ${connectionType}`}>
                {connectionType === 'local' ? 'LOCAL' : 'EXTERNAL'}
            </span>
        </h3>
        
        <div className="user-address">
          <strong>Address:</strong> {account}
        </div>
        <div className="balance-info">
          <strong>USDT Balance:</strong> {parseFloat(usdtBalance).toLocaleString()} USDT
        </div>

        {error && (
          <div className="error-message" style={{ margin: '10px 0' }}>
            {error}
          </div>
        )}

        {connectionType === 'local' ? (
          <div className="faucet-section">
            <button 
              onClick={requestFaucet}
              disabled={faucetLoading}
              className={`faucet-button ${faucetLoading ? 'loading' : ''}`}
            >
              {faucetLoading ? 'Minting USDT...' : '🎯 Get 1000 Test USDT'}
            </button>
            
            {faucetError && (
              <div className="error-message">{faucetError}</div>
            )}
            
            {faucetSuccess && (
              <div className="success-message">✅ Success! 1000 USDT added.</div>
            )}
          </div>
        ) : (
            <div className="faucet-section external-note">
                <p>ℹ️ Faucet is only available on Local Node connections.</p>
            </div>
        )}

        {connectionType === 'local' && availableAccounts.length > 1 && (
          <div className="account-selector">
            <button 
              onClick={() => setShowAccountSelector(!showAccountSelector)}
              className="toggle-selector"
            >
              🔄 Switch Test Account {showAccountSelector ? '▲' : '▼'}
            </button>
            
            {showAccountSelector && (
              <div className="accounts-list">
                {availableAccounts.map((acc, index) => (
                  <div 
                    key={acc}
                    className={`account-option ${acc === account ? 'active' : ''}`}
                    onClick={() => handleAccountSwitch(index)}
                  >
                    <span>Account {index}: {acc.slice(0, 8)}...{acc.slice(-6)}</span>
                    {acc === account && <span> ✅ Current</span>}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        <button onClick={() => {
          if (account && connectionType) {
            localStorage.setItem('connectedWalletType', connectionType);
            localStorage.setItem('connectedWalletAddress', account); // save selected wallet
          }
          onContinue?.();
        }} className="faucet-button primary">
          🚀 Continue to DApp
        </button>

        {/* ✅ Disconnect button */}
        <button onClick={handleDisconnect} className="faucet-button danger">
          ❌ Disconnect Wallet
        </button>
      </div>
    </div>
  );
};

export default WalletConnection;
