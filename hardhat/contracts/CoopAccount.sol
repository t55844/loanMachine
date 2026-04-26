// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

error CoopAccount_NotOwner();
error CoopAccount_NotGuardian();
error CoopAccount_OnlyLoanMachine();
error CoopAccount_AlreadyApproved();
error CoopAccount_AlreadyInitialized();
error CoopAccount_CallFailed();

contract CoopAccount {

    address public owner;
    address public loanMachine;
    bytes32 public memberId;            // hashed CPF/CNPJ

    address[] public guardians;

    mapping(address => mapping(address => bool)) private recoveryApprovals;
    mapping(address => uint256) public recoveryApprovalCount;

    uint256 public constant RECOVERY_THRESHOLD = 2;
    bool private _initialized;

    event Executed(address indexed target, bytes data);
    event RecoveryApproved(address indexed guardian, address indexed proposedOwner, uint256 approvalCount);
    event OwnerRecovered(address indexed oldOwner, address indexed newOwner);
    event GuardianAdded(address indexed guardian);

    modifier onlyOwner() {
        if (msg.sender != owner) revert CoopAccount_NotOwner();
        _;
    }

    modifier onlyGuardian() {
        bool found = false;
        for (uint i = 0; i < guardians.length; i++) {
            if (guardians[i] == msg.sender) { found = true; break; }
        }
        if (!found) revert CoopAccount_NotGuardian();
        _;
    }

    function initialize(
        address _owner,
        address _loanMachine,
        bytes32 _memberId,              // hashed CPF/CNPJ
        address[] calldata _guardians
    ) external {
        if (_initialized) revert CoopAccount_AlreadyInitialized();
        owner       = _owner;
        loanMachine = _loanMachine;
        memberId    = _memberId;

        for (uint i = 0; i < _guardians.length; i++) {
            guardians.push(_guardians[i]);
            emit GuardianAdded(_guardians[i]);
        }
        _initialized = true;
    }

    function execute(address target, bytes calldata data) external onlyOwner {
        if (target != loanMachine) revert CoopAccount_OnlyLoanMachine();
        (bool success, ) = target.call(data);
        if (!success) revert CoopAccount_CallFailed();
        emit Executed(target, data);
    }

    function approveRecovery(address proposedOwner) external onlyGuardian {
        if (recoveryApprovals[proposedOwner][msg.sender])
            revert CoopAccount_AlreadyApproved();

        recoveryApprovals[proposedOwner][msg.sender] = true;
        recoveryApprovalCount[proposedOwner] += 1;

        emit RecoveryApproved(msg.sender, proposedOwner, recoveryApprovalCount[proposedOwner]);

        if (recoveryApprovalCount[proposedOwner] >= RECOVERY_THRESHOLD) {
            address oldOwner = owner;
            owner = proposedOwner;
            emit OwnerRecovered(oldOwner, proposedOwner);
        }
    }

    function getGuardians() external view returns (address[] memory) { return guardians; }
    function getRecoveryApprovalCount(address proposedOwner) external view returns (uint256) { return recoveryApprovalCount[proposedOwner]; }
    function hasGuardianApproved(address guardian, address proposedOwner) external view returns (bool) { return recoveryApprovals[proposedOwner][guardian]; }
}
