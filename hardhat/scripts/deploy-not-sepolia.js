const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Starting deployment with USDT distribution...");

  // Get ALL signers (all 20 wallets)
  const signers = await ethers.getSigners();
  const [owner] = signers;

  console.log(`📱 Found ${signers.length} wallets to distribute USDT to`);

  // --- Deploy MockUSDT ---
  console.log("💰 Deploying MockUSDT...");
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  const mockUSDTAddress = await mockUSDT.getAddress();
  console.log("MockUSDT deployed to:", mockUSDTAddress);

  // --- Deploy ReputationSystem ---
  console.log("⭐ Deploying ReputationSystem...");
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

  // --- Set authorized caller for ReputationSystem ---
  console.log("🔐 Setting authorized caller for ReputationSystem...");
  await reputationSystem.setAuthorizedCaller(loanMachineAddress, true);
  console.log("✅ LoanMachine authorized in ReputationSystem");

  // --- Mint USDT to ALL 20 wallets ---
  console.log("🎁 Distributing USDT to ALL wallets...");
  const amount = ethers.parseUnits("200", 6); // 200 USDT with 6 decimals

  for (let i = 0; i < signers.length; i++) {
    const user = signers[i];
    const tx = await mockUSDT.mint(user.address, amount);
    await tx.wait();
    console.log(`✅ [${i + 1}/${signers.length}] Minted ${ethers.formatUnits(amount, 6)} USDT to ${user.address}`);
  }

  // --- Show balances ---
  console.log("\n💰 Final USDT Balances for all wallets:");
  for (let i = 0; i < signers.length; i++) {
    const user = signers[i];
    const bal = await mockUSDT.balanceOf(user.address);
    console.log(`   [${i}] ${user.address}: ${ethers.formatUnits(bal, 6)} USDT`);
  }

  console.log("\n✅ All contracts deployed and initialized successfully!");
  console.log("📋 Contract addresses:");
  console.log(`   MockUSDT: ${mockUSDTAddress}`);
  console.log(`   ReputationSystem: ${reputationSystemAddress}`);
  console.log(`   LoanMachine: ${loanMachineAddress}`);

  // Save deployment addresses to a file
  const addresses = {
    mockUSDT: mockUSDTAddress,
    reputationSystem: reputationSystemAddress,
    loanMachine: loanMachineAddress,
    users: {}
  };

  // Save all wallet addresses
  for (let i = 0; i < signers.length; i++) {
    addresses.users[`wallet${i}`] = signers[i].address;
  }
  
  const fs = require('fs');
  fs.writeFileSync('deployment-addresses.json', JSON.stringify(addresses, null, 2));

  console.log("\n🎉 Deployment and setup completed successfully!");
  console.log(`💰 Distributed USDT to ${signers.length} wallets`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("Deployment failed:", error);
    process.exit(1);
  });