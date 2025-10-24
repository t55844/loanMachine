const hre = require("hardhat");
const fs = require('fs');

async function main() {
  console.log("🧪 Testing Member Events...");

  // Read deployment addresses from file
  let deploymentAddresses;
  try {
    deploymentAddresses = JSON.parse(fs.readFileSync('deployment-addresses.json', 'utf8'));
    console.log("✅ Loaded deployment addresses");
  } catch (error) {
    console.log("❌ Could not read deployment-addresses.json");
    console.log("Please run: npx hardhat run scripts/deploy.js --network localhost");
    process.exit(1);
  }

  const CONTRACT_ADDRESS = deploymentAddresses.loanMachine;

  const [owner, user9, user3] = await hre.ethers.getSigners();
  const testUser = user9;
  const memberId = 456;

  console.log(`🧪 Test User: ${testUser.address}`);
  console.log(`🧪 Member ID: ${memberId}`);
  console.log(`📋 LoanMachine: ${CONTRACT_ADDRESS}`);

  // Get contract instance WITH library linking
  const LoanMachine = await hre.ethers.getContractFactory("LoanMachine");
  
  const contract = await LoanMachine.attach(CONTRACT_ADDRESS);

  // Test 1: Check if wallet is already vinculated
  console.log("\n1. Checking wallet vinculated status...");
  try {
    const isVinculated = await contract.isWalletVinculated(testUser.address);
    console.log(`   Is wallet vinculated: ${isVinculated}`);
    
    if (isVinculated) {
      const currentMemberId = await contract.getMemberId(testUser.address);
      console.log(`   Current member ID: ${currentMemberId}`);
    }
  } catch (error) {
    console.log("   ❌ Check failed:", error.message);
  }

  // Test 2: Vinculate member to wallet
  console.log("\n2. Vinculating member to wallet...");
  try {
    const tx = await contract.connect(testUser).vinculationMemberToWallet(memberId, testUser.address);
    console.log("   📝 Transaction sent...");
    
    const receipt = await tx.wait();
    console.log("   ✅ Transaction confirmed!");
    console.log(`   📦 Block: ${receipt.blockNumber}`);
    console.log(`   🔗 Tx Hash: ${receipt.hash}`);
    
    // Check for MemberToWalletVinculation event
    const eventTopic = hre.ethers.id("MemberToWalletVinculation(uint32,address,uint256)");
    const memberEvents = receipt.logs.filter(log => log.topics[0] === eventTopic);
    
    if (memberEvents.length > 0) {
      console.log("   🎉 MemberToWalletVinculation event emitted!");
      console.log("   📡 This event should be captured by your GraphQL subgraph");
    } else {
      console.log("   ⚠️  No MemberToWalletVinculation event found in logs");
    }
  } catch (error) {
    console.log("   ❌ Vinculation failed:", error.message);
    return;
  }

  // Test 3: Verify vinculation worked
  console.log("\n3. Verifying vinculation...");
  try {
    const isVinculated = await contract.isWalletVinculated(testUser.address);
    console.log(`   Is wallet vinculated: ${isVinculated}`);
    
    const memberIdFromWallet = await contract.getMemberId(testUser.address);
    console.log(`   Member ID from wallet: ${memberIdFromWallet}`);
    
    // Convert BigInt to Number for comparison
    const memberIdFromWalletNum = Number(memberIdFromWallet);
    
    if (memberIdFromWalletNum === memberId) {
      console.log("   ✅ Vinculation verified successfully!");
    } else {
      console.log(`   ❌ Vinculation verification failed! ${memberIdFromWalletNum} !== ${memberId}`);
    }
  } catch (error) {
    console.log("   ❌ Verification failed:", error.message);
  }

  // Test 4: Check reputation
  console.log("\n4. Checking reputation...");
  try {
    const reputation = await contract.getReputation(memberId);
    console.log(`   Reputation: ${reputation}`);
  } catch (error) {
    console.log("   ❌ Reputation check failed:", error.message);
  }

  console.log("\n🎉 Member events test completed!");
  console.log("\n📋 Next steps to check GraphQL:");
  console.log("   1. Check subgraph logs: graph logs <deployment-id>");
  console.log("   2. Query members: Run the test-query.graphql file");
  console.log("   3. Look for the MemberToWalletVinculation event in logs");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("Test failed:", error);
    process.exit(1);
  });