// Updated Web3Context.jsx - Fetch config from proxy, use proxy for RPC provider
import { createContext, useContext, useState, useEffect, useRef } from 'react'; // NEW: Added useRef
import { ethers } from 'ethers';
import LoanMachineABI from '../src/abi/LoanMachine.json';
import ReputationSystemABI from '../src/abi/ReputationSystem.json';
import { fetchWalletMember, fetchMemberReputation } from './graphql-frontend-query';

let PROXY_URL = import.meta.env.VITE_PROXY_URL;
const hostname = window.location.hostname.trim().toLowerCase(); 


if (typeof hostname !== 'undefined') {
    const isLocal = hostname === 'localhost' 

    PROXY_URL = isLocal 
      ? 'http://localhost:3001/api/proxy' 
      : 'http://proxy:3001/api/proxy';
  } else {
    // SSR/build fallback
    PROXY_URL = 'http://localhost:3001/api/proxy';
  }


const USD_ABI = [
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

const Web3Context = createContext();

// Web3Provider: This is the main React context provider component that wraps the app, managing Web3 state including account, contracts, provider, and connection logic. It fetches config on mount, handles auto-reconnection, and provides functions for connecting/disconnecting wallets and interacting with USD.
export function Web3Provider({ children }) {
  const [account, setAccount] = useState(null);
  const [contract, setContract] = useState(null);
  const [reputationContract, setReputationContract] = useState(null);
  const [usdContract, setusdContract] = useState(null);
  const [provider, setProvider] = useState(null);
  const [signer, setSigner] = useState(null); // ✅ Added for explicit signer access
  const [loanInterface, setLoanInterface] = useState(null); // NEW: Interface for error decoding
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [chainId, setChainId] = useState(null);
  const [member, setMember] = useState(null);
  const [connectionType, setConnectionType] = useState(null);
  const [config, setConfig] = useState(null); // Store proxy-fetched config

  // NEW: Refs to store listener handlers (prevents repeated additions and allows removal)
  const accountsChangedHandlerRef = useRef(null);
  const chainChangedHandlerRef = useRef(null);

  // Fetch config from proxy on mount
  useEffect(() => {
    // fetchConfig: Asynchronously fetches configuration data (like contract addresses) from a proxy server. It validates the response for required fields and sets the config state or handles errors.
    const fetchConfig = async () => {
      try {
        const response = await fetch(`${PROXY_URL}?type=config`);
        if (!response.ok) throw new Error('Failed to fetch config');
        const data = await response.json();
        //  Check all required addresses
        if (!data.contractAddress || !data.reputationContractAddress || !data.mockUsdAddress) {
          throw new Error('Invalid config from server: missing required contract addresses.');
        }
        setConfig(data);
      } catch (err) {
        setError(`Config fetch failed: ${err.message}`);
      }
    };
    fetchConfig();
  }, []);

  // OPTIONAL: If you need to set a default provider early (before any connection), you could add this useEffect.
  // But it's not necessary—your connection functions already handle it correctly.
  // useEffect(() => {
  //   if (config) { // Wait for config to load (though not used here)
  //     const customProvider = new ethers.providers.JsonRpcProvider(`${PROXY_URL}?type=rpc`); // Proxy RPC calls
  //     setProvider(customProvider);
  //   }
  // }, [config]);

  // fetchAndSetMemberData: Fetches member data from a subgraph using GraphQL queries for a given wallet address. It checks for vinculation (association), retrieves reputation, and sets member state with detailed data or defaults if no member is found. Handles errors gracefully by setting fallback data.
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

  // ✅ FIXED: Auto-reconnect logic - Now waits for config to load before attempting reconnect
  useEffect(() => {
    const savedType = localStorage.getItem('connectedWalletType');
    const savedAccount = localStorage.getItem('connectedWalletAddress');
    const savedPK = localStorage.getItem('demoPrivateKey');
    if (config && savedType && savedAccount && !account) {
      setLoading(true);
      if (savedType === 'local') connectToLocalNode(savedAccount);
      else if (savedType === 'external') connectToExternalWallet(savedAccount);
      else if (savedType === 'demo' && savedPK) connectWithPrivateKey(savedPK);
    } else if (!savedType) {
      setLoading(false);
    }
  }, [account, config]); // ✅ Added config to deps

  // setupContracts: Initializes Ethereum contracts (LoanMachine, ReputationSystem, USD) using the provided provider and signer. Validates config and chain ID (commented out), sets up interfaces for error decoding, authorizes the LoanMachine if the signer is the owner, fetches member data, and updates all relevant states. Handles USD connection warnings for non-local networks.
  const setupContracts = async (newProvider, newSigner, newAccount, newChainId, type) => {
    if (!config) {
      setError('Configuration loading... Please wait and try again.');
      setLoading(false);
      return; // Early return - retry connection after config loads
    }
    // ✅ ADDED: Double-check addresses (defense in depth)
    if (!config.contractAddress || !config.reputationContractAddress || !config.mockUsdAddress) {
      setError('Incomplete configuration: missing contract addresses. Please refresh and try again.');
      setLoading(false);
      return;
    }
   /* const expectedChainId = process.env.EXPECTED_CHAIN_ID; // Keep if needed
console.log('Setting up contracts with config:', config.reputationContractAddress);
console.log('Setting up contracts with config:', expectedChainId);
    if (expectedChainId && parseInt(expectedChainId) !== parseInt(newChainId)) {
      setError(`Wrong network. Please switch to chain ID ${expectedChainId}`);
      setLoading(false);
      return;
    } */
    // Now safe - config exists and addresses are present
    const loanContract = new ethers.Contract(config.contractAddress, LoanMachineABI.abi, newSigner);
    const reputationSystemContract = new ethers.Contract(
      config.reputationContractAddress,
      ReputationSystemABI.abi,
      newSigner
    ) // NEW: Create Interface for LoanMachine (for error decoding)
    const loanInterface = new ethers.utils.Interface(LoanMachineABI.abi);
    const usdAddress = config.mockUsdAddress;
    const usdTokenContract = new ethers.Contract(usdAddress, USD_ABI, newSigner);
    try {
      await usdTokenContract.symbol();
    } catch (testError) {
      console.warn(`⚠️ Cannot connect to MockUSD at ${usdAddress}. Expected if not on local Hardhat network.`);
      if(type === 'external' || type === 'demo') {
        // ✅ Allow demo/external to proceed with warning
        setError('MockUSD contract not found on this network. Faucet will be disabled.');
      } else {
        throw new Error(`USD contract not working at ${usdAddress}. Please check deployment.`);
      }
    }
    // maybeAuthorizeLoanMachine: Checks if the current signer is the owner of the ReputationSystem contract. If so, verifies if LoanMachine is authorized as a caller and authorizes it if not. This is a one-time setup to allow LoanMachine to interact with ReputationSystem. Errors are logged but do not interrupt the flow.
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
        const isAuthorized = await reputationSystemContract.authorizedCallers(config.contractAddress);
        if (isAuthorized) {
          //console.log('LoanMachine already authorized.');
          return;
        }
        //console.log('Authorizing LoanMachine as caller...');
        const tx = await reputationSystemContract.setAuthorizedCaller(config.contractAddress, true);
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
    setusdContract(usdTokenContract);
    setLoanInterface(loanInterface); // NEW: Set interface
    setAccount(newAccount);
    setChainId(newChainId);
    setConnectionType(type);
    // Fetch member data after setting up contracts
    await fetchAndSetMemberData(newAccount);
    setLoading(false);
  };

  // connectToExternalWallet: Connects to an external wallet provider like MetaMask. Requests accounts, sets up a Web3Provider, gets the signer and network, then calls setupContracts. Listens for account/chain changes to reload or disconnect. Handles errors like no provider detected.
  const connectToExternalWallet = async (preferredAccount = null) => {
    setLoading(true);
    setError('');
    if (!window.ethereum) {
      setError('No external wallet provider detected.');
      setLoading(false);
      return;
    }
    try {
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      });
      const externalProvider = new ethers.providers.Web3Provider(window.ethereum, 'any'); // Allow any chain
      await externalProvider.ready; // Wait for provider
      const signer = externalProvider.getSigner();
      const network = await externalProvider.getNetwork();
      const defaultAccount = preferredAccount && accounts.includes(preferredAccount) ? preferredAccount : accounts[0];
      if (!defaultAccount) {
        throw new Error('No authorized account found from wallet provider.');
      }
      await setupContracts(externalProvider, signer, defaultAccount, network.chainId, 'external');

      // NEW: Define and store handlers (only add if not already set)
      if (!accountsChangedHandlerRef.current) {
        accountsChangedHandlerRef.current = (newAccounts) => {
          if (newAccounts.length > 0) window.location.reload();
          else disconnect();
        };
        window.ethereum.on('accountsChanged', accountsChangedHandlerRef.current);
      }

      if (!chainChangedHandlerRef.current) {
        chainChangedHandlerRef.current = () => window.location.reload();
        window.ethereum.on('chainChanged', chainChangedHandlerRef.current);
      }
    } catch (err) {
      //console.error('Error connecting to external wallet:', err);
      setError(`Failed to connect: ${err.message}`);
      setLoading(false);
    }
  };

  // connectToLocalNode: Connects to a local Hardhat node via a proxy RPC endpoint. Lists available accounts, selects a preferred or default one, gets the signer, and calls setupContracts. Clears local storage on failure to prevent retry loops.
  const connectToLocalNode = async (preferredAccount = null) => {
    setLoading(true);
    setError('');
    try {
      // ✅ ENHANCED: Add timeout or better error handling for proxy/RPC
      // THIS IS WHERE THE PROXY IS ALREADY USED FOR THE PROVIDER (as suggested)
      const localProvider = new ethers.providers.JsonRpcProvider(`${PROXY_URL}?type=rpc`); // Use proxy for RPC
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
      setError(`Failed to connect to local node: ${err.message}. Ensure Hardhat node is running on localhost:8545 and proxy is configured.`);
      setLoading(false);
      // ✅ Clear local storage on failure to prevent repeated attempts
      localStorage.removeItem('connectedWalletType');
      localStorage.removeItem('connectedWalletAddress');
    }
  };

  // connectWithPrivateKey: Creates a wallet from a provided private key, connects it to the proxy RPC provider, derives the address, and calls setupContracts for demo mode. Persists the key and address in local storage (noted as insecure for testnet only). Handles validation and errors.
  const connectWithPrivateKey = async (privateKey) => {
    setLoading(true);
    setError('');
    try {
      // THIS IS WHERE THE PROXY IS ALREADY USED FOR THE PROVIDER (as suggested)
      const demoProvider = new ethers.providers.JsonRpcProvider(`${PROXY_URL}?type=rpc`); // Use proxy
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

  // switchAccount: Switches to a different account in local node mode only. Lists accounts, validates the index, gets a new signer, and re-runs setupContracts with the new account. Saves the new account to local storage for auto-reconnect.
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

  // disconnect: Clears all Web3-related states (account, contracts, provider, etc.), removes stored demo private key from local storage, and resets loading/error states to disconnect the wallet.
  const disconnect = () => {
    // Remove listeners if they exist
    if (accountsChangedHandlerRef.current && window.ethereum) {
      window.ethereum.removeListener('accountsChanged', accountsChangedHandlerRef.current);
      accountsChangedHandlerRef.current = null;
    }
    if (chainChangedHandlerRef.current && window.ethereum) {
      window.ethereum.removeListener('chainChanged', chainChangedHandlerRef.current);
      chainChangedHandlerRef.current = null;
    }

    setAccount(null);
    setContract(null);
    setReputationContract(null);
    setusdContract(null);
    setProvider(null);
    setSigner(null); // ✅ Clear signer
    setLoanInterface(null); // NEW: Clear interface
    setMember(null);
    setLoading(false);
    setError('');
    setConnectionType(null);
    localStorage.removeItem('demoPrivateKey'); // ✅ Clear demo PK
  };

  // Cleanup on unmount (for safety)
  useEffect(() => {
    return () => {
      disconnect(); // Removes listeners on context unmount
    };
  }, []);

  // refreshMemberData: Refreshes the member data by calling fetchAndSetMemberData if an account is connected. Returns a promise resolving to the member data or null if no account.
  const refreshMemberData = () => {
    if (account) {
      return fetchAndSetMemberData(account);
    }
    return Promise.resolve(null);
  };

  // getUSDBalance: Retrieves the USD balance for the given address (or current account). Formats the balance from wei to human-readable units (assuming 6 decimals). Returns '0' on error.
  const getUSDBalance = async (address = null) => {
    if (!usdContract) throw new Error('USD contract not initialized');
    try {
      const targetAddress = address || account;
      const balance = await usdContract.balanceOf(targetAddress);
      return ethers.utils.formatUnits(balance, 6);
    } catch (err) {
      //console.warn("Could not fetch USD balance", err.message);
      return '0';
    }
  };

  // approveUSD: Approves the LoanMachine contract to spend a specified amount of USD on behalf of the user. Converts amount to wei (6 decimals) and sends the approval transaction.
  const approveUSD = async (amount) => {
    if (!usdContract || !contract) throw new Error('Contracts not initialized');
    const amountInWei = ethers.utils.parseUnits(amount.toString(), 6);
    const tx = await usdContract.approve(contract.address, amountInWei);
    return tx;
  };

  // getUSDInfo: Fetches USD contract metadata like name, symbol, decimals, and address. Returns defaults on error for fallback.
  const getUSDInfo = async () => {
    if (!usdContract) throw new Error('USD contract not initialized');
    try {
      const [name, symbol, decimals] = await Promise.all([
        usdContract.name(),
        usdContract.symbol(),
        usdContract.decimals()
      ]);
      return {
        name,
        symbol,
        decimals,
        address: usdContract.address
      };
    } catch(err) {
      //console.warn("Could not fetch USD info", err.message);
      return {
        name: 'MockUSD',
        symbol: 'mUSD',
        decimals: 6,
        address: config.mockUsdAddress
      };
    }
  };

  // needsUSDApproval: Checks if the current allowance for the LoanMachine contract is less than the specified amount. Returns true if approval is needed, or on error (conservative default).
  const needsUSDApproval = async (amount) => {
    if (!usdContract || !contract) return true;
    try {
      const currentAllowance = await usdContract.allowance(account, contract.address);
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
    usdContract,
    provider,
    signer,
    loanInterface,
    config,
    loading,
    error,
    chainId,
    member,
    connectionType,
    connectToLocalNode,
    connectToExternalWallet,
    connectWithPrivateKey,
    disconnect,
    refreshMemberData,
    getUSDBalance,
    approveUSD,
    getUSDInfo,
    needsUSDApproval,
    switchAccount,
  };

  return (
    <Web3Context.Provider value={value}>
      {children}
    </Web3Context.Provider>
  );
}

// useWeb3: A custom React hook that retrieves the Web3 context. Throws an error if used outside the Web3Provider, ensuring proper context usage.
export function useWeb3() {
  const context = useContext(Web3Context);
  if (!context) {
    throw new Error('useWeb3 must be used within a Web3Provider');
  }
  return context;
}