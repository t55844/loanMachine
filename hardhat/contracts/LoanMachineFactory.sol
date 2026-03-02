// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./LoanMachine.sol";

contract LoanMachineFactory {

    // ── STORAGE ──────────────────────────────────────────────
    address public platformAdmin;

    struct CoopRecord {
        address loanMachine;  // the deployed instance
        string  name;
        uint256 deployedAt;
        bool    exists;
    }

    mapping(bytes32 => CoopRecord) public coops;
    bytes32[] public allCoopIds;

    // ── EVENTS ───────────────────────────────────────────────
    event CoopDeployed(
        bytes32 indexed coopId,
        address indexed loanMachine,
        address indexed coopAdmin,
        string  name
    );

    // ── ERRORS ───────────────────────────────────────────────
    error Factory_NotPlatformAdmin();
    error Factory_CoopNotFound();
    error Factory_NameEmpty();

    // ── CONSTRUCTOR ──────────────────────────────────────────
    constructor() {
        platformAdmin = msg.sender;
    }

    modifier onlyPlatformAdmin() {
        if (msg.sender != platformAdmin) revert Factory_NotPlatformAdmin();
        _;
    }

    // ── DEPLOY ───────────────────────────────────────────────

    /// Deploys a new LoanMachine and hands full control to coopAdmin.
    /// After this call, the Factory has ZERO authority over the instance.
    function deployCoop(
        string  calldata name,
        address          usdtToken,
        address          coopAdmin,   // who will own/manage the LoanMachine
        string  calldata accessCode
    ) external onlyPlatformAdmin returns (bytes32 coopId, address loanMachine) {
        if (bytes(name).length == 0) revert Factory_NameEmpty();

        // Deploy the instance
        LoanMachine instance = new LoanMachine(usdtToken);

        // Immediately transfer all authority to coopAdmin.
        // From this point the Factory cannot call anything privileged.
        instance.initializeAdmin(coopAdmin, accessCode);

        coopId = keccak256(abi.encodePacked(name, block.timestamp, coopAdmin));

        coops[coopId] = CoopRecord({
            loanMachine: address(instance),
            name:        name,
            deployedAt:  block.timestamp,
            exists:      true
        });

        allCoopIds.push(coopId);

        emit CoopDeployed(coopId, address(instance), coopAdmin, name);
        return (coopId, address(instance));
    }

    // ── VIEW ─────────────────────────────────────────────────

    function getCoopInstance(bytes32 coopId) external view returns (address) {
        if (!coops[coopId].exists) revert Factory_CoopNotFound();
        return coops[coopId].loanMachine;
    }

    function getAllCoops() external view returns (bytes32[] memory) {
        return allCoopIds;
    }

    function transferPlatformAdmin(address newAdmin) external onlyPlatformAdmin {
        platformAdmin = newAdmin;
    }
}