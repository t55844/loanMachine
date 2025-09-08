import { useEffect, useState } from "react";
import { PieChart, Pie, Cell, Tooltip, Legend, ResponsiveContainer } from "recharts";
import { ethers } from "ethers";
import abi from "../abi/loanMachine.json";

const CONTRACT_ADDRESS = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
const RPC_URL = "http://127.0.0.1:8545";

// Some nice colors for chart slices
const COLORS = ["#4A6FA5", "#4AA586", "#A54A6F", "#C67828", "#7F7F7F", "#B0B0B0", "#6F6FA5"];


export default function WalletDistribution() {
  const [donationsData, setDonationsData] = useState([]);
  const [borrowingsData, setBorrowingsData] = useState([]);


  useEffect(() => {
    async function fetchWalletData() {
      try {
        const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
        const contract = new ethers.Contract(CONTRACT_ADDRESS, abi, provider);

        // You need a list of accounts to loop
        const accounts = await provider.listAccounts();

        const donationsPromises = accounts.map(async (acc) => ({
          wallet: acc,
          value: parseFloat(ethers.utils.formatEther(await contract.donations(acc))),
        }));

        const borrowingsPromises = accounts.map(async (acc) => ({
          wallet: acc,
          value: parseFloat(ethers.utils.formatEther(await contract.borrowings(acc))),
        }));

        const donations = (await Promise.all(donationsPromises)).filter(d => d.value > 0);
        const borrowings = (await Promise.all(borrowingsPromises)).filter(b => b.value > 0);

        setDonationsData(donations);
        setBorrowingsData(borrowings);
      } catch (err) {
        console.error("Error fetching wallet data:", err);
      }
    }

    fetchWalletData();
  }, []);


  return (
    <div className="graphBlock">
      <h2>Wallet Distribution</h2>
      <div style={{ display: "flex", gap: "40px", flexWrap: "wrap" }}>
        <div style={{ flex: 1, minWidth: 300 }}>
          <h3>Donations</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={donationsData}
                dataKey="value"
                nameKey="wallet"
                cx="50%"
                cy="50%"
                outerRadius={80}
                label={(entry) => `${entry.wallet.slice(0,6)}...${entry.wallet.slice(-4)}`}
              >
                {donationsData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip formatter={(value) => `${value} ETH`} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>

        <div style={{ flex: 1, minWidth: 300 }}>
          <h3>Borrowings</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={borrowingsData}
                dataKey="value"
                nameKey="wallet"
                cx="50%"
                cy="50%"
                outerRadius={80}
                label={(entry) => `${entry.wallet.slice(0,6)}...${entry.wallet.slice(-4)}`}
              >
                {borrowingsData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip formatter={(value) => `${value} ETH`} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}
