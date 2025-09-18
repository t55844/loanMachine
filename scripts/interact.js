const hre = require("hardhat");

async function main(){
    const [owner,user1,user2,user3,user4] = await hre.ethers.getSigners();
    const LoanMachine = await hre.ethers.getContractFactory("loanMachine");
    const contract = await LoanMachine.attach("0x5FbDB2315678afecb367f032d93F642f64180aa3");

    console.log("Wallets: ",owner.address,user1.address,user2.address,user3.address,user4.address)

    //Donate
    const tx = await contract.connect(user1).donate({value: hre.ethers.utils.parseEther(Math.random().toString()) });
    await tx.wait();

    const tx2 = await contract.connect(user2).donate({value: hre.ethers.utils.parseEther(Math.random().toString()) });
    await tx2.wait();
    const tx3 = await contract.connect(user3).donate({value: hre.ethers.utils.parseEther(Math.random().toString()) });
    await tx3.wait();
    const tx4 = await contract.connect(user4).donate({value: hre.ethers.utils.parseEther(Math.random().toString()) });
    await tx4.wait();


    //Check donations
    const donations = await contract.getDonation(user1.address);
    console.log("Donations from user1: ", hre.ethers.utils.formatEther(donations));

};


main().catch((error) => {
    console.error(error)
    process.exit(1)
})