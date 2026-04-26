const { ethers, network } = require("hardhat");

// Real stablecoin addresses on Polygon
const STABLECOINS = {
  polygon: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359", // USDC native
  amoy:    "0x41E94Eb019C0762f9Bfcf9Fb1E58725BfB0e7582", // test USDC on Amoy
};

const LOCAL = {
  coopName: "Test Coop",
  accessCode: "test123",
  memberId: 1,
  guardians: [],
};

async function main() {
  const [platformAdmin, coopAdmin, member1] = await ethers.getSigners();
  const isLocal = network.name === "localhost" || network.name === "hardhat";

  console.log(`🚀 Deploying to ${network.name}`);
  console.log("   platformAdmin :", platformAdmin.address);

  // ── 1. USDT/USDC address ──────────────────────────────────
  let usdtAddr;
  if (isLocal) {
    const MockUSDT = await ethers.getContractFactory("MockUSDT");
    const mockUSDT = await MockUSDT.deploy();
    await mockUSDT.waitForDeployment();
    usdtAddr = await mockUSDT.getAddress();
    console.log("✅ MockUSDT           →", usdtAddr);
  } else {
    usdtAddr = STABLECOINS[network.name];
    if (!usdtAddr) throw new Error(`No stablecoin configured for network: ${network.name}`);
    console.log("✅ Using USDC         →", usdtAddr);
  }

  // ── 2. Factory ────────────────────────────────────────────
  const Factory = await ethers.getContractFactory("LoanMachineFactory");
  const factory = await Factory.deploy();
  await factory.waitForDeployment();
  const factoryAddr = await factory.getAddress();
  console.log("✅ LoanMachineFactory →", factoryAddr);

  // ── 3. Deploy Coop ────────────────────────────────────────
  // On mainnet you'd pass real coopAdmin address, not signers[1]
  const coopAdminAddr = isLocal ? coopAdmin.address : process.env.COOP_ADMIN_ADDRESS || coopAdmin.address;

  const deployTx = await factory.deployCoop(
    LOCAL.coopName,
    usdtAddr,
    coopAdminAddr,
    LOCAL.accessCode
  );
  const receipt = await deployTx.wait();

  const event = receipt.logs
    .map(log => { try { return factory.interface.parseLog(log); } catch { return null; } })
    .find(e => e?.name === "CoopDeployed");

  const coopId          = event.args.coopId;
  const loanMachineAddr = event.args.loanMachine;
  console.log("✅ Coop deployed");
  console.log("   Coop ID       :", coopId);
  console.log("   LoanMachine   :", loanMachineAddr);

  // ── 4-6. Local-only steps ─────────────────────────────────
  if (isLocal) {
    const CoopAccount = await ethers.getContractFactory("CoopAccount");
    const coopAccount = await CoopAccount.deploy();
    await coopAccount.waitForDeployment();
    const accountAddr = await coopAccount.getAddress();

    await coopAccount.initialize(
      member1.address, loanMachineAddr, LOCAL.memberId, LOCAL.guardians
    );
    /*console.log("✅ CoopAccount        →", accountAddr);

    const loanMachine = await ethers.getContractAt("LoanMachine", loanMachineAddr);
    await loanMachine.connect(coopAdmin).approveWallet(member1.address);
    console.log("✅ Wallet approved");

    await loanMachine.connect(member1).joinCoop(LOCAL.memberId, member1.address, LOCAL.accessCode);
    console.log("✅ Member1 joined coop");*/

    console.log("");
    console.log("── Addresses ────────────────────────────────────────────────");
    console.log("   MockUSDT      :", usdtAddr);
    console.log("   Factory       :", factoryAddr);
    console.log("   LoanMachine   :", loanMachineAddr);
    console.log("   CoopAccount   :", accountAddr);
    console.log("   Coop ID       :", coopId);
    console.log("─────────────────────────────────────────────────────────────");
  } else {
    console.log("");
    console.log("── Deployed Addresses ───────────────────────────────────────");
    console.log("   USDC          :", usdtAddr);
    console.log("   Factory       :", factoryAddr);
    console.log("   LoanMachine   :", loanMachineAddr);
    console.log("   Coop ID       :", coopId);
    console.log("─────────────────────────────────────────────────────────────");
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});