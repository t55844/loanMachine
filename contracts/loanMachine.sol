// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Import OpenZeppelin's ReentrancyGuard
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract loanMachine is ReentrancyGuard {
    // State variables
    mapping(address => uint256) private donations;
    mapping(address => uint256) private borrowings;
    mapping(address => uint256) private lastBorrowTime;

    uint256 private totalDonations;
    uint256 private totalBorrowed;
    uint256 private availableBalance;

    // Constants for rules
    uint256 private constant MAX_BORROW_AMOUNT = 1 ether;
    uint256 private constant BORROW_DURATION = 7 days;
    uint256 private constant MIN_DONATION_FOR_BORROW = 0.1 ether;

    // Events for gas-efficient tracking (CHEAP!)
    event Donated(address indexed donor, uint256 amount, uint256 totalDonation);
    event Borrowed(
        address indexed borrower,
        uint256 amount,
        uint256 totalBorrowing
    );
    event Repaid(
        address indexed borrower,
        uint256 amount,
        uint256 remainingDebt
    );
    event NewDonor(address indexed donor); // For tracking first-time donors
    event NewBorrower(address indexed borrower); // For tracking first-time borrowers
    event BorrowLimitReached(address indexed borrower);

    modifier validAmount(uint256 _amount) {
        require(_amount > 0, "Amount must be greater than 0");
        _;
    }

    modifier canBorrow(uint256 _amount) {
        require(
            _amount <= availableBalance,
            "Insufficient funds in vending machine"
        );
        require(_amount <= MAX_BORROW_AMOUNT, "Exceeds maximum borrow amount");
        require(
            donations[msg.sender] >= MIN_DONATION_FOR_BORROW ||
                borrowings[msg.sender] == 0,
            "Minimum donation required or existing debt"
        );
        require(
            borrowings[msg.sender] + _amount <= MAX_BORROW_AMOUNT,
            "Exceeds personal borrow limit"
        );
        require(
            lastBorrowTime[msg.sender] + BORROW_DURATION < block.timestamp ||
                borrowings[msg.sender] == 0,
            "Previous borrow not yet repaid or duration not expired"
        );
        _;
    }

    // Donate function - GAS EFFICIENT
    function donate() external payable validAmount(msg.value) nonReentrant {
        bool isNewDonor = donations[msg.sender] == 0;

        donations[msg.sender] += msg.value;
        totalDonations += msg.value;
        availableBalance += msg.value;

        // Emit events (CHEAP - ~2000 gas total)
        emit Donated(msg.sender, msg.value, donations[msg.sender]);
        if (isNewDonor) {
            emit NewDonor(msg.sender); // Helps off-chain indexers track all donors
        }
    }

    // Borrow function - GAS EFFICIENT
    function borrow(
        uint256 _amount
    ) external validAmount(_amount) canBorrow(_amount) nonReentrant {
        bool isNewBorrower = borrowings[msg.sender] == 0;

        borrowings[msg.sender] += _amount;
        lastBorrowTime[msg.sender] = block.timestamp;
        totalBorrowed += _amount;
        availableBalance -= _amount;

        // Emit events (CHEAP)
        emit Borrowed(msg.sender, _amount, borrowings[msg.sender]);
        if (isNewBorrower) {
            emit NewBorrower(msg.sender); // Helps off-chain indexers track all borrowers
        }

        // Transfer funds to borrower
        payable(msg.sender).transfer(_amount);
    }

    // Repay function - GAS EFFICIENT
    function repay() external payable validAmount(msg.value) nonReentrant {
        require(borrowings[msg.sender] > 0, "No active borrowing");
        require(
            msg.value <= borrowings[msg.sender],
            "Repayment exceeds borrowed amount"
        );

        borrowings[msg.sender] -= msg.value;
        totalBorrowed -= msg.value;
        availableBalance += msg.value;

        // Emit event (CHEAP)
        emit Repaid(msg.sender, msg.value, borrowings[msg.sender]);
    }

    // Check available borrow amount for user
    function getAvailableBorrowAmount(
        address _user
    ) external view returns (uint256) {
        uint256 maxPossible = MAX_BORROW_AMOUNT - borrowings[_user];
        return maxPossible < availableBalance ? maxPossible : availableBalance;
    }

    // Check if user can borrow
    function canUserBorrow(
        address _user,
        uint256 _amount
    ) external view returns (bool) {
        return (_amount > 0 &&
            _amount <= availableBalance &&
            _amount <= MAX_BORROW_AMOUNT &&
            (donations[_user] >= MIN_DONATION_FOR_BORROW ||
                borrowings[_user] == 0) &&
            (borrowings[_user] + _amount <= MAX_BORROW_AMOUNT) &&
            (lastBorrowTime[_user] + BORROW_DURATION < block.timestamp ||
                borrowings[_user] == 0));
    }

    /* GET INFORMATIONS ABOUT FUNDS, BALANCES AND VALUES */
    function getTotalDonations() external view returns (uint256) {
        return totalDonations;
    }

    function getTotalBorrowed() external view returns (uint256) {
        return totalBorrowed;
    }

    function getAvailableBalance() external view returns (uint256) {
        return availableBalance;
    }

    // Get contract balance
    function getContractBalance() external view returns (uint256) {
        return address(this).balance;
    }

    // Get user stats
    function getUserStats(
        address _user
    )
        external
        view
        returns (
            uint256 userDonations,
            uint256 userBorrowings,
            uint256 lastBorrow,
            bool canBorrowNow
        )
    {
        userDonations = donations[_user];
        userBorrowings = borrowings[_user];
        lastBorrow = lastBorrowTime[_user];
        canBorrowNow = (availableBalance > 0 &&
            borrowings[_user] < MAX_BORROW_AMOUNT &&
            (donations[_user] >= MIN_DONATION_FOR_BORROW ||
                borrowings[_user] == 0) &&
            (lastBorrowTime[_user] + BORROW_DURATION < block.timestamp ||
                borrowings[_user] == 0));
    }

    // Individual lookup functions (for current state)
    function getDonation(address _user) external view returns (uint256) {
        return donations[_user];
    }

    function getBorrowing(address _user) external view returns (uint256) {
        return borrowings[_user];
    }

    function getLastBorrowTime(address _user) external view returns (uint256) {
        return lastBorrowTime[_user];
    }
}
