const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Starting deployment with USDT distribution...");

  const [owner, user1, user2, user3, user4] = await ethers.getSigners();

  // First, deploy the DebtTracker library
  console.log("📚 Deploying DebtTracker library...");
  const DebtTracker = await ethers.getContractFactory("DebtTracker");
  const debtTracker = await DebtTracker.deploy();
  await debtTracker.waitForDeployment();
  const debtTrackerAddress = await debtTracker.getAddress();
  console.log("DebtTracker library deployed to:", debtTrackerAddress);

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

  // --- Deploy LoanMachine with library linking ---
  console.log("🤖 Deploying LoanMachine with library linking...");
  const LoanMachine = await ethers.getContractFactory("LoanMachine", {
    libraries: {
      "DebtTracker": debtTrackerAddress
    }
  });
  
  const loanMachine = await LoanMachine.deploy(mockUSDTAddress, reputationSystemAddress);
  await loanMachine.waitForDeployment();
  const loanMachineAddress = await loanMachine.getAddress();
  console.log("LoanMachine deployed to:", loanMachineAddress);

  // --- Set authorized caller for ReputationSystem ---
  console.log("🔐 Setting authorized caller for ReputationSystem...");
  await reputationSystem.setAuthorizedCaller(loanMachineAddress, true);
  console.log("✅ LoanMachine authorized in ReputationSystem");

  // --- Mint USDT to all wallets ---
  console.log("🎁 Distributing USDT to wallets...");
  const amount = ethers.parseUnits("20", 6); // 20 USDT with 6 decimals
  const users = [owner, user1, user2, user3, user4];

  for (const user of users) {
    const tx = await mockUSDT.mint(user.address, amount);
    await tx.wait();
    console.log(`✅ Minted ${ethers.formatUnits(amount, 6)} USDT to ${user.address}`);
  }

  // --- Show balances ---
  console.log("\n💰 Final USDT Balances:");
  for (const user of users) {
    const bal = await mockUSDT.balanceOf(user.address);
    console.log(`   ${user.address}: ${ethers.formatUnits(bal, 6)} USDT`);
  }


  console.log("\n✅ All contracts deployed and initialized successfully!");
  console.log("📋 Contract addresses:");
  console.log(`   DebtTracker Library: ${debtTrackerAddress}`);
  console.log(`   MockUSDT: ${mockUSDTAddress}`);
  console.log(`   ReputationSystem: ${reputationSystemAddress}`);
  console.log(`   LoanMachine: ${loanMachineAddress}`);

  // Save deployment addresses to a file
  const addresses = {
    debtTracker: debtTrackerAddress,
    mockUSDT: mockUSDTAddress,
    reputationSystem: reputationSystemAddress,
    loanMachine: loanMachineAddress,
    users: {
      owner: owner.address,
      user1: user1.address,
      user2: user2.address,
      user3: user3.address,
      user4: user4.address
    }
  };
  
  const fs = require('fs');
  fs.writeFileSync('deployment-addresses.json', JSON.stringify(addresses, null, 2));

  console.log("\n🎉 Deployment and setup completed successfully!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("Deployment failed:", error);
    process.exit(1);
  });