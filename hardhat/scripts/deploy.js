const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Starting deployment with USDT distribution...");

  const signers = await ethers.getSigners();
  const [owner] = signers;
  console.log(`📱 Found ${signers.length} wallets`);

  // --- Deploy MockUSDT ---
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  const mockUSDTAddress = await mockUSDT.getAddress();
  console.log("MockUSDT deployed to:", mockUSDTAddress);

  // --- Deploy ReputationSystem ---
  const ReputationSystem = await ethers.getContractFactory("ReputationSystem");
  const reputationSystem = await ReputationSystem.deploy();
  await reputationSystem.waitForDeployment();
  const reputationSystemAddress = await reputationSystem.getAddress();
  console.log("ReputationSystem deployed to:", reputationSystemAddress);

  // --- Deploy LoanMachine ---
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.deploy(mockUSDTAddress, reputationSystemAddress);
  await loanMachine.waitForDeployment();
  const loanMachineAddress = await loanMachine.getAddress();
  console.log("LoanMachine deployed to:", loanMachineAddress);

  console.log("✅ Deployment completed!");
  console.log({ mockUSDTAddress, reputationSystemAddress, loanMachineAddress });
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});


/**PS C:\codigos\loan-machine\hardhat> npx hardhat run scripts/deploy.js --network sepolia
🚀 Starting deployment with USDT distribution...
📱 Found 1 wallets
MockUSDT deployed to: 0x2107997bd769396b1B1f05A872f4e0a2BF16d54A
ReputationSystem deployed to: 0xf9B64b3242DDFc7627cd764825617e6d9310Ce95
LoanMachine deployed to: 0xE797948c05aa26369825bA03D2b5e0eBB4ed28C1
✅ Deployment completed!
{
  mockUSDTAddress: '0x2107997bd769396b1B1f05A872f4e0a2BF16d54A',
  reputationSystemAddress: '0xf9B64b3242DDFc7627cd764825617e6d9310Ce95',
  loanMachineAddress: '0xE797948c05aa26369825bA03D2b5e0eBB4ed28C1'
}
PS C:\codigos\loan-machine\hardhat>  */