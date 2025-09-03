// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Import OpenZeppelin's ReentrancyGuard
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

contract loanMachine is ReentrancyGuard {
    // State variables
    mapping(address => uint256) public donations;
    mapping(address => uint256) public borrowings;
    mapping(address => uint256) private lastBorrowTime;
    //bool private locked = false; // o locked faz parte de um padrao de segurança que evita que alguem acesse a funcao
    //de pagamento, antes dela terminar e atualizar, fazendo outro pedido e roubando antes da atualizacao, por isso
    //check se tem dinheiro, effect alterando o estado e diminuindo o balanco, interact enviando o dinheiro para a carteira.
    //tudo isso entre os locked.
    // 1. Check ✅
    // 2. Effect 📝 (Update state FIRST)
    // 3. Interact ✅ (Now safe to send Ether)

    uint256 public totalDonations;
    uint256 public totalBorrowed;
    uint256 public availableBalance;

    // Constants for rules
    uint256 private constant MAX_BORROW_AMOUNT = 1 ether;
    uint256 private constant BORROW_DURATION = 7 days;
    uint256 private constant MIN_DONATION_FOR_BORROW = 0.1 ether;

    // Events sao caixas especiais para guradar valores sem gastar tanto quanto em variaveis normais e acessiveis ao off-chain
    //pode definir 2 tipos de variavel, indexed and notIndexede, a diferenca é o indexede é mais eficiente para filtrar e
    //limita-se a 3
    event Donated(address indexed donor, uint256 amount);
    event Borrowed(address indexed borrower, uint256 amount);
    event Repaid(address indexed borrower, uint256 amount);
    event BorrowLimitReached(address indexed borrower);

    // Modifiers sao acoplamentos que contem uma logica, majoritariamente de validação, que colocada em uma outra funcao
    // se eu tenho uma funcao andar de carro, posso ter um modifier freio de mão puxado para validar.
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

    // Donate function
    function donate() external payable validAmount(msg.value) nonReentrant {
        donations[msg.sender] += msg.value;
        totalDonations += msg.value;
        availableBalance += msg.value;

        emit Donated(msg.sender, msg.value);
    }

    // Borrow function
    function borrow(
        uint256 _amount
    ) external validAmount(_amount) canBorrow(_amount) nonReentrant {
        borrowings[msg.sender] += _amount;
        lastBorrowTime[msg.sender] = block.timestamp;
        totalBorrowed += _amount;
        availableBalance -= _amount;

        emit Borrowed(msg.sender, _amount);

        // Transfer funds to borrower
        payable(msg.sender).transfer(_amount);
    }

    // Repay function
    function repay() external payable validAmount(msg.value) nonReentrant {
        require(borrowings[msg.sender] > 0, "No active borrowing");
        require(
            msg.value <= borrowings[msg.sender],
            "Repayment exceeds borrowed amount"
        );

        borrowings[msg.sender] -= msg.value;
        totalBorrowed -= msg.value;
        availableBalance += msg.value;

        emit Repaid(msg.sender, msg.value);
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
}
