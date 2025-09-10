import { useEffect, useState } from "react";
import { ethers } from "ethers";

const RPC_URL = import.meta.env.VITE_RPC_URL;
const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;

const ABI = [
  "function getTotalDonations() view returns (uint256)",
  "function getTotalBorrowed() view returns (uint256)",
  "function getAvailableBalance() view returns (uint256)",
  "function getContractBalance() view returns (uint256)"
];

export default function ContractInfo() {
  const [data, setData] = useState({ donations: "0", borrowed: "0", available: "0", balance: "0" });

  useEffect(() => {
    async function loadData() {
      try {
        const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
        const contract = new ethers.Contract(CONTRACT_ADDRESS, ABI, provider);
        const [d, b, a, bal] = await Promise.all([
          contract.getTotalDonations(),
          contract.getTotalBorrowed(),
          contract.getAvailableBalance(),
          contract.getContractBalance()
        ]);
        setData({
          donations: ethers.utils.formatEther(d),
          borrowed: ethers.utils.formatEther(b),
          available: ethers.utils.formatEther(a),
          balance: ethers.utils.formatEther(bal)
        });
      } catch (e) {
        console.error("Error fetching data:", e);
      }
    }
    loadData();
  }, []);

  return (
    <div className="stats-box">
      <h2>Contract Info</h2>
      <div className="stats-grid">
        <div><strong>Total Donations:</strong> {data.donations} ETH</div>
        <div><strong>Total Borrowed:</strong> {data.borrowed} ETH</div>
        <div><strong>Available Balance:</strong> {data.available} ETH</div>
        <div><strong>Contract Balance:</strong> {data.balance} ETH</div>
      </div>
    </div>
  );
}
