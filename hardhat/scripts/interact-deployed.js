const { ethers } = require("hardhat");

async function main() {
  console.log("🚀 Starting LoanMachine cover loan test script with authorization...");

  // --- Contract Addresses ---
  const loanMachineAddress = "0xE797948c05aa26369825bA03D2b5e0eBB4ed28C1";
  const reputationSystemAddress = "0xf9B64b3242DDFc7627cd764825617e6d9310Ce95";

  const signers = await ethers.getSigners();
  const [user] = signers;
  console.log(`📱 Using wallet: ${user.address}`);

  // --- Connect to Contracts ---
  console.log("\n--- Connecting to contracts ---");
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = LoanMachine.attach(loanMachineAddress);

  const ReputationSystem = await ethers.getContractFactory("ReputationSystem");
  const reputationSystem = ReputationSystem.attach(reputationSystemAddress);

  console.log("✅ Connected to contracts");

  // --- Authorization Logic (Merged: Check & Set if Possible) ---
  console.log("\n--- Checking/Setting authorization ---");
  try {
    // Get owner for verification
    const owner = await reputationSystem.owner();
    console.log(`Contract owner: ${owner}`);

    const isSignerOwner = owner.toLowerCase() === user.address.toLowerCase();
    if (!isSignerOwner) {
      console.log(`⚠️ Signer ${user.address} is not owner. Skipping authorization (use owner wallet if needed).`);
    }

    // Check current auth status
    const isAuthorized = await reputationSystem.authorizedCallers(loanMachineAddress);
    if (isAuthorized) {
      console.log("✅ LoanMachine already authorized");
    } else {
      if (!isSignerOwner) {
        console.error("❌ LoanMachine not authorized and signer is not owner. Cannot authorize. Run with owner wallet.");
        return;
      }

      console.log("Authorizing LoanMachine...");
      const tx = await reputationSystem.setAuthorizedCaller(loanMachineAddress, true);
      const receipt = await tx.wait();
      console.log(`✅ Authorization successful! Tx hash: ${receipt.hash}`);
    }
  } catch (err) {
    console.error("❌ Authorization failed:", err.message);
    return;
  }
/*
  // --- Cover Loan Parameters ---
  const requisitionId = 2;
  const coveragePercentage = 10;
  const memberId = 373272171;

  // --- Cover Loan ---
  console.log("\n--- Covering loan ---");
  try {
    console.log(`Attempting to cover ${coveragePercentage}% of requisition #${requisitionId} with member ${memberId}...`);
    const tx = await loanMachine.coverLoan(requisitionId, coveragePercentage, memberId);
    const receipt = await tx.wait();
    console.log("✅ Cover loan successful! Tx hash:", receipt.hash);
    
    // Check post-cover state
    const info = await loanMachine.getRequisitionInfo(requisitionId);
    console.log(`Post-cover: Current coverage = ${info.currentCoverage.toString()}%`);
  } catch (err) {
    console.error("❌ Cover loan failed:", err.message);
    if (err.reason) {
      console.error("Revert reason:", err.reason);
    }
    if (err.data) {
      console.error("Revert data:", err.data);
    }
    if (err.code === 'UNPREDICTABLE_GAS_LIMIT') {
      console.log("Confirmed: Gas estimation failed due to revert.");
    }
    console.log("This isolates the contract-side issue.");
  }
*/
  console.log("\n✅ Script completed!");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});