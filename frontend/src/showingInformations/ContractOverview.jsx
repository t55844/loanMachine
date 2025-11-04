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
        console.error("Erro de carregamento do ContractOverview:", err);
        if (mounted) setError("Erro ao carregar estatísticas do contrato");
        
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
      <h2>Visão Geral do Contrato</h2>

      {loading && <p className="loading">Carregando dados...</p>}
      {error && <p className="error">{error}</p>}

      {stats && (
        <div className="stats-grid" style={{ marginTop: 8 }}>
          <div><strong>Doações Totais:</strong> {formatUSDT(stats.totalDonations)} USDT</div>
          <div><strong>Total Emprestado:</strong> {formatUSDT(stats.totalBorrowed)} USDT</div>
          <div><strong>Saldo Disponível:</strong> {formatUSDT(stats.availableBalance)} USDT</div>
          <div><strong>Saldo do Contrato:</strong> {formatUSDT(stats.contractBalance)} USDT</div>
        </div>
      )}

      <h3 style={{ marginTop: 18 }}>Últimas 5 Transações</h3>

      <div className="transactions-box" style={{ marginTop: 12 }}>
        {lastTxs.length > 0 ? (
          lastTxs.map((tx, i) => (
            <div key={i} className="transaction-row">
              <div><strong>Carteira:</strong> {tx.wallet?.slice(0, 6)}...{tx.wallet?.slice(-4)}</div>
              <div><strong>Valor:</strong> {formatUSDT(tx.amount)} USDT</div>
              <div><strong>Horário:</strong> {tx.time}</div>
              <div><strong>Tipo:</strong> {tx.type}</div>
            </div>
          ))
        ) : (
          !loading && <div className="transaction-row">Nenhuma transação encontrada</div>
        )}
      </div>
    </div>
  );
}