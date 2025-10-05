const hre = require("hardhat");
const CONTRACT_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512';
async function main() {
  const [owner,user1,user2,user3,user4] = await hre.ethers.getSigners();
  const LoanMachine = await hre.ethers.getContractFactory("LoanMachine");
  const contract = await LoanMachine.attach(CONTRACT_ADDRESS);

  console.log("Wallets:", owner.address, user1.address, user2.address, user3.address, user4.address);

  // Example: get loan contract with id = 0
  try {
    const loan = await contract.getLoanContract(0);
    console.log("Loan contract 0:", loan);
  } catch (err) {
    console.log("No loan contract found yet.");
  }

    
/*
    const accounts = await hre.ethers.getSigners();

    for (let i = 0; i < accounts.length; i++) {
    const balance = await hre.ethers.provider.getBalance(accounts[i].address);

    const donationsInCoverage = await contract.getDonationsInCoverage(accounts[i].address);

    console.log(
        `Account ${i}: ${accounts[i].address} -> ${hre.ethers.formatEther(balance)} ETH | DonationsInCoverage: ${hre.ethers.formatEther(donationsInCoverage)} ETH`
    );
    }*/



    const users = [owner, user1, user2, user3, user4];
    for (const user of users) {
        const bal = await hre.ethers.provider.getBalance(user.address);
        const usdtBal = await hre.ethers.provider.getUSDTBalance(user.address);

        console.log(`Balance of ${user.address}: ${hre.ethers.formatUnits(bal, 6)} USDT`);
       console.log(`Balance of ${user.address}: ${hre.ethers.formatUnits(usdtBal, 6)} USDT`);

    }

}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
