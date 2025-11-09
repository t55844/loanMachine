// quick-check.js
const { ethers } = require("ethers");

async function quickCheck() {
  const provider = new ethers.JsonRpcProvider(process.env.ALCHEMY_API_URL);
  const receipt = await provider.getTransactionReceipt("0x6f20cd4b360a93f0ef2afd11c80c6720720a9c372820042e272891f263a5af74");
  
  const eventTopic = ethers.id("MemberToWalletVinculation(address,uint256)");
  const hasEvent = receipt.logs.some(log => 
    log.address.toLowerCase() === "0xf9B64b3242DDFc7627cd764825617e6d9310Ce95".toLowerCase() &&
    log.topics[0] === eventTopic
  );
  
  console.log(hasEvent ? "✅ YES - Event was emitted" : "❌ NO - Event was not emitted");
}

quickCheck().catch(console.error);