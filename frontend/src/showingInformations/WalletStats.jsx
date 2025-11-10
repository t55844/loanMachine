import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { fetchUserData } from "../graphql-frontend-query"; // Use GraphQL
import { useWeb3 } from "../Web3Context";
import DonationWithdrawTabs from "../showingInformations/DonationWithdrawTabs";

export default function UserStatus() {
  const [userData, setUserData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  
  const { account, getUSDTBalance } = useWeb3();

  useEffect(() => {
    if (account) {
      loadUserData();
      loadUSDTBalance();
    }
  }, [account]);

  async function loadUserData() {
    if (!account) return;

    setLoading(true);
    setError("");

    try {
      const data = await fetchUserData(account);
      
      // FIXED: Sum all events for accurate totals (GraphQL first:1 was incomplete)
      const totalDonatedWei = data.donations.reduce((sum, d) => sum + parseInt(d.amount || "0"), 0);
      const totalBorrowedWei = data.borrows.reduce((sum, b) => sum + parseInt(b.amount || "0"), 0);
      const currentDebtWei = parseInt(data.currentDebt || "0");
      
      // FIXED: Format with 6 decimals for USDT
      const totalDonated = ethers.utils.formatUnits(totalDonatedWei.toString(), 6);
      const totalBorrowed = ethers.utils.formatUnits(totalBorrowedWei.toString(), 6);
      const currentDebt = ethers.utils.formatUnits(currentDebtWei.toString(), 6);
      
      const canBorrowNow = parseFloat(currentDebt) === 0;

      setUserData({
        donations: totalDonated,
        borrowings: totalBorrowed,
        currentDebt,
        canBorrowNow,
        borrowCount: data.borrows.length,
      });
    } catch (e) {
      //console.error("Erro ao carregar dados do usuário:", e);
      setError("Falha ao carregar dados do usuário");
    } finally {
      setLoading(false);
    }
  }

  async function loadUSDTBalance() {
    if (!account) return;
    
    try {
      const balance = await getUSDTBalance();
      setUsdtBalance(balance);
    } catch (e) {
      //console.error("Erro ao carregar saldo USDT:", e);
    }
  }

  const refreshAllData = async () => {
    await Promise.all([loadUserData(), loadUSDTBalance()]);
  };

  // Format USDT amounts for display
  const formatUSDT = (amount) => {
    return parseFloat(amount).toFixed(2);
  };

  if (!account) {
    return (
      <div className="stats-box">
        <h2>Status do Usuário</h2>
        <p>Conecte sua carteira para ver o status</p>
      </div>
    );
  }

  return (
    <div className="stats-box">
      <h2>Status do Usuário</h2>
      
      <div className="user-address">
        <strong>Conectado:</strong> {account}
      </div>

      {/* USDT Balance Display */}
      <div className="usdt-balance-section">
        <div className="balance-card">
          <strong>Seu Saldo USDT:</strong> 
          <span className="balance-amount">{formatUSDT(usdtBalance)} USDT</span>
        </div>
      </div>

      {loading && <p>Carregando dados do usuário...</p>}
      {error && <p className="error">{error}</p>}

      {userData && (
        <div className="stats-grid">
          <div className="stat-item">
            <strong>Doações do Usuário:</strong> 
            <span>{formatUSDT(userData.donations)} USDT</span>
          </div>
          <div className="stat-item">
            <strong>Valor Emprestado:</strong> 
            <span>{formatUSDT(userData.borrowings)} USDT</span>
          </div>
          <div className="stat-item">
            <strong>Dívida Atual:</strong> 
            <span className={parseFloat(userData.currentDebt) > 0 ? "debt-amount" : ""}>
              {formatUSDT(userData.currentDebt)} USDT
            </span>
          </div>
          <div className="stat-item">
            <strong>Pode Pegar Empréstimo:</strong> 
            <span className={userData.canBorrowNow ? "can-borrow-yes" : "can-borrow-no"}>
              {userData.canBorrowNow ? "Sim" : "Não"}
            </span>
          </div>
          <div className="stat-item">
            <strong>Empréstimos Realizados:</strong> 
            <span>{userData.borrowCount}</span>
          </div>
        </div>
      )}

      <button onClick={refreshAllData} className="refresh-button" disabled={loading}>
        {loading ? "Atualizando..." : "Atualizar Dados"}
      </button>

      {/* Replace Donate with Tabbed Interface */}
      <DonationWithdrawTabs />
    </div>
  );
}