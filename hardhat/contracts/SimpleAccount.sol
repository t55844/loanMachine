// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

// ── ERRORS ───────────────────────────────────────────────────
error CoopAccount_NotOwner();
error CoopAccount_NotGuardian();
error CoopAccount_OnlyLoanMachine();
error CoopAccount_AlreadyApproved();
error CoopAccount_AlreadyInitialized();
error CoopAccount_CallFailed();

contract CoopAccount {

    // ── STORAGE ──────────────────────────────────────────────
    address public owner;
    address public loanMachine;
    uint32  public memberId;

    // Guardians — other members who can help recover this account
    // e.g. 3 trusted members of the same cooperative
    address[] public guardians;

    // Recovery tracking:
    // proposedOwner → (guardian address → has approved?)
    // Lets multiple guardians each vote for a specific new owner
    mapping(address => mapping(address => bool)) private recoveryApprovals;

    // proposedOwner → how many guardians approved it
    mapping(address => uint256) public recoveryApprovalCount;

    // How many guardians must agree to recover (e.g. 2 of 3)
    uint256 public constant RECOVERY_THRESHOLD = 2;

    bool private _initialized;

    // ── EVENTS ───────────────────────────────────────────────
    event Executed(address indexed target, bytes data);
    event RecoveryApproved(address indexed guardian, address indexed proposedOwner, uint256 approvalCount);
    event OwnerRecovered(address indexed oldOwner, address indexed newOwner);
    event GuardianAdded(address indexed guardian);

    // ── MODIFIERS ────────────────────────────────────────────
    modifier onlyOwner() {
        if (msg.sender != owner) revert CoopAccount_NotOwner();
        _;
    }

    modifier onlyGuardian() {
        bool found = false;
        for (uint i = 0; i < guardians.length; i++) {
            if (guardians[i] == msg.sender) {
                found = true;
                break;
            }
        }
        if (!found) revert CoopAccount_NotGuardian();
        _;
    }

    // ── INITIALIZATION ────────────────────────────────────────
    // Called once by the factory right after deployment.

    function initialize(
        address _owner,
        address _loanMachine,
        uint32  _memberId,
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

    // ── EXECUTE ───────────────────────────────────────────────
    // The core function — owner sends transactions through this wallet.
    // Only calls to OUR LoanMachine are allowed.
    // This is the security boundary: even if the owner key is compromised,
    // an attacker can't drain funds to an arbitrary address.

    function execute(
        address target,
        bytes calldata data
    ) external onlyOwner {
        // ← FIXED: was ==, should be !=
        // "if target is NOT our LoanMachine, reject"
        if (target != loanMachine) revert CoopAccount_OnlyLoanMachine();

        (bool success, ) = target.call(data);
        if (!success) revert CoopAccount_CallFailed();

        emit Executed(target, data);
    }

    // ── SOCIAL RECOVERY ───────────────────────────────────────
    // If a member loses their key, their guardians can vote to
    // install a new owner address.
    //
    // Flow:
    //   Guardian A calls approveRecovery(newOwner)  → count = 1
    //   Guardian B calls approveRecovery(newOwner)  → count = 2 → owner rotated!
    //
    // Each guardian can only vote once per proposed address.

    function approveRecovery(address proposedOwner) external onlyGuardian {
        // Prevent double-voting for the same proposed owner
        if (recoveryApprovals[proposedOwner][msg.sender])
            revert CoopAccount_AlreadyApproved();

        // Record this guardian's vote
        recoveryApprovals[proposedOwner][msg.sender] = true;
        recoveryApprovalCount[proposedOwner] += 1;

        emit RecoveryApproved(
            msg.sender,
            proposedOwner,
            recoveryApprovalCount[proposedOwner]
        );

        // If enough guardians agreed, rotate the owner
        if (recoveryApprovalCount[proposedOwner] >= RECOVERY_THRESHOLD) {
            address oldOwner = owner;
            owner = proposedOwner;
            emit OwnerRecovered(oldOwner, proposedOwner);
        }
    }

    // ── VIEW ──────────────────────────────────────────────────

    function getGuardians() external view returns (address[] memory) {
        return guardians;
    }

    function getRecoveryApprovalCount(address proposedOwner)
        external view returns (uint256)
    {
        return recoveryApprovalCount[proposedOwner];
    }

    function hasGuardianApproved(address guardian, address proposedOwner)
        external view returns (bool)
    {
        return recoveryApprovals[proposedOwner][guardian];
    }
}
/*```

---

### Why each piece exists
```
owner           → the Privy embedded wallet key
                  signs all transactions

loanMachine     → the cooperative's contract address
                  execute() ONLY allows calls to this address
                  even if owner key is stolen, attacker
                  can only call LoanMachine (can't send ETH elsewhere)

memberId        → links this smart wallet to the cooperative member
                  useful for on-chain queries

guardians[]     → 3 trusted members chosen by the wallet owner
                  stored as an array because there are few of them (<5)
                  each is another member's wallet address

recoveryApprovals[proposedOwner][guardian]
                → tracks which guardian voted for which new owner
                  mapping(address → mapping(address → bool))
                  first address = proposed new owner
                  second address = guardian who voted
                  prevents double-voting

recoveryApprovalCount[proposedOwner]
                → simple counter: how many guardians approved this address
                  when it hits RECOVERY_THRESHOLD (2), owner is swapped*/