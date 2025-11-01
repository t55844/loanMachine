// components/StandaloneFaucet.jsx
import { useState } from 'react';
import { useWeb3 } from '../Web3Context';

const StandaloneFaucet = () => {
  const { 
    account, 
    getUSDTBalance,
    usdtContract
  } = useWeb3();
  
  const [usdtBalance, setUsdtBalance] = useState('0');
  const [faucetLoading, setFaucetLoading] = useState(false);
  const [faucetError, setFaucetError] = useState('');
  const [faucetSuccess, setFaucetSuccess] = useState(false);

  const refreshBalance = async () => {
    if (account) {
      try {
        const balance = await getUSDTBalance();
        setUsdtBalance(balance);
      } catch (err) {
        console.error('Error fetching balance:', err);
      }
    }
  };

  const requestFaucet = async () => {
    if (!usdtContract || !account) {
      setFaucetError('Wallet not connected');
      return;
    }

    setFaucetLoading(true);
    setFaucetError('');
    setFaucetSuccess(false);

    try {
      const amount = ethers.utils.parseUnits('1000', 6);
      const tx = await usdtContract.mint(account, amount);
      await tx.wait();
      
      await refreshBalance();
      setFaucetSuccess(true);
      setTimeout(() => setFaucetSuccess(false), 3000);
      
    } catch (err) {
      console.error('Faucet error:', err);
      setFaucetError(err.message || 'Failed to get USDT from faucet');
    } finally {
      setFaucetLoading(false);
    }
  };

  return (
    <div className="faucet-block">
      <h3>🚰 USDT Faucet</h3>
      
      {!account ? (
        <div className="not-connected-message">
          <p>Please connect your wallet to use the faucet.</p>
        </div>
      ) : (
        <div className="faucet-content">
          <div className="faucet-info">
            <div className="current-balance">
              <strong>Current Balance:</strong> {parseFloat(usdtBalance).toLocaleString()} USDT
            </div>
            <button 
              onClick={refreshBalance}
              className="refresh-button"
            >
              🔄 Refresh
            </button>
          </div>

          <div className="faucet-action">
            <button 
              onClick={requestFaucet}
              disabled={faucetLoading}
              className={`faucet-action-button ${faucetLoading ? 'loading' : ''}`}
            >
              {faucetLoading ? '⏳ Minting USDT...' : '🎯 Get 1000 Test USDT'}
            </button>
            
            <div className="faucet-note">
              <small>
                💡 This faucet provides <strong>fake USDT</strong> for testing purposes only. 
                You can use it multiple times.
              </small>
            </div>
          </div>

          {faucetError && (
            <div className="error-message">
              {faucetError}
            </div>
          )}
          
          {faucetSuccess && (
            <div className="success-message">
              ✅ Success! 1000 USDT has been added to your wallet
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default StandaloneFaucet;