import { useEffect, useState } from "react";
import { ethers } from "ethers";

const RPC_URL = "http://127.0.0.1:8545";
const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
const ABI = [
  "function totalDonations() view returns (uint256)",
  "function totalBorrowed() view returns (uint256)",
  "function availableBalance() view returns (uint256)",
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
          contract.totalDonations(),
          contract.totalBorrowed(),
          contract.availableBalance(),
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
