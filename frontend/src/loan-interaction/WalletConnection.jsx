import { useState, useEffect } from "react";
import { useWeb3 } from "../Web3Context";
import { ethers } from "ethers";
import { useToast } from "../handlers/useToast";
import Toast from "../handlers/Toast";

const WalletConnection = ({ onContinue }) => {
  const {
    account,
    provider,
    loading,
    error: web3Error,
    getUSDTBalance,
    usdtContract,
    connectToExternalWallet,
    connectWithPrivateKey,
    disconnect,
    connectionType,
    signer,
    config // Added: From proxy-fetched config
  } = useWeb3();
  const { showError, showSuccess } = useToast(provider, usdtContract); // UPDATED: Pass provider/usdtContract

  const valueToMint = config?.valueToMint || '1000'; // Fallback if not loaded
  const [usdtBalance, setUsdtBalance] = useState('0');
  const [faucetLoading, setFaucetLoading] = useState(false);
  const [privateKeyInput, setPrivateKeyInput] = useState('');
  const [showDemoInput, setShowDemoInput] = useState(false);

  // ✅ Enhanced auto-reconnect handling: Clear invalid local connections on failure
  useEffect(() => {
    if (web3Error && web3Error.includes('local node') && !account) {
      // Clear invalid local storage on local node failure
      localStorage.removeItem('connectedWalletType');
      localStorage.removeItem('connectedWalletAddress');
      localStorage.removeItem('demoPrivateKey');
      setLoading(false); // Ensure loading stops
    }
  }, [web3Error, account]);

  // ✅ Load saved connection (external and demo only) - skip local here since UI doesn't support it
  useEffect(() => {
    const savedType = localStorage.getItem('connectedWalletType');
    const savedAccount = localStorage.getItem('connectedWalletAddress');
    const savedPK = localStorage.getItem('demoPrivateKey');
    if (!account && savedType) {
      if (savedType === 'external') {
        connectToExternalWallet(savedAccount);
      } else if (savedType === 'demo' && savedPK) {
        connectWithPrivateKey(savedPK);
      }
      // Skip 'local' here - let context handle, but clear on failure via above effect
    }
  }, [account, connectToExternalWallet, connectWithPrivateKey]);

  // ✅ Update balance when connected (no local accounts)
  useEffect(() => {
    const fetchData = async () => {
      if (account && provider) {
        try {
          const balance = await getUSDTBalance();
          setUsdtBalance(balance);
        } catch (err) {
          //console.error('Erro ao buscar dados:', err);
          await showError(err); // UPDATED: Await showError
        }
      }
    };
    fetchData();
  }, [account, provider, getUSDTBalance, connectionType, showError]);

  // ✅ Request faucet (enabled for demo)
  const requestFaucet = async () => {
    if (!usdtContract || !account) {
      showError('Carteira não conectada ou contrato USDT não encontrado');
      return;
    }
    setFaucetLoading(true);
    try {
      const amount = ethers.utils.parseUnits(valueToMint, 6); // FIXED: Use utils.parseUnits
      const tx = await usdtContract.mint(account, amount);
      await tx.wait();
      const newBalance = await getUSDTBalance();
      setUsdtBalance(newBalance);
      showSuccess(`Sucesso! ${valueToMint} USDT adicionados.`);
    } catch (err) {
      //console.error('Erro no faucet:', err);
      await showError(err); // UPDATED: Await
    } finally {
      setFaucetLoading(false);
    }
  };

  // ✅ Handle connect buttons (external and demo only)
  const handleConnectExternal = async () => {
    try {
      await connectToExternalWallet();
      localStorage.setItem('connectedWalletType', 'external');
    } catch (err) {
      await showError(err); // UPDATED: Await
    }
  };

  const handleConnectDemoWithKey = async () => {
    if (!privateKeyInput) {
      showError('Insira uma chave privada válida.');
      return;
    }
    try {
      await connectWithPrivateKey(privateKeyInput);
      localStorage.setItem('connectedWalletType', 'demo');
    } catch (err) {
      await showError(err); // UPDATED: Await
    }
  };

  // ✅ Handle disconnect (clear demo PK too)
  const handleDisconnect = () => {
    disconnect();
    localStorage.removeItem('connectedWalletType');
    localStorage.removeItem('connectedWalletAddress');
    localStorage.removeItem('demoPrivateKey');
    window.location.reload();
  };

  // Show error if web3 error (e.g., local node failure)
  if (web3Error && !account) {
    return (
      <div className="wallet-connection-block error-state">
        <h3>❌ Erro na Conexão</h3>
        <p>{web3Error}</p>
        <p>Por favor, tente conectar novamente ou verifique se o nó local está rodando (se aplicável).</p>
        <button onClick={() => window.location.reload()} className="faucet-button primary">
          🔄 Tentar Novamente
        </button>
      </div>
    );
  }

  // INITIAL CONNECTION SCREEN (external and demo only)
  if (!account && !loading) {
    return (
      <div className="wallet-connection-block initial-state">
        <h3>🔌 Conecte Sua Carteira</h3>
        <p>Escolha como deseja conectar para teste.</p>
        <div className="connection-options" style={{display: 'flex', flexDirection:'row', justifyContent: 'space-around'}}>
          <button onClick={handleConnectExternal} className="faucet-button primary" >
            Conectar Carteira Externa
          </button>
          <button onClick={() => setShowDemoInput(!showDemoInput)} className="faucet-button demo" >
            Conectar com Chave Privada Demo {showDemoInput ? '▲' : '▼'}
          </button>
        </div>
        {showDemoInput && (
          <div className="demo-input-section" style={{marginTop: '1rem'}}>
            <input
              type="text"
              placeholder="Cole sua chave privada demo (0x...)"
              value={privateKeyInput}
              onChange={(e) => setPrivateKeyInput(e.target.value)}
              style={{width: '100%', padding: '8px', marginBottom: '8px'}}
            />
            <button onClick={handleConnectDemoWithKey} className="faucet-button primary" >
              Conectar Demo
            </button>
            <p style={{fontSize: '12px', color: 'gray', marginTop: '8px'}}>
              ⚠️ Testnet apenas. Funde o endereço com ETH de teste.
            </p>
          </div>
        )}
      </div>
    );
  }

  if (loading) {
    return (
      <div className="wallet-connection loading">
        <div className="loading-spinner">Conectando à carteira...</div>
      </div>
    );
  }

  // CONNECTED VIEW (removed local-specific account selector)
  return (
    <>
      <Toast /> {/* ✅ Render Toast component here to ensure it's mounted */}
      <div className="wallet-connection-block">
        <div className="wallet-info">
          <h3 style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span>💰 Carteira Conectada</span>
            <span className={`connection-badge ${connectionType}`}>
              {connectionType === 'external' ? 'EXTERNA' : 'DEMO'}
            </span>
          </h3>
          <div className="user-address">
            <strong>Endereço:</strong> {account}
          </div>
          <div className="balance-info">
            <strong>Saldo USDT:</strong> {parseFloat(usdtBalance).toLocaleString()} USDT
          </div>
          {connectionType === 'demo' ? (
            <div className="faucet-section">
              <button
                onClick={requestFaucet}
                disabled={faucetLoading}
                className={`faucet-button ${faucetLoading ? 'loading' : ''}`}
              >
                {faucetLoading ? 'Mintando USDT...' : `🎯 Obter ${valueToMint} USDT de Teste`}
              </button>
            </div>
          ) : (
            <div className="faucet-section external-note">
              <p>ℹ️ O faucet está disponível apenas para conexões demo.</p>
            </div>
          )}
          <button
            onClick={() => {
              if (account && connectionType) {
                localStorage.setItem('connectedWalletType', connectionType);
                localStorage.setItem('connectedWalletAddress', account);
              }
              onContinue?.();
            }}
            className="faucet-button primary"
          >
            🚀 Continuar para DApp
          </button>
          <button onClick={handleDisconnect} className="faucet-button danger">
            ❌ Desconectar Carteira
          </button>
        </div>
      </div>
    </>
  );
};

export default WalletConnection;