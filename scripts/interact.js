const hre = require("hardhat");

async function main() {
  const [owner,user1,user2,user3,user4] = await hre.ethers.getSigners();
  const LoanMachine = await hre.ethers.getContractFactory("LoanMachine");
  const contract = await LoanMachine.attach("0x5FbDB2315678afecb367f032d93F642f64180aa3");

  console.log("Wallets:", owner.address, user1.address, user2.address, user3.address, user4.address);

  // Example: get loan contract with id = 0
  try {
    const loan = await contract.getLoanContract(0);
    console.log("Loan contract 0:", loan);
  } catch (err) {
    console.log("No loan contract found yet.");
  }

    

    const accounts = await hre.ethers.getSigners();

    for (let i = 0; i < accounts.length; i++) {
    const balance = await hre.ethers.provider.getBalance(accounts[i].address);

    const donationsInCoverage = await contract.getDonationsInCoverage(accounts[i].address);

    console.log(
        `Account ${i}: ${accounts[i].address} -> ${hre.ethers.formatEther(balance)} ETH | DonationsInCoverage: ${hre.ethers.formatEther(donationsInCoverage)} ETH`
    );
    }


 /* // Check for Borrowed events
  const filter = contract.filters.Borrowed();
  const events = await contract.queryFilter(filter, 0, "latest");

  for (let ev of events) {
    console.log("Borrowed:", ev.args.borrower, ev.args.amount.toString());
  }*/

  // Check donations for user4
  const donations = await contract.getDonation(user4.address);
  console.log("Donations from user4:", hre.ethers.formatEther(donations));
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
