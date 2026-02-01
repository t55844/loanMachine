import { useEffect, useState } from "react";
import { PieChart, Pie, Cell, Tooltip, Legend, ResponsiveContainer } from "recharts";
import { fetchDonationsAndBorrows } from "../graphql-frontend-query";
import { ethers } from "ethers";

const COLORS = ["#4A6FA5", "#4AA586", "#A54A6F", "#C67828", "#7F7F7F", "#B0B0B0", "#6F6FA5"];

export default function WalletDistribution() {
  const [donationsData, setDonationsData] = useState([]);
  const [borrowingsData, setBorrowingsData] = useState([]);

  useEffect(() => {
    async function loadData() {
      try {
        const [donations, borrowings] = await fetchDonationsAndBorrows();

        // Aggregate donations by donor - convert from wei to USDT (6 decimals)
        const donationsAggregated = donations.reduce((acc, d) => {
          const wallet = d.donor.id;
          const value = parseFloat(ethers.utils.formatUnits(d.amount, 6));
          acc[wallet] = (acc[wallet] || 0) + value;
          return acc;
        }, {});

        const donationsChartData = Object.entries(donationsAggregated).map(([wallet, value]) => ({
          wallet,
          value: parseFloat(value.toFixed(4))
        }));

        // Aggregate borrowings by borrower - convert from wei to USDT (6 decimals)
        const borrowingsAggregated = borrowings.reduce((acc, b) => {
          const wallet = b.borrower.id;
          const value = parseFloat(ethers.utils.formatUnits(b.amount, 6));
          acc[wallet] = (acc[wallet] || 0) + value;
          return acc;
        }, {});

        const borrowingsChartData = Object.entries(borrowingsAggregated).map(([wallet, value]) => ({
          wallet,
          value: parseFloat(value.toFixed(4))
        }));

        setDonationsData(donationsChartData);
        setBorrowingsData(borrowingsChartData);
      } catch (err) {
        //console.error("Erro ao buscar dados:", err);
      }
    }
    loadData();
  }, []);

  return (
    <div className="graphBlock">
      <h2>Distribuição de Carteiras</h2>
      <div style={{ display: 'flex', flexDirection: 'row', gap: '20px', flexWrap: 'wrap' }}>
        {/* Donations Pie Chart */}
        <div style={{ flex: 1, minWidth: 300 }}>
          <h3>Doações</h3>
          <ResponsiveContainer width="100%" height={400}>
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
                  <Cell key={`donation-cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip formatter={(value) => `${value} USDT`} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Borrowings Pie Chart */}
        <div style={{ flex: 1, minWidth: 300 }}>
          <h3>Empréstimos</h3>
          <ResponsiveContainer width="100%" height={400}>
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
                  <Cell key={`borrowing-cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip formatter={(value) => `${value} USDT`} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}