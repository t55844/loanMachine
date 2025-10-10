const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Deploying contracts...");

  const [owner, user1, user2, user3, user4] = await ethers.getSigners();

  // --- Deploy MockUSDT ---
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  console.log("MockUSDT deployed to:", await mockUSDT.getAddress());

  // --- Deploy ReputationSystem ---
  const ReputationSystem = await ethers.getContractFactory("ReputationSystem");
  const reputationSystem = await ReputationSystem.deploy();
  await reputationSystem.waitForDeployment();
  console.log("ReputationSystem deployed to:", await reputationSystem.getAddress());

  // --- Deploy LoanMachine ---
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.deploy(
    await mockUSDT.getAddress(),
    await reputationSystem.getAddress()
  );
  await loanMachine.waitForDeployment();
  console.log("LoanMachine deployed to:", await loanMachine.getAddress());

  // After deploying both contracts:
  await reputationSystem.setAuthorizedCaller(loanMachine.address, true);

  // --- Mint same amount of USDT to each wallet ---
  const amount = ethers.parseUnits("10", 6); // 100,000 USDT with 6 decimals
  const users = [owner, user1, user2, user3, user4];

  for (const user of users) {
    const tx = await mockUSDT.mint(user.address, amount);
    await tx.wait();
  }

  // --- Show balances ---
  for (const user of users) {
    const bal = await mockUSDT.balanceOf(user.address);
    console.log(`Balance of ${user.address}: ${ethers.formatUnits(bal, 6)} USDT`);
  }

  console.log("✅ All contracts deployed successfully!");
  console.log("📋 Contract addresses:");
  console.log(`   MockUSDT: ${await mockUSDT.getAddress()}`);
  console.log(`   ReputationSystem: ${await reputationSystem.getAddress()}`);
  console.log(`   LoanMachine: ${await loanMachine.getAddress()}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});