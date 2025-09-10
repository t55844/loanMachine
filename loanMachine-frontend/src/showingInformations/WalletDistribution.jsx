import { useEffect, useState } from "react";
import { PieChart, Pie, Cell, Tooltip, Legend, ResponsiveContainer } from "recharts";
import { fetchDonations } from "../graphql-frontend-query";

const COLORS = ["#4A6FA5", "#4AA586", "#A54A6F", "#C67828", "#7F7F7F", "#B0B0B0", "#6F6FA5"];

export default function WalletDistribution() {
  const [donationsData, setDonationsData] = useState([]);

  useEffect(() => {
    async function loadData() {
      try {
        const donations = await fetchDonations();

        // Aggregate by donor
        const aggregated = donations.reduce((acc, d) => {
          const wallet = d.donor.id;
          const value = parseFloat((+d.amount / 1e18).toFixed(4)); // Convert from wei to ETH
          acc[wallet] = (acc[wallet] || 0) + value;
          return acc;
        }, {});

        const chartData = Object.entries(aggregated).map(([wallet, value]) => ({
          wallet,
          value
        }));

        setDonationsData(chartData);
      } catch (err) {
        console.error("Error fetching donations:", err);
      }
    }
    loadData();
  }, []);

  return (
    <div className="graphBlock">
      <h2>Wallet Distribution</h2>
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
    </div>
  );
}
