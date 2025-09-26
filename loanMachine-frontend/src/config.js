export const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3"; // substitua pelo endereço gerado
export const CONTRACT_ABI = [
  "function donate() payable",
  "function borrow(uint256 _amount) external",
  "function repay() payable",
  "function getUserStats(address _user) external view returns (uint256,uint256,uint256,bool)",
  "function getContractBalance() external view returns (uint256)",
  "function getAvailableBorrowAmount(address _user) external view returns (uint256)"
];