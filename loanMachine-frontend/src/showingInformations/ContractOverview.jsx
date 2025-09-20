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
          totalDonations: ethers.utils.formatEther(rawStats.totalDonations ?? "0"),
          totalBorrowed: ethers.utils.formatEther(rawStats.totalBorrowed ?? "0"),
          availableBalance: ethers.utils.formatEther(rawStats.availableBalance ?? "0"),
          contractBalance: ethers.utils.formatEther(rawStats.availableBalance ?? "0")
        });

        // Get last transactions
        const rawTxs = await fetchLastTransactions({ limit: 5 });
        if (!mounted) return;

        const mapped = rawTxs.map((tx) => ({
          wallet: tx.donor?.id || tx.borrower?.id || "unknown",
          amount: ethers.utils.formatEther(tx.amount ?? "0"),
          time: new Date(Number(tx.timestamp) * 1000).toLocaleString(),
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

  return (
    <div className="graphBlock contract-overview">
      <h2>Contract Overview</h2>

      {loading && <p className="loading">Loading data...</p>}
      {error && <p className="error">{error}</p>}

      {stats && (
        <div className="stats-grid" style={{ marginTop: 8 }}>
          <div><strong>Total Donations:</strong> {stats.totalDonations} ETH</div>
          <div><strong>Total Borrowed:</strong> {stats.totalBorrowed} ETH</div>
          <div><strong>Available Balance:</strong> {stats.availableBalance} ETH</div>
          <div><strong>Contract Balance:</strong> {stats.contractBalance} ETH</div>
        </div>
      )}

      <h3 style={{ marginTop: 18 }}>Last 5 Transactions</h3>

      <div className="transactions-box" style={{ marginTop: 12 }}>
        {lastTxs.length > 0 ? (
          lastTxs.map((tx, i) => (
            <div key={i} className="transaction-row">
              <div><strong>Wallet:</strong> {tx.wallet?.slice(0, 6)}...{tx.wallet?.slice(-4)}</div>
              <div><strong>Amount:</strong> {tx.amount} ETH</div>
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