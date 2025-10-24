const { ethers, network } = require("hardhat");
const fs = require("fs");

async function main() {
  const addresses = require("../deployment-addresses.json");
  const [owner, user1, user2, user3, user4] = await ethers.getSigners();

  console.log("🔍 Connecting to deployed contracts...");

  // Load contract instance
  const reputationSystem = await ethers.getContractAt("ReputationSystem", addresses.reputationSystem);


  // ============ STEP 3: Simulate Late Payment ============
  console.log("⏩ Step 3: Advancing time by 61 days...");
  await network.provider.send("evm_increaseTime", [61 * 24 * 60 * 60]); // +91 days
  await network.provider.send("evm_mine");
  console.log("✅ Time advanced successfully.");


}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("💥 Script failed:", error);
    process.exit(1);
  });