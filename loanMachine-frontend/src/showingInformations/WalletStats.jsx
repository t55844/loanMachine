import { useState, useEffect } from "react";
import { fetchUserData } from "../graphql-frontend-query";
import { useWeb3 } from "../Web3Context";
import Donate from "../loan-interaction/Donate";
import { ethers } from "ethers";

async function fetchUserStats(userAddress) {
  const userData = await fetchUserData(userAddress);
  
  const totalDonations = ethers.utils.formatEther(userData.totalDonated || "0");
  const totalBorrowed = ethers.utils.formatEther(userData.totalBorrowed || "0");
  const currentDebt = ethers.utils.formatEther(userData.currentDebt || "0");
  
  const lastActivity = userData.lastActivity !== "0" 
    ? new Date(parseInt(userData.lastActivity) * 1000).toLocaleString()
    : "Never";
  
  const canBorrowNow = parseFloat(currentDebt) === 0;
  
  return {
    donations: totalDonations,
    borrowings: totalBorrowed,
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
  
  const { account,contract } = useWeb3();

  useEffect(() => {
    if (account) {
      loadUserData();
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

      {loading && <p>Loading...</p>}
      {error && <p className="error">{error}</p>}

      {userData && (
        <div className="stats-grid">
          <div><strong>Donations:</strong> {userData.donations} ETH</div>
          <div><strong>Borrowed:</strong> {userData.borrowings} ETH</div>
          <div><strong>Current Debt:</strong> {userData.currentDebt} ETH</div>
          <div><strong>Can Borrow:</strong> {userData.canBorrowNow ? "Yes" : "No"}</div>
          <div><strong>Last Activity:</strong> {userData.lastActivity}</div>
          <div><strong>Donations Made:</strong> {userData.donationCount}</div>
        </div>
      )}

      <Donate account={account} contract={contract} />

      <button onClick={loadUserData} className="refresh-button">
        Refresh Data
      </button>
    </div>
  );
}