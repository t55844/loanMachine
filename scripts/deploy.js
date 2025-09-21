const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying LoanMachine contract...");
  
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.deploy();
  
  // Wait for deployment to complete (no .deployed() needed)
  await loanMachine.waitForDeployment();
  
  // Get the contract address
  const address = await loanMachine.getAddress();
  console.log("LoanMachine deployed to:", address);
  
  // Verify deployment
  const totalDonations = await loanMachine.getTotalDonations();
  console.log("Initial total donations:", totalDonations.toString());
  
  // Test a few more functions to ensure everything works
  const availableBalance = await loanMachine.getAvailableBalance();
  console.log("Initial available balance:", availableBalance.toString());
  
  const contractBalance = await loanMachine.getContractBalance();
  console.log("Contract ETH balance:", contractBalance.toString());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});