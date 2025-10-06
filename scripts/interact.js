const hre = require("hardhat");
const CONTRACT_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512';

// Get contract instance
async function getContract(signer = null) {
  const LoanMachine = await hre.ethers.getContractFactory("LoanMachine");
  const contract = await LoanMachine.attach(CONTRACT_ADDRESS);
  return signer ? contract.connect(signer) : contract;
}

// Get USDT contract
async function getUSDTContract(signer = null) {
  const contract = await getContract();
  const usdtAddress = await contract.usdtToken();
  const USDT = await hre.ethers.getContractFactory("IERC20");
  const usdtContract = await USDT.attach(usdtAddress);
  return signer ? usdtContract.connect(signer) : usdtContract;
}

// Approve USDT spending
async function approveUSDT(signer) {
  try {
    const usdtContract = await getUSDTContract(signer);
    const amount = 4000000; // 4 USDT
    
    console.log(`Approving 4 USDT for contract...`);
    
    const tx = await usdtContract.approve(CONTRACT_ADDRESS, amount);
    await tx.wait();
    
    console.log(`✅ Approved 4 USDT`);
    return true;
  } catch (error) {
    console.error('❌ Approval failed:', error.message);
    return false;
  }
}

// Vinulate member to wallet
async function vinulateMember(memberId, signer) {
  try {
    const contract = await getContract(signer);
    console.log(`Vinulating member ${memberId}...`);
    
    const tx = await contract.vinculationMemberToWallet(memberId, signer.address);
    await tx.wait();
    
    console.log(`✅ Member ${memberId} vinculated`);
    return true;
  } catch (error) {
    console.error('❌ Vinulation failed:', error.message);
    return false;
  }
}

// Make donation
async function donate(signer) {
  try {
    const contract = await getContract(signer);
    const donationAmount = 4000000; // 4 USDT (6 decimals)
    
    console.log(`Donating 4 USDT...`);
    
    const tx = await contract.donate(donationAmount);
    await tx.wait();
    
    console.log(`✅ Donated 4 USDT`);
    return true;
  } catch (error) {
    console.error('❌ Donation failed:', error.message);
    return false;
  }
}

// Cover loan
async function coverLoan(requisitionId, coveragePercentage, signer) {
  try {
    const contract = await getContract(signer);
    
    console.log(`Covering loan ${requisitionId} with ${coveragePercentage}%...`);
    
    const tx = await contract.coverLoan(requisitionId, coveragePercentage);
    await tx.wait();
    
    console.log(`✅ Covered loan ${requisitionId}`);
    return true;
  } catch (error) {
    console.error('❌ Coverage failed:', error.message);
    return false;
  }
}

// Check USDT balance
async function checkUSDTBalance(user) {
  const usdtContract = await getUSDTContract();
  const balance = await usdtContract.balanceOf(user.address);
  console.log(`USDT Balance: ${balance} (${hre.ethers.formatUnits(balance, 6)} USDT)`);
  return balance;
}

// Check donation balance
async function checkDonationBalance(user) {
  const contract = await getContract();
  const donationBalance = await contract.getDonation(user.address);
  console.log(`Donation Balance: ${donationBalance} (${hre.ethers.formatUnits(donationBalance, 6)} USDT)`);
  return donationBalance;
}

// Check allowance
async function checkAllowance(user) {
  const usdtContract = await getUSDTContract();
  const allowance = await usdtContract.allowance(user.address, CONTRACT_ADDRESS);
  console.log(`Allowance: ${allowance} (${hre.ethers.formatUnits(allowance, 6)} USDT)`);
  return allowance;
}

// Main workflow
async function main() {
  const [owner, user1, user2, user3, user4] = await hre.ethers.getSigners();
  const user = user3; // Using user3
  const memberId = 123;
  const requisitionId = 0;
  const coveragePercentage = 25;

  console.log('🚀 Starting cover loan workflow...\n');

  // 1. Check USDT balance
  console.log('1. Checking USDT balance...');
  await checkUSDTBalance(user);

  // 2. Check current allowance
  console.log('2. Checking allowance...');
  await checkAllowance(user);

  // 3. Approve USDT
  console.log('3. Approving USDT...');
  const approved = await approveUSDT(user);
  if (!approved) return;

  // 4. Donate 4 USDT
  console.log('4. Making donation...');
  const donated = await donate(user);
  if (!donated) return;

  // 5. Check donation balance
  console.log('5. Checking donation balance...');
  await checkDonationBalance(user);

  // 6. Vinulate member
  console.log('6. Vinulating member...');
  const vinculated = await vinulateMember(memberId, user);
  if (!vinculated) return;

  // 7. Cover loan
  console.log('7. Covering loan...');
  const covered = await coverLoan(requisitionId, coveragePercentage, user);
  if (!covered) return;

  // 8. Check final donation balance
  console.log('8. Final donation balance...');
  await checkDonationBalance(user);

  console.log('\n🎉 Workflow completed!');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});