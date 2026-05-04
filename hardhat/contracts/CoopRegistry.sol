// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title  CoopRegistry
 * @notice Lightweight on-chain registry for cooperatives.
 *         Does NOT deploy LoanMachine — your server does that separately.
 *         This only stores the mapping coopId → loanMachine address
 *         so the on-chain record can't be lost.
 */
contract CoopRegistry {

    address public platformAdmin;

    struct CoopRecord {
        address loanMachine;
        string  name;
        uint256 registeredAt;
        bool    exists;
    }

    mapping(bytes32 => CoopRecord) public coops;
    bytes32[] public allCoopIds;

    event CoopRegistered(
        bytes32 indexed coopId,
        address indexed loanMachine,
        string  name
    );

    error Registry_NotPlatformAdmin();
    error Registry_CoopNotFound();
    error Registry_NameEmpty();
    error Registry_AlreadyRegistered();

    constructor() {
        platformAdmin = msg.sender;
    }

    modifier onlyPlatformAdmin() {
        if (msg.sender != platformAdmin) revert Registry_NotPlatformAdmin();
        _;
    }

    /// @notice Server deploys LoanMachine separately, then registers it here.
    function registerCoop(
        string  calldata name,
        address loanMachine
    ) external onlyPlatformAdmin returns (bytes32 coopId) {
        if (bytes(name).length == 0) revert Registry_NameEmpty();

        coopId = keccak256(abi.encodePacked(name, loanMachine));
        if (coops[coopId].exists) revert Registry_AlreadyRegistered();

        coops[coopId] = CoopRecord({
            loanMachine: loanMachine,
            name:        name,
            registeredAt: block.timestamp,
            exists:      true
        });
        allCoopIds.push(coopId);

        emit CoopRegistered(coopId, loanMachine, name);
    }

    function getCoopInstance(bytes32 coopId) external view returns (address) {
        if (!coops[coopId].exists) revert Registry_CoopNotFound();
        return coops[coopId].loanMachine;
    }

    function getAllCoops() external view returns (bytes32[] memory) {
        return allCoopIds;
    }

    function getCoopCount() external view returns (uint256) {
        return allCoopIds.length;
    }

    function transferPlatformAdmin(address newAdmin) external onlyPlatformAdmin {
        platformAdmin = newAdmin;
    }
}
