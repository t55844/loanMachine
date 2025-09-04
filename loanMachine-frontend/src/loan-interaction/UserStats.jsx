import { useEffect, useState } from "react";
import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";


const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";

function UserStats({ account }) {
  const [stats, setStats] = useState({});

  useEffect(() => {
    async function fetchStats() {
      const provider = new ethers.providers.Web3Provider(window.ethereum);
      const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, provider);
      const userStats = await contract.getUserStats(account);
      setStats({
        donations: ethers.utils.formatEther(userStats[0]),
        borrowings: ethers.utils.formatEther(userStats[1]),
      });
    }
    fetchStats();
  }, [account]);

  return (
    <div className="mb-4">
      <h3>Minhas Informações</h3>
      <p>Doações: {stats.donations} ETH</p>
      <p>Empréstimos: {stats.borrowings} ETH</p>
    </div>
  );
}

export default UserStats;
