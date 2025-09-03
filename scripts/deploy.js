async function main() {
  const LoanMachine = await ethers.getContractFactory("loanMachine");
  const loanMachine = await LoanMachine.deploy();
  await loanMachine.deployed();
  console.log("loanMachine deployed to:", loanMachine.address);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
