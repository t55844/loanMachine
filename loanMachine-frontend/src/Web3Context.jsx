import { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';

// Import your contract ABIs
import LoanMachineABI from '../../artifacts/contracts/LoanMachine.sol/LoanMachine.json';
import ReputationSystemABI from '../../artifacts/contracts/ReputationSystem.sol/ReputationSystem.json';
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
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [chainId, setChainId] = useState(null);
  const [member, setMember] = useState(null);

  // ✅ ADICIONE ESTA FUNÇÃO: Function to fetch member data
  const fetchMemberData = async (walletAddress) => {
    try {
      
      let memberData;
      let hasVinculation = true;
      
      try {
        // Tente buscar do subgraph
        memberData = await fetchWalletMember(walletAddress);
        
        // Verifique se o member foi encontrado
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
      
      // Se não tem vinculação, setar member como null mas com flag de erro
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
      
      // Se tem vinculação, processar os dados
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
    connectToLocalNode();
  }, []);

  const connectToLocalNode = async () => {
    try {
      // Connect to local Hardhat node
      const localProvider = new ethers.providers.JsonRpcProvider(RPC_URL);
      
      // Get network chainId
      const network = await localProvider.getNetwork();
      setChainId(network.chainId);
      
      // Get accounts from local node
      const accounts = await localProvider.listAccounts();
      
      if (accounts.length === 0) {
        setError('No accounts found in local Hardhat node');
        setLoading(false);
        return;
      }

      // Use the first account as default
      const defaultAccount = accounts[1];
      
      // Create a signer (for write operations)
      const signer = localProvider.getSigner(defaultAccount);
      
      // Create LoanMachine contract instance
      const loanContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, signer);
      
      // ✅ ADD REPUTATION SYSTEM CONTRACT
      if (!REPUTATION_CONTRACT_ADDRESS) {
        throw new Error('Reputation contract address not configured. Check VITE_REPUTATION_CONTRACT_ADDRESS env variable');
      }
      
      const reputationSystemContract = new ethers.Contract(
        REPUTATION_CONTRACT_ADDRESS, 
        ReputationSystemABI.abi, 
        signer
      );
      
      // Use MockUSDT address
      const usdtAddress = MOCK_USDT_ADDRESS;
      
      if (!usdtAddress) {
        throw new Error('Mock USDT address not configured. Check VITE_MOCK_USDT_ADDRESS env variable');
      }
      
      const usdtTokenContract = new ethers.Contract(usdtAddress, USDT_ABI, signer);
      
      // Test the USDT contract connection
      try {
        const symbol = await usdtTokenContract.symbol();
        const decimals = await usdtTokenContract.decimals();
      } catch (testError) {
        console.error('❌ Failed to connect to USDT contract:', testError);
        throw new Error(`USDT contract not working at ${usdtAddress}. Please check deployment.`);
      }
      
      setProvider(localProvider);
      setContract(loanContract);
      setReputationContract(reputationSystemContract); // ✅ SET REPUTATION CONTRACT
      setUsdtContract(usdtTokenContract);
      setAccount(defaultAccount);
      setError('');
      
      // ✅ AGORA fetchMemberData ESTÁ DEFINIDA
      // Fetch member data after setting account
      await fetchMemberData(defaultAccount);
      
    } catch (err) {
      console.error('Error connecting to local node:', err);
      setError(`Failed to connect to local node: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  const switchAccount = async (accountIndex) => {
    try {
      const accounts = await provider.listAccounts();
      if (accountIndex >= 0 && accountIndex < accounts.length) {
        const newAccount = accounts[accountIndex];
        const newSigner = provider.getSigner(newAccount);
        
        // Update all contracts with new signer
        const newContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
        const newReputationContract = new ethers.Contract(
          REPUTATION_CONTRACT_ADDRESS, 
          ReputationSystemABI.abi, 
          newSigner
        );
        const newUsdtContract = new ethers.Contract(usdtContract.address, USDT_ABI, newSigner);
        
        setAccount(newAccount);
        setContract(newContract);
        setReputationContract(newReputationContract); // ✅ UPDATE REPUTATION CONTRACT
        setUsdtContract(newUsdtContract);
        
        // Fetch member data for the new account
        await fetchMemberData(newAccount);
      }
    } catch (err) {
      console.error('Error switching account:', err);
    }
  };

  // Helper function to get USDT balance for any address
  const getUSDTBalance = async (address = null) => {
    if (!usdtContract) throw new Error('USDT contract not initialized');
    const targetAddress = address || account;
    const balance = await usdtContract.balanceOf(targetAddress);
    return ethers.utils.formatUnits(balance, 6); // USDT has 6 decimals
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
    
    const [name, symbol, decimals] = await Promise.all([
      usdtContract.name(),
      usdtContract.symbol(),
      usdtContract.decimals()
    ]);
    
    return { name, symbol, decimals, address: usdtContract.address };
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

  // ✅ ADD HELPER TO GET MEMBER ID FROM REPUTATION CONTRACT
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

  // ✅ ADD HELPER TO GET REPUTATION SCORE
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
    reputationContract, // ✅ EXPORT REPUTATION CONTRACT
    usdtContract,
    provider,
    loading,
    error,
    chainId,
    member,
    switchAccount,
    connectToLocalNode,
    refreshMemberData,
    // USDT helper functions
    getUSDTBalance,
    approveUSDT,
    getUSDTInfo,
    needsUSDTApproval,
    // ✅ REPUTATION HELPER FUNCTIONS
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