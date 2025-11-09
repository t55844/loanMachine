import { createContext, useContext, useState, useEffect } from 'react';
import { ethers } from 'ethers';
import LoanMachineABI from '../src/abi/LoanMachine.json';
import ReputationSystemABI from '../src/abi/ReputationSystem.json';
import { fetchWalletMember, fetchMemberReputation } from './graphql-frontend-query';

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;
const REPUTATION_CONTRACT_ADDRESS = import.meta.env.VITE_REPUTATION_CONTRACT_ADDRESS;
const RPC_URL = import.meta.env.VITE_RPC_URL;

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
  const [signer, setSigner] = useState(null); // ✅ Added for explicit signer access
  const [loanInterface, setLoanInterface] = useState(null); // NEW: Interface for error decoding
  const [loading, setLoading] = useState(false); 
  const [error, setError] = useState('');
  const [chainId, setChainId] = useState(null);
  const [member, setMember] = useState(null);
  const [connectionType, setConnectionType] = useState(null); 

  // Simple function to fetch and set member data
  const fetchAndSetMemberData = async (walletAddress) => {
    try {
      const subgraphResult = await fetchWalletMember(walletAddress); 
      
      if (subgraphResult && subgraphResult.hasVinculation) {
        const reputation = await fetchMemberReputation(subgraphResult.memberId);
        
        const memberData = {
          ...subgraphResult,
          id: subgraphResult.memberId,
          memberId: subgraphResult.memberId,
          walletAddress: walletAddress,
          name: `Member ${subgraphResult.memberId}`,
          hasVinculation: true,
          currentReputation: reputation,
          wallets: subgraphResult.wallets || []
        };
        setMember(memberData);
        return memberData;
      } else {
        const noMemberData = { 
          id: null, 
          memberId: null, 
          walletAddress, 
          hasVinculation: false,
          currentReputation: 0
        };
        setMember(noMemberData);
        return noMemberData;
      }
    } catch (err) {
      //console.error('Error fetching member data:', err);
      const errorMemberData = {
        id: null, 
        memberId: null, 
        walletAddress, 
        hasVinculation: false,
        currentReputation: 0,
        error: 'Failed to fetch member data'
      };
      setMember(errorMemberData);
      return errorMemberData;
    }
  };

  // Auto-reconnect logic (added demo)
  useEffect(() => {
    const savedType = localStorage.getItem('connectedWalletType');
    const savedAccount = localStorage.getItem('connectedWalletAddress');
    const savedPK = localStorage.getItem('demoPrivateKey');
    
    if (savedType && savedAccount && !account) {
      setLoading(true); 
      if (savedType === 'local') connectToLocalNode(savedAccount);
      else if (savedType === 'external') connectToExternalWallet(savedAccount);
      else if (savedType === 'demo' && savedPK) connectWithPrivateKey(savedPK);
    } else if (!savedType) {
      setLoading(false);
    }
  }, [account]);

  // Setup contracts
  const setupContracts = async (newProvider, newSigner, newAccount, newChainId, type) => {
    const expectedChainId = import.meta.env.VITE_EXPECTED_CHAIN_ID;
    if (expectedChainId && parseInt(expectedChainId) !== parseInt(newChainId)) {
      setError(`Wrong network. Please switch to chain ID ${expectedChainId}`);
      setLoading(false);
      return;
    }

    const loanContract = new ethers.Contract(CONTRACT_ADDRESS, LoanMachineABI.abi, newSigner);
    const reputationSystemContract = new ethers.Contract(
      REPUTATION_CONTRACT_ADDRESS, 
      ReputationSystemABI.abi, 
      newSigner
    );
    
    // NEW: Create Interface for LoanMachine (for error decoding)
    const loanInterface = new ethers.utils.Interface(LoanMachineABI.abi);
    
    const usdtAddress = MOCK_USDT_ADDRESS;
    if (!usdtAddress) {
      throw new Error('Mock USDT address not configured. Check VITE_MOCK_USDT_ADDRESS env variable');
    }

    const usdtTokenContract = new ethers.Contract(usdtAddress, USDT_ABI, newSigner);

    try {
      await usdtTokenContract.symbol();
    } catch (testError) {
      console.warn(`⚠️ Cannot connect to MockUSDT at ${usdtAddress}. Expected if not on local Hardhat network.`);
      if(type === 'external' || type === 'demo') { // ✅ Allow demo/external to proceed with warning
        setError('MockUSDT contract not found on this network. Faucet will be disabled.');
      } else {
        throw new Error(`USDT contract not working at ${usdtAddress}. Please check deployment.`);
      }
    }

    // NEW: One-time authorization of LoanMachine as caller (auto-runs if owner)
    const maybeAuthorizeLoanMachine = async () => {
      if (!reputationSystemContract || !newSigner || !newAccount) return; // Skip if not ready

      try {
        // Check if current signer is the owner
        const owner = await reputationSystemContract.owner();
        const signerAddress = await newSigner.getAddress();
        if (owner.toLowerCase() !== signerAddress.toLowerCase()) {
          //console.log('Skipping authorization: Not owner wallet.');
          return;
        }

        // Check if LoanMachine is already authorized
        const isAuthorized = await reputationSystemContract.authorizedCallers(CONTRACT_ADDRESS);
        if (isAuthorized) {
          //console.log('LoanMachine already authorized.');
          return;
        }

        //console.log('Authorizing LoanMachine as caller...');
        const tx = await reputationSystemContract.setAuthorizedCaller(CONTRACT_ADDRESS, true);
        const receipt = await tx.wait();
        //console.log('Authorization successful. Tx hash:', receipt.transactionHash);
      } catch (err) {
        //console.error('Auto-authorization skipped due to error:', err.message);
        // Don't throw – keep connection flowing
      }
    };

    // Trigger the authorization check
    await maybeAuthorizeLoanMachine();

    setProvider(newProvider);
    setSigner(newSigner); // ✅ Set signer
    setContract(loanContract);
    setReputationContract(reputationSystemContract);
    setUsdtContract(usdtTokenContract);
    setLoanInterface(loanInterface); // NEW: Set interface
    setAccount(newAccount);
    setChainId(newChainId);
    setConnectionType(type); 
    
    // Fetch member data after setting up contracts
    await fetchAndSetMemberData(newAccount);
    setLoading(false); 
  };

  // External wallet connection
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
      //console.error('Error connecting to external wallet:', err);
      setError(`Failed to connect: ${err.message}`);
      setLoading(false);
    }
  };

  // Local node connection
  const connectToLocalNode = async (preferredAccount = null) => { 
    setLoading(true); 
    setError('');
    try {
      const localProvider = new ethers.providers.JsonRpcProvider(RPC_URL);
      const network = await localProvider.getNetwork();
      const accounts = await localProvider.listAccounts();
      
      if (accounts.length === 0) throw new Error('No accounts found in local node');

      const finalAccount = preferredAccount || accounts[0];

      if (!accounts.includes(finalAccount)) {
        throw new Error('Saved account not available in local node.');
      }

      const signer = localProvider.getSigner(finalAccount);
      await setupContracts(localProvider, signer, finalAccount, network.chainId, 'local'); 
    } catch (err) {
      //console.error('Error connecting to local node:', err);
      setError(`Failed to connect to local node: ${err.message}`);
      setLoading(false);
    }
  };

  // ✅ New: Connect with private key (for demo mode)
  const connectWithPrivateKey = async (privateKey) => {
    setLoading(true);
    setError('');

    try {
      const rpcUrl = RPC_URL; // 'https://rpc.sepolia.org'; // Public Sepolia RPC (or your Alchemy/Infura URL)
      const demoProvider = new ethers.providers.JsonRpcProvider(rpcUrl); // FIXED: v5 syntax

      // Validate and create wallet from PK
      if (!privateKey.startsWith('0x')) privateKey = '0x' + privateKey;
      const demoWallet = new ethers.Wallet(privateKey);
      const demoSigner = demoWallet.connect(demoProvider);
      const demoAddress = demoWallet.address;
      const network = await demoProvider.getNetwork();

      await setupContracts(demoProvider, demoSigner, demoAddress, network.chainId, 'demo');

      // Persist PK and address (insecure - testnet only)
      localStorage.setItem('demoPrivateKey', privateKey);
      localStorage.setItem('connectedWalletAddress', demoAddress);
      localStorage.setItem('connectedWalletType', 'demo');
    } catch (err) {
      //console.error('Error connecting with private key:', err);
      setError(`Failed to connect with private key: ${err.message}`);
      setLoading(false);
    }
  };

  const switchAccount = async (accountIndex) => {
    if (connectionType !== 'local' || !provider || !(provider instanceof ethers.providers.JsonRpcProvider)) {
      //console.warn('Account switching is only supported for local node connections (JsonRpcProvider).');
      return;
    }

    setLoading(true);
    setError('');
    
    try {
      // We can assume provider is a JsonRpcProvider here due to the check above
      const accounts = await provider.listAccounts(); 
      if (accountIndex >= accounts.length) throw new Error('Invalid account index');
      
      const newAccount = accounts[accountIndex];
      const signer = provider.getSigner(newAccount);
      const network = await provider.getNetwork();
      
      // Re-setup contracts with the new signer/account
      await setupContracts(provider, signer, newAccount, network.chainId, 'local'); 

      // ✅ Crucial step: Save the newly selected account for auto-reconnect
      localStorage.setItem('connectedWalletAddress', newAccount);

    } catch (err) {
      //console.error('Error switching account:', err);
      setError(`Failed to switch account: ${err.message}`);
    } finally {
      setLoading(false);
    }
  };

  // Disconnect
  const disconnect = () => {
    setAccount(null);
    setContract(null);
    setReputationContract(null);
    setUsdtContract(null);
    setProvider(null);
    setSigner(null); // ✅ Clear signer
    setLoanInterface(null); // NEW: Clear interface
    setMember(null);
    setLoading(false);
    setError('');
    setConnectionType(null);
    localStorage.removeItem('demoPrivateKey'); // ✅ Clear demo PK
  };

  // Simple refresh function - just refetches member data
  const refreshMemberData = () => {
    if (account) {
      return fetchAndSetMemberData(account);
    }
    return Promise.resolve(null);
  };

  // USDT functions
  const getUSDTBalance = async (address = null) => {
    if (!usdtContract) throw new Error('USDT contract not initialized');
    try {
      const targetAddress = address || account;
      const balance = await usdtContract.balanceOf(targetAddress);
      return ethers.utils.formatUnits(balance, 6);
    } catch (err) {
      //console.warn("Could not fetch USDT balance", err.message);
      return '0';
    }
  };

  const approveUSDT = async (amount) => {
    if (!usdtContract || !contract) throw new Error('Contracts not initialized');
    const amountInWei = ethers.utils.parseUnits(amount.toString(), 6);
    const tx = await usdtContract.approve(contract.address, amountInWei);
    return tx;
  };

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
      //console.warn("Could not fetch USDT info", err.message);
      return { name: 'MockUSDT', symbol: 'mUSDT', decimals: 6, address: MOCK_USDT_ADDRESS };
    }
  };

  const needsUSDTApproval = async (amount) => {
    if (!usdtContract || !contract) return true;
    try {
      const currentAllowance = await usdtContract.allowance(account, contract.address);
      const amountInWei = ethers.utils.parseUnits(amount.toString(), 6);
      return currentAllowance.lt(amountInWei);
    } catch (err) {
      //console.error('Error checking allowance:', err);
      return true;
    }
  };

  const value = {
    account,
    contract,
    reputationContract, 
    usdtContract,
    provider,
    signer, // ✅ Added to exports
    loanInterface, // NEW: Export for error decoding
    loading,
    error,
    chainId,
    member,
    connectionType, 
    connectToLocalNode,
    connectToExternalWallet, 
    connectWithPrivateKey, // ✅ Added to exports
    disconnect, 
    refreshMemberData,
    getUSDTBalance,
    approveUSDT,
    getUSDTInfo,
    needsUSDTApproval,
    switchAccount, 
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