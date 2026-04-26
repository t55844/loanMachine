// Deployable contract definitions — includes bytecode for deploy scripts & tests.
// This file is separate from abis.rs to avoid bundling bytecode into production
// binaries that only need to call contracts, not deploy them.

use alloy::sol;

// Hardhat puts compiled artifacts here. Adjust the path if your layout differs.
// The `json` attribute tells sol! to read the ABI + bytecode from the artifact.
sol! {
    #[sol(rpc)]
    MockUSDT,
    "../../hardhat/artifacts/contracts/MockUSDT.sol/MockUSDT.json"
}

sol! {
    #[sol(rpc)]
    LoanMachine,
    "../../hardhat/artifacts/contracts/LoanMachine.sol/LoanMachine.json"
}

sol! {
    #[sol(rpc)]
    CoopRegistry,
    "../../hardhat/artifacts/contracts/CoopRegistry.sol/CoopRegistry.json"
}

sol! {
    #[sol(rpc)]
    CoopAccount,
    "../../hardhat/artifacts/contracts/CoopAccount.sol/CoopAccount.json"
}