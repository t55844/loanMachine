// Web3Context.jsx
import { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';

// Import your contract ABI (you'll need to adjust this path)
import LoanMachineABI from '../../artifacts/contracts/LoanMachine.sol/LoanMachine.json';
import { fetchWalletMember } from './graphql-frontend-query';

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
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
  "function symbol() view returns (string)"
];

// USDT addresses for different networks
const USDT_ADDRESSES = {
  // Mainnet
  1: "0xdAC17F958D2ee523a2206206994597C13D831ec7",
  // Goerli (testnet - you might need to deploy mock USDT for testing)
  5: "0xdc31Ee1784292379Fbb2964b3B9C4124D8F89C60", // Goerli USDT
  // Sepolia
  11155111: "0xaA8E23Fb1079EA71e0a56F48a2aA51851D8433D0", // Sepolia USDT
  // Local Hardhat (you'll need to deploy a mock)
  31337: "0x5FbDB2315678afecb367f032d93F642f64180aa3" // Default Hardhat first contract
};
const Web3Context = createContext();

export function Web3Provider({ children }) {
  const [account, setAccount] = useState(null);
  const [contract, setContract] = useState(null);
  const [usdtContract, setUsdtContract] = useState(null);
  const [provider, setProvider] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [chainId, setChainId] = useState(null);
  const [member, setMember] = useState(null); // New state for member data

  useEffect(() => {
    connectToLocalNode();
  }, []);

  // Function to fetch member data
  const fetchMemberData = async (walletAddress) => {
    try {
      const memberData = await fetchWalletMember(walletAddress);
      setMember(memberData);
      return memberData;
    } catch (err) {
      console.error('Error fetching member data:', err);
      setMember(null);
      return null;
    }
  };

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
      const defaultAccount = accounts[5];
      
      // Create a signer (for write operations)
      const signer = localProvider.getSigner(defaultAccount);
      
      // Create LoanMachine contract instance
      const loanContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, signer);
      
      // Create USDT contract instance
      const usdtAddress = USDT_ADDRESSES[network.chainId];
      if (!usdtAddress) {
        throw new Error(`No USDT address configured for chain ID ${network.chainId}`);
      }
      
      const usdtTokenContract = new ethers.Contract(usdtAddress, USDT_ABI, signer);
      
      setProvider(localProvider);
      setContract(loanContract);
      setUsdtContract(usdtTokenContract);
      setAccount(defaultAccount);
      setError('');
      
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
        
        // Update both contracts with new signer
        const newContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
        const newUsdtContract = new ethers.Contract(usdtContract.address, USDT_ABI, newSigner);
        
        setAccount(newAccount);
        setContract(newContract);
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
    return tx; // ✅ Return the transaction object, not tx.wait()
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

  const value = {
    account,
    contract,
    usdtContract,
    provider,
    loading,
    error,
    chainId,
    member, // Include member data in context value
    switchAccount,
    connectToLocalNode,
    refreshMemberData, // Function to manually refresh member data
    // USDT helper functions
    getUSDTBalance,
    approveUSDT,
    getUSDTInfo,
    needsUSDTApproval
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