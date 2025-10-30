const { ethers, network } = require("hardhat");
const fs = require("fs");

async function main() {
  const addresses = require("../deployment-addresses.json");
  const [owner, user1, user2, user3, user4] = await ethers.getSigners();

  console.log("🔍 Connecting to deployed contracts...");

  // Load contract instance
  const reputationSystem = await ethers.getContractAt("ReputationSystem", addresses.reputationSystem);

  console.log("✅ Connected to reputationSystem at:", addresses.reputationSystem);
  console.log("📡 Fetching historical events...\n");

  // Helper to fetch and print events
  async function fetchEvent(eventName, parser) {
    try {
      // Use the correct event filter method
      const filter = reputationSystem.filters[eventName]();
      const events = await reputationSystem.queryFilter(filter, 0, "latest");
      console.log(`📘 Found ${events.length} ${eventName} events:`);

      const parsed = events.map((e) => parser(e));
      parsed.forEach((p, i) => console.log(`${i + 1}.`, p));

      return parsed;
    } catch (error) {
      console.log(`❌ Error fetching ${eventName}:`, error.message);
      return [];
    }
  }

  // ========== FETCH EVENTS ==========

  // Corrected event parsing for ReputationChanged
  const ReputationChanged = await fetchEvent("ReputationChanged", (e) => ({
    memberId: e.args.memberId.toString(),
    points: e.args.points.toString(),
    increase: e.args.increase,
    newReputation: e.args.newReputation.toString(),
    timestamp: e.args.timestamp.toString(),
    blockNumber: e.blockNumber,
    transactionHash: e.transactionHash
  }));


  // ========== SAVE TO FILE ==========
  const allEvents = {
    ReputationChanged
  };

  fs.writeFileSync("reputationSystem-events.json", JSON.stringify(allEvents, null, 2));
  console.log("\n💾 Saved all events to reputationSystem-events.json");
  console.log("✅ Done!");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("💥 Script failed:", error);
    process.exit(1);
  });