import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";

const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

function Donate({ account }) {
  async function handleDonate() {
    const amount = prompt("Quanto deseja doar em ETH?");
    if (!amount) return;

    const provider = new ethers.providers.Web3Provider(window.ethereum);
    const signer = provider.getSigner();
    const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, signer);

    const tx = await contract.donate({ value: ethers.utils.parseEther(amount) });
    await tx.wait();
    alert("Doação enviada!");
  }

  return (
    <button onClick={handleDonate} className="bg-green-500 text-white px-4 py-2 rounded mr-2">
      Doar
    </button>
  );
}

export default Donate;
