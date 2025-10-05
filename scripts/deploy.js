const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Deploying contracts...");

  const [owner, user1, user2, user3, user4] = await ethers.getSigners();

  // --- Deploy MockUSDT ---
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  console.log("MockUSDT deployed to:", await mockUSDT.getAddress());

  // --- Deploy LoanMachine ---
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.deploy(await mockUSDT.getAddress());
  await loanMachine.waitForDeployment();
  console.log("LoanMachine deployed to:", await loanMachine.getAddress());

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

  console.log("✅ Done!");
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
