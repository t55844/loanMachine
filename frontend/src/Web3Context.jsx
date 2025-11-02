import { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';

// Import your contract ABIs
import LoanMachineABI from '../src/abi/LoanMachine.json';
import ReputationSystemABI from '../src/abi/ReputationSystem.json';
import { fetchWalletMember } from './graphql-frontend-query';

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
const REPUTATION_CONTRACT_ADDRESS = import.meta.env.VITE_REPUTATION_CONTRACT_ADDRESS;
const RPC_URL = import.meta.env.VITE_RPC_URL;

// USDT ABI - minimal version for the functions we need
const USDT_ABI = [
  "function balanceOf(address) view returns (uint256)",
  "function approve(address spender, uint256 amount) returns (bool)",
  "function allowance(address owner, address spender) view returns (uint256)",
  "function transfer(address to, uint256 amount) returns (bool)",
  "function transferFrom(address from, address to, uint256 amount) returns (bool)",
  "function decimals() view returns (uint8)",
  "function name() view returns (string)",
  "function symbol() view returns (string)",
  "function mint(address to, uint256 amount) returns (bool)"
];

const MOCK_USDT_ADDRESS = import.meta.env.VITE_MOCK_USDT_ADDRESS;

const Web3Context = createContext();

export function Web3Provider({ children }) {
  const [account, setAccount] = useState(null);
  const [contract, setContract] = useState(null);
  const [reputationContract, setReputationContract] = useState(null);
  const [usdtContract, setUsdtContract] = useState(null);
  const [provider, setProvider] = useState(null);
  const [loading, setLoading] = useState(false); 
  const [error, setError] = useState('');
  const [chainId, setChainId] = useState(null);
  const [member, setMember] = useState(null);
  const [connectionType, setConnectionType] = useState(null); 

  // Function to fetch member data
    const fetchMemberData = async (walletAddress) => {
        const defaultMemberData = { 
            id: null, memberId: null, walletAddress, hasVinculation: false 
        };

        try {
            const subgraphResult = await fetchWalletMember(walletAddress); 
            // Keep debug logs for now to confirm the true state on reload
            console.log('Subgraph Result:', subgraphResult); 
            console.log('Vinculation Status:', subgraphResult?.hasVinculation); 
            
            if (subgraphResult && subgraphResult.hasVinculation) {
                
                // SUCCESS PATH: Vinculation found
                const finalMemberData = {
                    id: subgraphResult.memberId,
                    memberId: subgraphResult.memberId,
                    walletAddress: walletAddress,
                    name: `Member ${subgraphResult.memberId}`,
                    hasVinculation: true,
                    ...subgraphResult
                };

                setMember(finalMemberData);
                return finalMemberData;

            } else {
                // NO VINCUATION PATH
                const noMemberData = {
                    ...defaultMemberData,
                    error: 'Wallet not vinculated to any member',
                };
                
                setMember(noMemberData); 
                return noMemberData;
            }

        } catch (err) {
            console.error('❌ Error in fetchMemberData (Network/Subgraph Query):', err);
            const errorMemberData = {
                ...defaultMemberData,
                error: 'Failed to check member vinculation due to network/subgraph error.',
            };

            setMember(errorMemberData);
            return errorMemberData;
        }
    };

    // 🚩 AUTO-RECONNECT LOGIC
    useEffect(() => {
        const savedType = localStorage.getItem('connectedWalletType');
        const savedAccount = localStorage.getItem('connectedWalletAddress'); // Use your specific key
        
        if (savedType && savedAccount && !account) {
            setLoading(true); 
            // Pass the saved account to initiate reconnection
            if (savedType === 'local') connectToLocalNode(savedAccount);
            else if (savedType === 'external') connectToExternalWallet(savedAccount);
        } else if (!savedType) {
            setLoading(false);
        }
    }, []); 

  // Configurar contratos com um Provider/Signer
  const setupContracts = async (newProvider, newSigner, newAccount, newChainId, type) => {
    // ... (network checks and contract instantiation) ...
    
    // 1. Contratos principais com o novo Signer
    const loanContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
    const reputationSystemContract = new ethers.Contract(
      REPUTATION_CONTRACT_ADDRESS, 
      ReputationSystemABI.abi, 
      newSigner
    );
    
    // 2. Contrato USDT
    const usdtAddress = MOCK_USDT_ADDRESS;
    if (!usdtAddress) {
      throw new Error('Mock USDT address not configured. Check VITE_MOCK_USDT_ADDRESS env variable');
    }

    const usdtTokenContract = new ethers.Contract(usdtAddress, USDT_ABI, newSigner);

    // Teste de conexão com o contrato USDT (retained for safety)
    try {
        await usdtTokenContract.symbol();
    } catch (testError) {
        console.warn(`⚠️ Não foi possível conectar ao MockUSDT em ${usdtAddress}. Isso é esperado se você NÃO estiver na rede Hardhat local.`);
        if(type === 'external') {
           setError('MockUSDT contract not found on this network. Faucet will be disabled.');
        } else {
            throw new Error(`USDT contract not working at ${usdtAddress}. Please check deployment.`);
        }
    }

    setProvider(newProvider);
    setContract(loanContract);
    setReputationContract(reputationSystemContract);
    setUsdtContract(usdtTokenContract);
    setAccount(newAccount);
    setChainId(newChainId);
    setConnectionType(type); 
    if(error === 'MockUSDT contract not found on this network. Faucet will be disabled.') {
      // Don't clear the MockUSDT error
    } else {
      setError('');
    }
    
    // No longer saving here. We rely on the initial saving from WalletConnection.jsx
    await fetchMemberData(newAccount);
    setLoading(false); 
  };

  // Conexão com Provedor Externo (MetaMask, etc.)
  const connectToExternalWallet = async (preferredAccount = null) => { 
  setLoading(true);
  setError('');

  if (!window.ethereum) {
    setError('No external wallet provider detected.');
    setLoading(false);
    return;
  }

  try {
    const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
    const externalProvider = new ethers.providers.Web3Provider(window.ethereum);
    const signer = externalProvider.getSigner();
    const network = await externalProvider.getNetwork();

    // 🚨 NEW LOGIC: ONLY use preferredAccount, otherwise error and force back to connection screen
    const defaultAccount = preferredAccount && accounts.includes(preferredAccount) 
      ? preferredAccount 
      : accounts[0]; // Retain for first time connection, as accounts[0] is the authorized one

    if (!defaultAccount) {
      throw new Error('No authorized account found from wallet provider.');
    }
    
    await setupContracts(externalProvider, signer, defaultAccount, network.chainId, 'external'); 

    window.ethereum.on('accountsChanged', (newAccounts) => {
      if (newAccounts.length > 0) window.location.reload();
      else disconnect();
    });
    window.ethereum.on('chainChanged', () => window.location.reload());
  } catch (err) {
    console.error('Error connecting to external wallet:', err);
    setError(`Failed to connect: ${err.message}`);
    setLoading(false);
  }
};

  // Conexão com Nó Local
 const connectToLocalNode = async (preferredAccount = null) => { 
  setLoading(true); 
  setError('');
  try {
    const localProvider = new ethers.providers.JsonRpcProvider(RPC_URL);
    const network = await localProvider.getNetwork();
    const accounts = await localProvider.listAccounts();
    
    if (accounts.length === 0) throw new Error('No accounts found in local node');

    // 🚨 NEW LOGIC: ONLY use preferredAccount. If not passed during reconnect, fail.
    // However, on first connect (when preferredAccount is null), use accounts[0] (the default)
    // The "default wallet logic" is essential here if the user hasn't selected a wallet yet.
    const finalAccount = preferredAccount || accounts[0]; // Safe fallback for first connection

    if (!accounts.includes(finalAccount)) {
      throw new Error('Saved account not available in local node.');
    }

    const signer = localProvider.getSigner(finalAccount);
    await setupContracts(localProvider, signer, finalAccount, network.chainId, 'local'); 
  } catch (err) {
    console.error('Error connecting to local node:', err);
    setError(`Failed to connect to local node: ${err.message}`);
    setLoading(false);
  }
};

  // Desconexão
  const disconnect = () => {
      setAccount(null);
      setContract(null);
      setReputationContract(null);
      setUsdtContract(null);
      setProvider(null);
      setMember(null);
      setLoading(false);
      setError('');
      setConnectionType(null);
      // NOTE: We do NOT clear localStorage here, as this function is called 
      // by external wallet listeners. The App.jsx/WalletConnection handles clearing.
  };

  // Troca de Conta (apenas para conexão local)
  const switchAccount = async (accountIndex) => {
    if (connectionType !== 'local') {
        console.warn('Account switching is only available for local node connections.');
        return;
    }

    setLoading(true); 

    try {
      const accounts = await provider.listAccounts();
      if (accountIndex >= 0 && accountIndex < accounts.length) {
        const newAccount = accounts[accountIndex];
        const newSigner = provider.getSigner(newAccount);
        
        // Re-instantiate contracts with the new signer
        const newContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
        const newReputationContract = new ethers.Contract(
          REPUTATION_CONTRACT_ADDRESS, 
          ReputationSystemABI.abi, 
          newSigner
        );
        const newUsdtContract = new ethers.Contract(usdtContract.address, USDT_ABI, newSigner);
        
        setAccount(newAccount);
        setContract(newContract);
        setReputationContract(newReputationContract); 
        setUsdtContract(newUsdtContract);

        // NOTE: WalletConnection.jsx is responsible for saving the new account to localStorage 
        // after a switch, but we'll leave it to the user to handle.
        
        await fetchMemberData(newAccount);
      }
    } catch (err) {
      console.error('Error switching account:', err);
    } finally {
        setLoading(false); 
    }
  };

  // ... (Auxiliary functions: getUSDTBalance, approveUSDT, etc. - unchanged) ...
 // 

  // Helper function to get USDT balance for any address
  const getUSDTBalance = async (address = null) => {
    if (!usdtContract) throw new Error('USDT contract not initialized');
    try {
      const targetAddress = address || account;
      const balance = await usdtContract.balanceOf(targetAddress);
      return ethers.utils.formatUnits(balance, 6); // USDT has 6 decimals
    } catch (err) {
      console.warn("Could not fetch USDT balance (expected on external networks)", err.message);
      return '0';
    }
  };

  // Helper function to approve USDT spending
  const approveUSDT = async (amount) => {
    if (!usdtContract || !contract) throw new Error('Contracts not initialized');
    const amountInWei = ethers.utils.parseUnits(amount.toString(), 6);
    const tx = await usdtContract.approve(contract.address, amountInWei);
    return tx;
  };

  // Helper function to get USDT info
  const getUSDTInfo = async () => {
    if (!usdtContract) throw new Error('USDT contract not initialized');
    try {
      const [name, symbol, decimals] = await Promise.all([
        usdtContract.name(),
        usdtContract.symbol(),
        usdtContract.decimals()
      ]);
      return { name, symbol, decimals, address: usdtContract.address };
    } catch(err) {
      console.warn("Could not fetch USDT info", err.message);
      return { name: 'MockUSDT', symbol: 'mUSDT', decimals: 6, address: MOCK_USDT_ADDRESS };
    }
  };

  // Check if user needs to approve USDT for a specific amount
  const needsUSDTApproval = async (amount) => {
    if (!usdtContract || !contract) return true;
    try {
      const currentAllowance = await usdtContract.allowance(account, contract.address);
      const amountInWei = ethers.utils.parseUnits(amount.toString(), 6);
      return currentAllowance.lt(amountInWei);
    } catch (err) {
      console.error('Error checking allowance:', err);
      return true;
    }
  };

  // Function to refresh member data
  const refreshMemberData = async () => {
    if (account) {
      return await fetchMemberData(account);
    }
    return null;
  };

  // GET MEMBER ID FROM REPUTATION CONTRACT
  const getMemberIdFromReputation = async (walletAddress = null) => {
    if (!reputationContract) throw new Error('Reputation contract not initialized');
    const address = walletAddress || account;
    try {
      const memberId = await reputationContract.getMemberId(address);
      return memberId;
    } catch (err) {
      console.error('Error getting member ID from reputation contract:', err);
      return 0;
    }
  };

  // GET REPUTATION SCORE
  const getReputation = async (memberId = null) => {
    if (!reputationContract) throw new Error('Reputation contract not initialized');
    let targetMemberId = memberId;
    if (!targetMemberId && member?.memberId) {
      targetMemberId = member.memberId;
    }
    if (!targetMemberId) {
      throw new Error('No member ID provided');
    }
    try {
      const reputation = await reputationContract.getReputation(targetMemberId);
      return reputation;
    } catch (err) {
      console.error('Error getting reputation:', err);
      return 0;
    }
  };

  const value = {
    account,
    contract,
    reputationContract, 
    usdtContract,
    provider,
    loading,
    error,
    chainId,
    member,
    connectionType, 
    switchAccount,
    connectToLocalNode,
    connectToExternalWallet, 
    disconnect, 
    refreshMemberData,
    // USDT helper functions
    getUSDTBalance,
    approveUSDT,
    getUSDTInfo,
    needsUSDTApproval,
    // REPUTATION HELPER FUNCTIONS
    getMemberIdFromReputation,
    getReputation
  };

  return (
    <Web3Context.Provider value={value}>
      {children}
    </Web3Context.Provider>
  );
}

export function useWeb3() {
  const context = useContext(Web3Context);
  if (!context) {
    throw new Error('useWeb3 must be used within a Web3Provider');
  }
  return context;
}