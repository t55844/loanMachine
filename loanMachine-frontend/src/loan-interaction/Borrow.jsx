import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";

const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

function Borrow({ account }) {
  async function handleBorrow() {
    const amount = prompt("Quanto deseja emprestar em ETH?");
    if (!amount) return;

    const provider = new ethers.providers.Web3Provider(window.ethereum);
    const signer = provider.getSigner();
    const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, signer);

    const tx = await contract.borrow(ethers.utils.parseEther(amount));
    await tx.wait();
    alert("Empréstimo realizado!");
  }

 return (
  <button onClick={handleBorrow} className="button button-yellow">
    Emprestar
  </button>
);

}

export default Borrow;
