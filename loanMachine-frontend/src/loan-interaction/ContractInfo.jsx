import { useEffect, useState } from "react";
import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";

const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

function ContractInfo({ account }) {
  const [balance, setBalance] = useState("0");

  useEffect(() => {
    async function fetchData() {
      if (window.ethereum) {
        const provider = new ethers.providers.Web3Provider(window.ethereum);
        const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, provider);
        const bal = await contract.getContractBalance();
        setBalance(ethers.utils.formatEther(bal));
      }
    }
    fetchData();
  }, [account]);

  return (
    <div className="mb-4">
      <h2>Saldo do Contrato: {balance} ETH</h2>
    </div>
  );
}

export default ContractInfo;
