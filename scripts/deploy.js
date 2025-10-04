const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying contracts...");

  // Deploy mock USDT
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  console.log("MockUSDT deployed to:", await mockUSDT.getAddress());

  // Deploy LoanMachine with USDT address
  const LoanMachine = await ethers.getContractFactory("LoanMachine");
  const loanMachine = await LoanMachine.deploy(await mockUSDT.getAddress());
  await loanMachine.waitForDeployment();
  console.log("LoanMachine deployed to:", await loanMachine.getAddress());

  // Check balances
  const contractBalance = await mockUSDT.balanceOf(await loanMachine.getAddress());
  console.log("LoanMachine USDT balance:", contractBalance.toString());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
