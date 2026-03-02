const { ethers } = require("hardhat");

// ─────────────────────────────────────────────────────────────────────────────
// LOCAL DEV CONFIG — hardcoded for hardhat local network
// When moving to testnet/mainnet, swap these for env vars or a config file
// ─────────────────────────────────────────────────────────────────────────────
const LOCAL = {
  coopName:    "Dev Coop",
  accessCode:  "dev-access-code",
  memberId:    1,
  // guardians left empty for local — in production these are other members'
  // CoopAccount addresses (the Privy embedded wallet addresses)
  guardians:   [],
};

async function main() {
  const [platformAdmin, coopAdmin, member1] = await ethers.getSigners();

  console.log("🚀 Local deployment — hardhat network");
  console.log("   platformAdmin :", platformAdmin.address);
  console.log("   coopAdmin     :", coopAdmin.address);
  console.log("   member1       :", member1.address);
  console.log("");

  // ── 1. MockUSDT ───────────────────────────────────────────────────────────
  const MockUSDT = await ethers.getContractFactory("MockUSDT");
  const mockUSDT = await MockUSDT.deploy();
  await mockUSDT.waitForDeployment();
  const usdtAddr = await mockUSDT.getAddress();
  console.log("✅ MockUSDT           →", usdtAddr);

  // ── 2. LoanMachineFactory ─────────────────────────────────────────────────
  // One factory per platform — never deploy LoanMachine directly.
  // ReputationLib is internal so no separate deployment needed there either.
  const Factory = await ethers.getContractFactory("LoanMachineFactory");
  const factory = await Factory.deploy();
  await factory.waitForDeployment();
  const factoryAddr = await factory.getAddress();
  console.log("✅ LoanMachineFactory →", factoryAddr);

  // ── 3. Deploy first coop via factory ─────────────────────────────────────
  // deployCoop() internally does: new LoanMachine(usdt) + initializeAdmin()
  // Factory renounces control immediately after — coopAdmin owns the instance
  const deployTx = await factory.deployCoop(
    LOCAL.coopName,
    usdtAddr,
    coopAdmin.address,
    LOCAL.accessCode
  );
  const receipt = await deployTx.wait();

  const event = receipt.logs
    .map(log => { try { return factory.interface.parseLog(log); } catch { return null; } })
    .find(e => e?.name === "CoopDeployed");

  const coopId          = event.args.coopId;
  const loanMachineAddr = event.args.loanMachine;
  console.log("✅ Coop deployed");
  console.log("   coopId        :", coopId);
  console.log("   LoanMachine   :", loanMachineAddr);

  // ── 4. CoopAccount (Privy smart wallet) for member1 ──────────────────────
  // In production this is deployed by the frontend after Privy creates the
  // embedded wallet — the embedded wallet address becomes `owner` here.
  // For local dev we simulate it with the hardhat signer directly.
  const CoopAccount = await ethers.getContractFactory("CoopAccount");
  const coopAccount = await CoopAccount.deploy();
  await coopAccount.waitForDeployment();
  const accountAddr = await coopAccount.getAddress();

  await coopAccount.initialize(
    member1.address,    // owner — in prod this is the Privy embedded wallet
    loanMachineAddr,    // the coop this account is locked to
    LOCAL.memberId,     // memberId links account to cooperative member
    LOCAL.guardians     // empty locally; in prod: other members' CoopAccount addresses
  );
  console.log("✅ CoopAccount        →", accountAddr);
  console.log("   owner         :", member1.address);
  console.log("   loanMachine   :", loanMachineAddr);
  console.log("   memberId      :", LOCAL.memberId);

  // ── 5. Approve member1's wallet in the coop (coopAdmin step) ─────────────
  // Before a member can joinCoop(), coopAdmin must approve their wallet.
  // In prod the frontend calls this after off-chain KYC/verification.
  const loanMachine = await ethers.getContractAt("LoanMachine", loanMachineAddr);
  await loanMachine.connect(coopAdmin).approveWallet(member1.address);
  console.log("✅ Wallet approved     → member1 can now call joinCoop()");

  // ── 6. Summary ────────────────────────────────────────────────────────────
  console.log("");
  console.log("── Addresses ────────────────────────────────────────────────");
  console.log("   MockUSDT      :", usdtAddr);
  console.log("   Factory       :", factoryAddr);
  console.log("   LoanMachine   :", loanMachineAddr);
  console.log("   CoopAccount   :", accountAddr);
  console.log("   Coop ID       :", coopId);
  console.log("─────────────────────────────────────────────────────────────");
  console.log("");
  console.log("🔑 Access code        :", LOCAL.accessCode);
  console.log("   (member1 needs this to call joinCoop())");
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});