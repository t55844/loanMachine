// scripts/test-event.js
const { ethers } = require("hardhat");

async function testEvent() {
  const [deployer, user1] = await ethers.getSigners();
  
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.attach("0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9");

  console.log("Testing MemberToWalletVinculation event...");
  
  // Listen for the event
  loanMachine.on("MemberToWalletVinculation", (memberId, wallet, timestamp) => {
    console.log("✅ EVENT DETECTED:");
    console.log("   Member ID:", memberId.toString());
    console.log("   Wallet:", wallet);
    console.log("   Timestamp:", timestamp.toString());
  });

  // Trigger the function that should emit the event
  try {
    const tx = await loanMachine.connect(user1).vinculationMemberToWallet();
    console.log("Transaction sent:", tx.hash);
    
    const receipt = await tx.wait();
    console.log("Transaction confirmed in block:", receipt.blockNumber);
    
    // Check if event was emitted
    if (receipt.events && receipt.events.length > 0) {
      console.log("Events in receipt:", receipt.events.map(e => e.event));
    } else {
      console.log("❌ NO EVENTS IN RECEIPT");
    }
    
  } catch (error) {
    console.log("Error:", error.message);
  }
}

testEvent();