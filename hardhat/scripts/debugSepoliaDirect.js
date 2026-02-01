// scripts/debugSepoliaDirect.js
const { ethers } = require("ethers");
require('dotenv').config();

async function main() {
  // Sepolia RPC URL 
  const SEPOLIA_RPC = process.env.ALCHEMY_API_URL;
  if (!SEPOLIA_RPC) {
    console.error("Please set ALCHEMY_API_URL environment variable");
    return;
  }
  
  // For ethers v6, it's just ethers.JsonRpcProvider
  const provider = new ethers.JsonRpcProvider(SEPOLIA_RPC);
  
  // Your contract address
  const loanMachineAddress = "0xE797948c05aa26369825bA03D2b5e0eBB4ed28C1";
  
  // Your wallet private key
  const privateKey = process.env.PRIVATE_KEY_TEST;
  if (!privateKey) {
    console.error("Please set PRIVATE_KEY_TEST environment variable");
    return;
  }
  
  const wallet = new ethers.Wallet(privateKey, provider);
  
  // ABI for just the function we're calling
  const abi = [
    "function createLoanRequisition(uint256 amount, uint32 minimumCoverage, uint32 parcelscount, uint32 memberId, uint32 daysIntervalOfPayment) external"
  ];
  
  const loanMachine = new ethers.Contract(loanMachineAddress, abi, wallet);
  
  console.log("Using wallet:", wallet.address);
  
  try {
    console.log("Attempting to create loan requisition...");
    
    const tx = await loanMachine.createLoanRequisition(
      ethers.parseUnits("100000", 6), // ethers v6 uses parseUnits directly
      75,
      10, 
      373272171,
      30,
      {
        gasLimit: 500000
      }
    );
    
    console.log("Transaction sent:", tx.hash);
    const receipt = await tx.wait();
    console.log("Transaction successful!");
    
  } catch (error) {
    console.log("\n=== RAW ERROR ===");
    console.log(error);
    
    console.log("\n=== ERROR MESSAGE ===");
    console.log(error.message);
    
    console.log("\n=== ERROR CODE ===");
    console.log(error.code);
    
    console.log("\n=== ERROR DATA ===");
    console.log(error.data);
    
    console.log("\n=== ERROR REASON ===");
    console.log(error.reason);
    
    // Try to extract from nested structures
    if (error.error) {
      console.log("\n=== NESTED ERROR ===");
      console.log(error.error);
      
      if (error.error.data) {
        console.log("Nested error data:", error.error.data);
      }
    }
    
    // Check transaction if available
    if (error.transaction) {
      console.log("\n=== TRANSACTION DATA ===");
      console.log(error.transaction);
    }
    
    // Try to parse the error body if it exists
    if (error.body) {
      console.log("\n=== ERROR BODY ===");
      console.log(error.body);
      try {
        const bodyObj = JSON.parse(error.body);
        console.log("Parsed error body:", bodyObj);
      } catch (e) {
        console.log("Could not parse error body as JSON");
      }
    }
    
    // In ethers v6, error info might be in error.info
    if (error.info) {
      console.log("\n=== ERROR INFO ===");
      console.log(error.info);
    }
  }
}

main().catch(console.error);

/// npx hardhat run scripts/debugTransaction.js --network sepolia