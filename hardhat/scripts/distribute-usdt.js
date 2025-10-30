const { ethers } = require("hardhat");
const fs = require('fs');

async function main() {
  console.log("🎁 Distributing USDT to existing contracts...");

  // Load deployment addresses
  const addresses = JSON.parse(fs.readFileSync('deployment-addresses.json', 'utf8'));
  
  const [owner, user1, user2, user3, user4] = await ethers.getSigners();

  // Get the deployed MockUSDT contract
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = MockUSDT.attach(addresses.mockUSDT);

  // --- Mint USDT to all wallets ---
  console.log("💰 Distributing USDT to wallets...");
  const amount = ethers.parseUnits("10000", 6); // 10,000 USDT with 6 decimals
  const users = await ethers.getSigners();

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

  console.log("✅ USDT distribution completed!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("Distribution failed:", error);
    process.exit(1);
  });