// ContractOverview.jsx
import { useEffect, useState } from "react";
import { ethers } from "ethers";
import { fetchContractStats, fetchLastTransactions } from "../graphql-frontend-query";

export default function ContractOverview() {
  const [stats, setStats] = useState(null);
  const [lastTxs, setLastTxs] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    let mounted = true;

    async function load() {
      setLoading(true);
      setError("");
      try {
        // Get contract status using fetchContractStats function
        const rawStats = await fetchContractStats();
        if (!mounted) return;

        setStats({
          totalDonations: ethers.utils.formatUnits(rawStats.totalDonations ?? "0", 6), // USDT (6 decimals)
          totalBorrowed: ethers.utils.formatUnits(rawStats.totalBorrowed ?? "0", 6), // USDT (6 decimals)
          availableBalance: ethers.utils.formatUnits(rawStats.availableBalance ?? "0", 6), // USDT (6 decimals)
          contractBalance: ethers.utils.formatUnits(rawStats.availableBalance ?? "0", 6) // USDT (6 decimals)
        });

        // Get last transactions
        const rawTxs = await fetchLastTransactions({ limit: 5 });
        if (!mounted) return;
        const mapped = rawTxs.map((tx) => ({
          wallet: (typeof tx.donor === 'string' ? tx.donor : tx.donor?.id) || 
                  (typeof tx.borrower === 'string' ? tx.borrower : tx.borrower?.id) || 
                  "unknown",
          amount: ethers.utils.formatUnits(tx.amount ?? "0", 6),
          time: tx.timestamp,
          type: tx.type
        }));

        setLastTxs(mapped);

      } catch (err) {
        console.error("ContractOverview load error:", err);
        if (mounted) setError("Error loading contract statistics");
        
        // Set empty stats if there's an error
        setStats({
          totalDonations: "0.0",
          totalBorrowed: "0.0",
          availableBalance: "0.0",
          contractBalance: "0.0"
        });
        
        setLastTxs([]);
      } finally {
        if (mounted) setLoading(false);
      }
    }

    load();
    return () => {
      mounted = false;
    };
  }, []);

  // Format USDT amount for display
  const formatUSDT = (amount) => {
    return parseFloat(amount).toFixed(2);
  };

  return (
    <div className="graphBlock contract-overview">
      <h2>Contract Overview</h2>

      {loading && <p className="loading">Loading data...</p>}
      {error && <p className="error">{error}</p>}

      {stats && (
        <div className="stats-grid" style={{ marginTop: 8 }}>
          <div><strong>Total Donations:</strong> {formatUSDT(stats.totalDonations)} USDT</div>
          <div><strong>Total Borrowed:</strong> {formatUSDT(stats.totalBorrowed)} USDT</div>
          <div><strong>Available Balance:</strong> {formatUSDT(stats.availableBalance)} USDT</div>
          <div><strong>Contract Balance:</strong> {formatUSDT(stats.contractBalance)} USDT</div>
        </div>
      )}

      <h3 style={{ marginTop: 18 }}>Last 5 Transactions</h3>

      <div className="transactions-box" style={{ marginTop: 12 }}>
        {lastTxs.length > 0 ? (
          lastTxs.map((tx, i) => (
            <div key={i} className="transaction-row">
              <div><strong>Wallet:</strong> {tx.wallet?.slice(0, 6)}...{tx.wallet?.slice(-4)}</div>
              <div><strong>Amount:</strong> {formatUSDT(tx.amount)} USDT</div>
              <div><strong>Time:</strong> {tx.time}</div>
              <div><strong>Type:</strong> {tx.type}</div>
            </div>
          ))
        ) : (
          !loading && <div className="transaction-row">No transactions found</div>
        )}
      </div>
    </div>
  );
}