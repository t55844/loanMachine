import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";

const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

function Repay({ account }) {
  async function handleRepay() {
    const amount = prompt("Quanto deseja devolver em ETH?");
    if (!amount) return;

    const provider = new ethers.providers.Web3Provider(window.ethereum);
    const signer = provider.getSigner();
    const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, signer);

    const tx = await contract.repay({ value: ethers.utils.parseEther(amount) });
    await tx.wait();
    alert("Empréstimo devolvido!");
  }

 return (
  <button onClick={handleRepay} className="button button-red">
    Pagar
  </button>
);

}

export default Repay;
