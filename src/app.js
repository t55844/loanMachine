let provider;
let signer;
let contract;
let userAddress;

// Contract ABI (Application Binary Interface)
const contractABI = [
    "function donate() external payable",
    "function borrow(uint256 _amount) external",
    "function repay() external payable",
    "function donations(address) external view returns (uint256)",
    "function borrowings(address) external view returns (uint256)",
    "function totalDonations() external view returns (uint256)",
    "function totalBorrowed() external view returns (uint256)",
    "function availableBalance() external view returns (uint256)",
    "function getContractBalance() external view returns (uint256)",
    "event Donated(address indexed donor, uint256 amount)",
    "event Borrowed(address indexed borrower, uint256 amount)",
    "event Repaid(address indexed borrower, uint256 amount)"
];

// Contract address (you'll replace this after deployment)
const contractAddress = "YOUR_CONTRACT_ADDRESS_HERE";

// Initialize when page loads
window.addEventListener('load', async () => {
    // Check if MetaMask is installed
    if (typeof window.ethereum !== 'undefined') {
        provider = new ethers.providers.Web3Provider(window.ethereum);
        await connectWallet();
    } else {
        showStatus('Please install MetaMask!', 'error');
    }

    setupEventListeners();
});

// Connect to MetaMask
async function connectWallet() {
    try {
        await window.ethereum.request({ method: 'eth_requestAccounts' });
        signer = provider.getSigner();
        userAddress = await signer.getAddress();
        
        // Initialize contract
        contract = new ethers.Contract(contractAddress, contractABI, signer);
        
        // Update UI
        document.getElementById('walletAddress').textContent = 
            userAddress.substring(0, 6) + '...' + userAddress.substring(38);
        
        const balance = await provider.getBalance(userAddress);
        document.getElementById('ethBalance').textContent = 
            ethers.utils.formatEther(balance);
        
        document.getElementById('walletInfo').style.display = 'block';
        document.getElementById('connectWallet').style.display = 'none';
        
        await refreshContractData();
        
    } catch (error) {
        showStatus('Error connecting wallet: ' + error.message, 'error');
    }
}

// Setup button listeners
function setupEventListeners() {
    document.getElementById('connectWallet').addEventListener('click', connectWallet);
    document.getElementById('donateBtn').addEventListener('click', donate);
    document.getElementById('borrowBtn').addEventListener('click', borrow);
    document.getElementById('repayBtn').addEventListener('click', repay);
    document.getElementById('refreshBtn').addEventListener('click', refreshContractData);
}

// Donate function
async function donate() {
    try {
        const amount = document.getElementById('donationAmount').value;
        if (!amount || amount <= 0) {
            showStatus('Please enter a valid amount', 'error');
            return;
        }

        showStatus('Processing donation...', 'loading');
        
        const tx = await contract.donate({
            value: ethers.utils.parseEther(amount)
        });
        
        showStatus('Transaction sent! Waiting for confirmation...', 'loading');
        
        await tx.wait();
        showStatus('Donation successful!', 'success');
        await refreshContractData();
        
    } catch (error) {
        showStatus('Error: ' + error.message, 'error');
    }
}

// Borrow function
async function borrow() {
    try {
        const amount = document.getElementById('borrowAmount').value;
        if (!amount || amount <= 0) {
            showStatus('Please enter a valid amount', 'error');
            return;
        }

        showStatus('Processing borrow...', 'loading');
        
        const tx = await contract.borrow(ethers.utils.parseEther(amount));
        
        showStatus('Transaction sent! Waiting for confirmation...', 'loading');
        
        await tx.wait();
        showStatus('Borrow successful!', 'success');
        await refreshContractData();
        
    } catch (error) {
        showStatus('Error: ' + error.message, 'error');
    }
}

// Repay function
async function repay() {
    try {
        const amount = document.getElementById('repayAmount').value;
        if (!amount || amount <= 0) {
            showStatus('Please enter a valid amount', 'error');
            return;
        }

        showStatus('Processing repayment...', 'loading');
        
        const tx = await contract.repay({
            value: ethers.utils.parseEther(amount)
        });
        
        showStatus('Transaction sent! Waiting for confirmation...', 'loading');
        
        await tx.wait();
        showStatus('Repayment successful!', 'success');
        await refreshContractData();
        
    } catch (error) {
        showStatus('Error: ' + error.message, 'error');
    }
}

// Refresh contract data
async function refreshContractData() {
    try {
        if (!contract) return;

        const [
            totalDonations,
            totalBorrowed,
            availableBalance,
            userDonations,
            userBorrowings
        ] = await Promise.all([
            contract.totalDonations(),
            contract.totalBorrowed(),
            contract.availableBalance(),
            contract.donations(userAddress),
            contract.borrowings(userAddress)
        ]);

        document.getElementById('totalDonations').textContent = 
            ethers.utils.formatEther(totalDonations);
        document.getElementById('totalBorrowed').textContent = 
            ethers.utils.formatEther(totalBorrowed);
        document.getElementById('availableBalance').textContent = 
            ethers.utils.formatEther(availableBalance);
        document.getElementById('userDonations').textContent = 
            ethers.utils.formatEther(userDonations);
        document.getElementById('userBorrowings').textContent = 
            ethers.utils.formatEther(userBorrowings);
            
    } catch (error) {
        console.error('Error refreshing data:', error);
    }
}

// Show status message
function showStatus(message, type) {
    const statusDiv = document.getElementById('status');
    statusDiv.textContent = message;
    statusDiv.className = type;
    statusDiv.style.display = 'block';
    
    if (type !== 'loading') {
        setTimeout(() => {
            statusDiv.style.display = 'none';
        }, 5000);
    }
}

// Listen to contract events
function listenToEvents() {
    if (!contract) return;

    contract.on('Donated', (donor, amount) => {
        if (donor.toLowerCase() === userAddress.toLowerCase()) {
            showStatus(`Donated ${ethers.utils.formatEther(amount)} ETH!`, 'success');
        }
    });

    contract.on('Borrowed', (borrower, amount) => {
        if (borrower.toLowerCase() === userAddress.toLowerCase()) {
            showStatus(`Borrowed ${ethers.utils.formatEther(amount)} ETH!`, 'success');
        }
    });

    contract.on('Repaid', (borrower, amount) => {
        if (borrower.toLowerCase() === userAddress.toLowerCase()) {
            showStatus(`Repaid ${ethers.utils.formatEther(amount)} ETH!`, 'success');
        }
    });
}