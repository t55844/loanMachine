const { ethers } = require("hardhat");

async function testEvent() {
  const [deployer, user1,user2,user3,user4] = await ethers.getSigners();
  
  // Load deployment addresses
  const addresses = require("../deployment-addresses.json");
  
  // Connect to contracts
  const loanMachine = await ethers.getContractAt("LoanMachine", addresses.loanMachine);
  const reputationSystem = await ethers.getContractAt("ReputationSystem", addresses.reputationSystem);

  console.log("Testing with user4:", user3.address);

  try {
    // Check if user4 is registered
    const memberId = await loanMachine.getMemberId(user3.address);
    console.log("user4 member ID:", memberId.toString());
    
    // Get current reputation
    const reputation = await loanMachine.getReputation(memberId);
    console.log("Current reputation:", reputation.toString());

    // Listen for ReputationChanged event from ReputationSystem
    reputationSystem.on("ReputationChanged", (memberId, points, increase, newReputation, timestamp) => {
      console.log("✅ REPUTATION CHANGED EVENT:");
      console.log("   Member ID:", memberId.toString());
      console.log("   Points:", points.toString());
      console.log("   Increase:", increase);
      console.log("   New Reputation:", newReputation.toString());
    });

    // Call a function that might change reputation
    console.log("Calling getReputation...");
    const tx = await loanMachine.connect(user3).getReputation(memberId);
    await tx.wait();
    
    console.log("Transaction completed");

    // Wait a bit for event
    await new Promise(resolve => setTimeout(resolve, 3000));
    
  } catch (error) {
    console.log("Error:", error.message);
  }

  // Cleanup and exit
  reputationSystem.removeAllListeners();
  process.exit(0);
}

testEvent();