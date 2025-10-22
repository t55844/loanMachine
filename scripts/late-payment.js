const { ethers, network } = require("hardhat");

async function main() {
  const addresses = require("../deployment-addresses.json");
  const [owner, user] = await ethers.getSigners();

  const loanMachine = await ethers.getContractAt("LoanMachine", addresses.loanMachine);

  console.log("🚀 Creating loan requisition...");

  // (amount, minimumCoverage, parcelsCount, memberId)
  const requisitionTx = await loanMachine
    .connect(user)
    .createLoanRequisition(1000, 30, 3, 1);
  await requisitionTx.wait();

  console.log("📄 Loan requisition created");

  console.log("💸 Covering loan...");
  const coverTx = await loanMachine.connect(owner).coverLoan(0, 100, 1);
  await coverTx.wait();

  console.log("⏩ Fast-forwarding 31 days...");
  await network.provider.send("evm_increaseTime", [31 * 24 * 60 * 60]); // 31 days
  await network.provider.send("evm_mine");
  console.log("✅ Time advanced successfully");

  console.log("💰 Repaying loan after due date...");
  const repayTx = await loanMachine.connect(user).repayLoan(0, 1);
  await repayTx.wait();

  console.log("🎉 Loan repaid after due date (late payment simulated)");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
