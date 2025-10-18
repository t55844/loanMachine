import { useState, useEffect } from "react";
import { fetchUserData } from "../graphql-frontend-query";
import { useWeb3 } from "../Web3Context";
import DonationWithdrawTabs from "../showingInformations/DonationWithdrawTabs";
import { ethers } from "ethers";

async function fetchUserStats(userAddress) {
  const userData = await fetchUserData(userAddress);
  
  // Convert from wei (6 decimals for USDT) to USDT
  const userDonations = ethers.utils.formatUnits(userData.totalDonated || "0", 6);
  const userBorrowed = ethers.utils.formatUnits(userData.totalBorrowed || "0", 6);
  const currentDebt = ethers.utils.formatUnits(userData.currentDebt || "0", 6);
  
  const lastActivity = userData.lastActivity !== "0" 
    ? userData.lastActivity
    : "Never";
  console.log(userData)
  const canBorrowNow = parseFloat(currentDebt) === 0;
  
  return {
    donations: userDonations,
    borrowings: userBorrowed,
    currentDebt,
    lastActivity,
    canBorrowNow,
    donationCount: userData.donations.length,
    borrowCount: userData.borrows.length,
  };
}

export default function UserStatus() {
  const [userData, setUserData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [usdtBalance, setUsdtBalance] = useState("0");
  
  const { account, contract, getUSDTBalance } = useWeb3();

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
      const stats = await fetchUserStats(account);
      setUserData(stats);
    } catch (e) {
      console.error("Error loading user data:", e);
      setError("Failed to load user data");
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
      console.error("Error loading USDT balance:", e);
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
        <h2>User Status</h2>
        <p>Connect your wallet to view status</p>
      </div>
    );
  }

  return (
    <div className="stats-box">
      <h2>User Status</h2>
      
      <div className="user-address">
        <strong>Connected:</strong> {account}
      </div>

      {/* USDT Balance Display */}
      <div className="usdt-balance-section">
        <div className="balance-card">
          <strong>Your USDT Balance:</strong> 
          <span className="balance-amount">{formatUSDT(usdtBalance)} USDT</span>
        </div>
      </div>

      {loading && <p>Loading user data...</p>}
      {error && <p className="error">{error}</p>}

      {userData && (
        <div className="stats-grid">
          <div className="stat-item">
            <strong>User Donations:</strong> 
            <span>{formatUSDT(userData.donations)} USDT</span>
          </div>
          <div className="stat-item">
            <strong>User Borrowed:</strong> 
            <span>{formatUSDT(userData.borrowings)} USDT</span>
          </div>
          <div className="stat-item">
            <strong>Current Debt:</strong> 
            <span className={parseFloat(userData.currentDebt) > 0 ? "debt-amount" : ""}>
              {formatUSDT(userData.currentDebt)} USDT
            </span>
          </div>
          <div className="stat-item">
            <strong>Can Borrow:</strong> 
            <span className={userData.canBorrowNow ? "can-borrow-yes" : "can-borrow-no"}>
              {userData.canBorrowNow ? "Yes" : "No"}
            </span>
          </div>
          <div className="stat-item">
            <strong>Last Activity:</strong> 
            <span>{userData.lastActivity}</span>
          </div>
          <div className="stat-item">
            <strong>User Donations Made:</strong> 
            <span>{userData.donationCount}</span>
          </div>
          <div className="stat-item">
            <strong>User Loans Taken:</strong> 
            <span>{userData.borrowCount}</span>
          </div>
        </div>
      )}

      <button onClick={refreshAllData} className="refresh-button" disabled={loading}>
        {loading ? "Refreshing..." : "Refresh Data"}
      </button>

      {/* Replace Donate with Tabbed Interface */}
      <DonationWithdrawTabs />
    </div>
  );
}