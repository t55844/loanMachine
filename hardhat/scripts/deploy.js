const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Deploying merged LoanMachine...");

  const [owner] = await ethers.getSigners();

  // MockUSDT
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  console.log("MockUSDT →", await mockUSDT.getAddress());

  // LoanMachine (agora com Reputation dentro)
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.deploy(await mockUSDT.getAddress());
  await loanMachine.waitForDeployment();

  const addr = await loanMachine.getAddress();
  console.log("LoanMachine (merged) →", addr);

  console.log("✅ Tudo pronto!");
}

main().catch((error) => {
  console.error(error);
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