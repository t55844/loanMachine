// Web3Context.jsx
import { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';

// Import your contract ABI (you'll need to adjust this path)
import LoanMachineABI from '../../artifacts/contracts/LoanMachine.sol/LoanMachine.json';

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
const RPC_URL = import.meta.env.VITE_RPC_URL;


const Web3Context = createContext();

export function Web3Provider({ children }) {
  const [account, setAccount] = useState(null);
  const [contract, setContract] = useState(null);
  const [provider, setProvider] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    connectToLocalNode();
  }, []);

  const connectToLocalNode = async () => {
    try {
      // Connect to local Hardhat node
      const localProvider = new ethers.providers.JsonRpcProvider(RPC_URL);
      
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
      
      // Create contract instance
      const loanContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, signer);
      
      setProvider(localProvider);
      setContract(loanContract);
      setAccount(defaultAccount);
      setError('');
      
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
        const newContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
        
        setAccount(newAccount);
        setContract(newContract);
      }
    } catch (err) {
      console.error('Error switching account:', err);
    }
  };

  const value = {
    account,
    contract,
    provider,
    loading,
    error,
    switchAccount,
    connectToLocalNode
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