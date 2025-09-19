// ContractOverview.jsx
import { useEffect, useState } from "react";
import { ethers } from "ethers";
import { GetAllData } from "../graphql-frontend-query"; // existing function you already use

const CONTRACT_ADDRESS = import.meta.env.VITE_CONTRACT_ADDRESS;

/**
 * Expected optional exports from ../graphql-frontend-query (if present):
 * - fetchContractStats() -> { totalDonations, totalBorrowed, availableBalance, contractBalance } (values in wei as strings)
 * - fetchLastTransactions({ limit }) -> [{ wallet, amount, timestamp, type }, ...] (amount in wei, timestamp unix seconds or ISO)
 *
 * If they don't exist, this component will compute totals from GetAllData() and use donations as fallback tx list.
 */

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
        // dynamic import to see if optional helpers exist
        let fetchContractStats = null;
        let fetchLastTransactions = null;
        try {
          const mod = await import("../graphql-frontend-query");
          fetchContractStats = mod.fetchContractStats;
          fetchLastTransactions = mod.fetchLastTransactions;
        } catch (err) {
          // module import should succeed because GetAllData is imported statically,
          // but in case of errors we will fallback to donations only.
        }

        // If contract stats helper exists, use it. Otherwise compute from donations.
        if (fetchContractStats) {
          const rawStats = await fetchContractStats();
          if (!mounted) return;

          setStats({
            totalDonations: ethers.utils.formatEther(rawStats.totalDonations ?? "0"),
            totalBorrowed: ethers.utils.formatEther(rawStats.totalBorrowed ?? "0"),
            availableBalance: ethers.utils.formatEther(rawStats.availableBalance ?? "0"),
            contractBalance: ethers.utils.formatEther(rawStats.contractBalance ?? "0")
          });
        } else {
          // fallback: compute totals from donations list
          const donations = await GetAllData();
          if (!mounted) return;

          // sum donations using BigNumber to avoid precision issues
          let totalDonationsBN = ethers.BigNumber.from(0);
          donations.forEach((d) => {
            try {
              totalDonationsBN = totalDonationsBN.add(ethers.BigNumber.from(d.amount));
            } catch {
              // ignore malformed amounts
            }
          });

          setStats({
            totalDonations: ethers.utils.formatEther(totalDonationsBN),
            totalBorrowed: "0.0", // unknown in fallback
            availableBalance: "0.0", // unknown in fallback
            contractBalance: "0.0" // unknown in fallback
          });
        }

        // If there's a dedicated last transactions query, use it. Otherwise fallback to last donation entries.
        if (fetchLastTransactions) {
          const rawTxs = await fetchLastTransactions({ limit: 5 });
          if (!mounted) return;

          const mapped = (rawTxs || []).slice(-5).reverse().map((tx) => ({
            wallet: tx.wallet ?? tx.from ?? tx.donor?.id ?? tx.fromAddress ?? "unknown",
            amount: ethers.utils.formatEther(tx.amount ?? tx.value ?? "0"),
            // support different timestamp formats: numeric unix seconds, blockTimestamp, or ISO
            time:
              tx.timestamp
                ? (isNaN(Number(tx.timestamp)) ? tx.timestamp : new Date(Number(tx.timestamp) * 1000).toLocaleString())
                : tx.blockTimestamp
                  ? new Date(Number(tx.blockTimestamp) * 1000).toLocaleString()
                  : tx.createdAt
                    ? new Date(tx.createdAt).toLocaleString()
                    : "",
            type: tx.type ?? tx.txType ?? (tx.to && tx.to.toLowerCase() === CONTRACT_ADDRESS?.toLowerCase() ? "donation" : "borrow")
          }));
          console.log(mapped)
          setLastTxs(mapped);
        } else {
          // fallback: use last donations from GetAllData
          const donations = await GetAllData();
          if (!mounted) return;

          const lastFive = (donations || []).slice(-5).reverse().map((d) => ({
            wallet: d.donor?.id ?? d.from ?? "unknown",
            amount: ethers.utils.formatEther(d.amount ?? "0"),
            time: d.timestamp
              ? (isNaN(Number(d.timestamp)) ? d.timestamp : new Date(Number(d.timestamp) * 1000).toLocaleString())
              : d.blockTimestamp
                ? new Date(Number(d.blockTimestamp) * 1000).toLocaleString()
                : "",
            type: "donation"
          }));

          setLastTxs(lastFive);
        }
      } catch (err) {
        console.error("ContractOverview load error:", err);
        if (mounted) setError("Erro ao carregar estatísticas (ver console)");
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

      {loading && <p className="loading">Carregando dados...</p>}
      {error && <p className="error">{error}</p>}

      {stats && (
        <div className="stats-grid" style={{ marginTop: 8 }}>
          <div><strong>Total Donations:</strong> {stats.totalDonations} ETH</div>
          <div><strong>Total Borrowed:</strong> {stats.totalBorrowed}</div>
          <div><strong>Available Balance:</strong> {stats.availableBalance} ETH</div>
          <div><strong>Contract Balance:</strong> {stats.contractBalance} ETH</div>
        </div>
      )}

      <h3 style={{ marginTop: 18 }}>Últimas 5 Transações</h3>

      <div className="transactions-box" style={{ marginTop: 12 }}>
        {lastTxs.length > 0 ? (
          lastTxs.map((tx, i) => (
            <div key={i} className="transaction-row">
              <div><strong>Wallet:</strong> {tx.wallet?.slice?.(0, 6) ?? tx.wallet}...{tx.wallet?.slice?.(-4)}</div>
              <div><strong>Valor:</strong> {tx.amount} ETH</div>
              <div><strong>Data:</strong> {tx.time}</div>
            </div>
          ))
        ) : (
          <div className="transaction-row">
            <div colSpan="3">Nenhuma transação encontrada</div>
          </div>
        )}
      </div>
    </div>
  );
}
