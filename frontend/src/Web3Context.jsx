// Web3Context.jsx (FINAL)

import { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';

// Import your contract ABIs
import LoanMachineABI from '../src/abi/LoanMachine.json';
import ReputationSystemABI from '../src/abi/ReputationSystem.json';
import { fetchWalletMember } from './graphql-frontend-query';

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
const REPUTATION_CONTRACT_ADDRESS = import.meta.env.VITE_REPUTATION_CONTRACT_ADDRESS;
const RPC_URL = import.meta.env.VITE_RPC_URL;
const VITE_SUBGRAPH_URL = import.meta.env.VITE_SUBGRAPH_URL;

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

// Para desenvolvimento local, use o endereço do MockUSDT do seu deploy
const MOCK_USDT_ADDRESS = import.meta.env.VITE_MOCK_USDT_ADDRESS;

const Web3Context = createContext();

export function Web3Provider({ children }) {
  const [account, setAccount] = useState(null);
  const [contract, setContract] = useState(null);
  const [reputationContract, setReputationContract] = useState(null);
  const [usdtContract, setUsdtContract] = useState(null);
  const [provider, setProvider] = useState(null);
  // Estado inicial agora é false, sem conexão automática
  const [loading, setLoading] = useState(false); 
  const [error, setError] = useState('');
  const [chainId, setChainId] = useState(null);
  const [member, setMember] = useState(null);
  const [connectionType, setConnectionType] = useState(null); 

  // Function to fetch member data
  const fetchMemberData = async (walletAddress) => {
    try {
      let memberData;
      let hasVinculation = true;
      
      try {
        memberData = await fetchWalletMember(walletAddress);
        if (!memberData || (!memberData.id && !memberData.memberId)) {
          console.warn('⚠️ Wallet not vinculated to any member');
          hasVinculation = false;
          memberData = null;
        }
      } catch (subgraphError) {
        console.warn('❌ Subgraph query failed:', subgraphError);
        hasVinculation = false;
        memberData = null;
      }
      
      if (!hasVinculation || !memberData) {
        const noMemberData = {
          id: null,
          memberId: null,
          walletAddress: walletAddress,
          hasVinculation: false,
          error: 'Wallet not vinculated to any member'
        };
        setMember(noMemberData);
        return noMemberData;
      }
      
      const finalMemberData = {
        id: memberData.memberId || memberData.id,
        memberId: memberData.memberId || memberData.id,
        walletAddress: memberData.wallet?.id || walletAddress,
        name: memberData.name || `Member ${memberData.memberId || memberData.id}`,
        hasVinculation: true,
        ...memberData
      };
      
      setMember(finalMemberData);
      return finalMemberData;
      
    } catch (err) {
      console.error('❌ Error in fetchMemberData:', err);
      const errorMemberData = {
        id: null,
        memberId: null,
        walletAddress: walletAddress,
        hasVinculation: false,
        error: 'Failed to check member vinculation'
      };
      setMember(errorMemberData);
      return errorMemberData;
    }
  };

 useEffect(() => {
  // Remove a conexão automática e define loading como false
  setLoading(false); 

  // ✅ Auto-reconnect from localStorage
  const savedType = localStorage.getItem('connectedWalletType');
  if (!account && savedType === 'local') connectToLocalNode();
  else if (!account && savedType === 'external') connectToExternalWallet();

}, []); 

  // Configurar contratos com um Provider/Signer
  const setupContracts = async (newProvider, newSigner, newAccount, newChainId, type) => {
    
    // Verificações de rede (Apenas para local)
    if (type === 'local' && newChainId !== 31337) { 
        console.warn('ChainId é diferente do esperado para o ambiente local:', newChainId);
    }
    
    // 1. Contratos principais com o novo Signer
    const loanContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
    const reputationSystemContract = new ethers.Contract(
      REPUTATION_CONTRACT_ADDRESS, 
      ReputationSystemABI.abi, 
      newSigner
    );
    
    // 2. Contrato USDT (usando endereço mock em ambos os casos)
    const usdtAddress = MOCK_USDT_ADDRESS;
    if (!usdtAddress) {
      throw new Error('Mock USDT address not configured. Check VITE_MOCK_USDT_ADDRESS env variable');
    }

    const usdtTokenContract = new ethers.Contract(usdtAddress, USDT_ABI, newSigner);

    // Teste de conexão com o contrato USDT (pode falhar se estiver em uma rede externa)
    try {
        await usdtTokenContract.symbol();
    } catch (testError) {
        console.warn(`⚠️ Não foi possível conectar ao MockUSDT em ${usdtAddress}. Isso é esperado se você NÃO estiver na rede Hardhat local.`);
        // Se for conexão externa, o contrato USDT não será funcional, mas o restante sim.
        if(type === 'external') {
           setError('MockUSDT contract not found on this network. Faucet will be disabled.');
        } else {
            // Se for local e falhar, é um erro real.
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
      // Não limpe o erro do MockUSDT
    } else {
      setError('');
    }
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

    const defaultAccount = preferredAccount && accounts.includes(preferredAccount) 
      ? preferredAccount 
      : accounts[0];

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

    const defaultAccount = preferredAccount && accounts.includes(preferredAccount) 
      ? preferredAccount 
      : accounts[0];

    const signer = localProvider.getSigner(defaultAccount);
    await setupContracts(localProvider, signer, defaultAccount, network.chainId, 'local');
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
  };

  // Troca de Conta (apenas para conexão local)
  const switchAccount = async (accountIndex) => {
    if (connectionType !== 'local') {
        console.warn('Account switching is only available for local node connections.');
        return;
    }

    try {
      const accounts = await provider.listAccounts();
      if (accountIndex >= 0 && accountIndex < accounts.length) {
        const newAccount = accounts[accountIndex];
        const newSigner = provider.getSigner(newAccount);
        
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
        
        await fetchMemberData(newAccount);
      }
    } catch (err) {
      console.error('Error switching account:', err);
    }
  };

  // --- Funções Auxiliares (sem alterações) ---

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