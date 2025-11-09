require('dotenv').config();
const { ethers } = require('ethers');

async function main() {
  const RPC_URL = process.env.RPC_URL || 'https://rpc.sepolia.org';
  const PRIVATE_KEY = process.env.PRIVATE_KEY;
  if (!PRIVATE_KEY) throw new Error('Set PRIVATE_KEY in .env (owner/deployer wallet)');

  const reputationAddress = '0xf9B64b3242DDFc7627cd764825617e6d9310Ce95';
  const loanMachineAddress = '0xE797948c05aa26369825bA03D2b5e0eBB4ed28C1';

  const provider = new ethers.JsonRpcProvider(RPC_URL);
  const wallet = new ethers.Wallet(PRIVATE_KEY, provider);
  console.log(`Using owner wallet: ${wallet.address}`);

  const abi = [
    'function owner() view returns (address)',
    'function authorizedCallers(address) view returns (bool)',
    'function setAuthorizedCaller(address caller, bool authorized) external'
  ];
  const reputation = new ethers.Contract(reputationAddress, abi, wallet);

  // Verify signer is owner
  const owner = await reputation.owner();
  if (owner.toLowerCase() !== wallet.address.toLowerCase()) {
    throw new Error(`Signer ${wallet.address} is not owner (${owner}). Use deployer wallet.`);
  }

  // Check current status
  const isAuth = await reputation.authorizedCallers(loanMachineAddress);
  if (isAuth) {
    console.log('✅ Already authorized');
    return;
  }

  // Authorize
  console.log('Authorizing LoanMachine...');
  const tx = await reputation.setAuthorizedCaller(loanMachineAddress, true);
  const receipt = await tx.wait();
  console.log(`✅ Authorized! Tx: https://sepolia.etherscan.io/tx/${receipt.hash}`);
}

main().catch(console.error);