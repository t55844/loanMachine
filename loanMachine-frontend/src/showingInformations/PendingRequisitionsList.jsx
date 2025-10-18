import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { fetchLoanRequisitions, fetchUserDonations } from "../graphql-frontend-query";
import { useToast } from "../handlers/useToast";
import Toast from "../handlers/Toast";
import TransactionPendingRequisition from "./TransactionPendingRequisition";

export default function PendingRequisitionsList({ contract, account, onCoverLoan, member }) {
  const [requisitions, setRequisitions] = useState([]);
  const [selectedRequisition, setSelectedRequisition] = useState(null);
  const [customPercentage, setCustomPercentage] = useState("");
  const [loading, setLoading] = useState(true);
  const [donationBalances, setDonationBalances] = useState({
    total: "0",
    allocated: "0",
    free: "0"
  });

  const { showToast, showSuccess, showError, handleContractError } = useToast();
  const quickPercentages = [10, 15, 20, 25, 33, 50, 75, 100];

  useEffect(() => {
    if (contract && account) {
      loadPendingRequisitions();
      loadUserDonationBalances();
    }
  }, [contract, account]);

  const loadUserDonationBalances = async () => {
    if (!account || !contract) return;
    try {
      const donations = await fetchUserDonations(account);
      const totalBalance = donations.reduce((total, donation) => {
        return total + parseFloat(ethers.utils.formatUnits(donation.amount || "0", 6));
      }, 0);

      const allocatedWei = await contract.getDonationsInCoverage(account);
      const allocatedBalance = parseFloat(ethers.utils.formatUnits(allocatedWei, 6));
      const freeBalance = totalBalance - allocatedBalance;

      setDonationBalances({
        total: totalBalance.toString(),
        allocated: allocatedBalance.toString(),
        free: Math.max(0, freeBalance).toString()
      });
    } catch (err) {
      console.error("Error loading donation balances:", err);
      setDonationBalances({ total: "0", allocated: "0", free: "0" });
    }
  };

  const loadPendingRequisitions = async () => {
    if (!contract || !account) return;
    setLoading(true);
    try {
      // Use GraphQL to get pending requisitions
      const graphRequisitions = await fetchLoanRequisitions();
      console.log("GraphQL requisitions:", graphRequisitions);
      
      const requisitionDetails = await Promise.all(
        graphRequisitions.map(async (graphReq) => {
          try {
            const requisitionId = parseInt(graphReq.requisitionId);
            
            // Fallback to contract for additional data
            let contractData = {};
            try {
              const info = await contract.getRequisitionInfo(requisitionId);
              contractData = {
                borrower: info.borrower,
                minimumCoverage: safeNumber(info.minimumCoverage),
                currentCoverage: safeNumber(info.currentCoverage),
                status: safeNumber(info.status),
                durationDays: safeNumber(info.durationDays),
                creationTime: new Date(safeNumber(info.creationTime) * 1000).toLocaleString(),
              };
            } catch (contractErr) {
              console.warn(`Could not get contract data for requisition ${requisitionId}:`, contractErr);
              // Use GraphQL data as fallback
              contractData = {
                borrower: graphReq.borrower,
                minimumCoverage: graphReq.minimumCoverage || 0,
                currentCoverage: graphReq.currentCoveragePercentage || 0,
                status: getStatusNumber(graphReq.status),
                durationDays: graphReq.durationDays || 0,
                creationTime: new Date(parseInt(graphReq.creationTime) * 1000).toLocaleString(),
              };
            }

            let coveringLendersCount = graphReq.coveringLendersCount || 0;

            // Handle amount conversion
            let amount;
            try {
              if (typeof graphReq.amount === 'string') {
                amount = ethers.utils.formatUnits(graphReq.amount, 6);
              } else {
                amount = ethers.utils.formatUnits(graphReq.amount.toString(), 6);
              }
            } catch (e) {
              console.error("Error formatting amount:", e);
              amount = "0";
            }

            return {
              id: requisitionId.toString(),
              borrower: contractData.borrower,
              amount: amount,
              minimumCoverage: contractData.minimumCoverage,
              currentCoverage: contractData.currentCoverage,
              status: contractData.status,
              durationDays: contractData.durationDays,
              creationTime: contractData.creationTime,
              coveringLendersCount: coveringLendersCount
            };
          } catch (err) {
            console.error(`Error loading requisition ${graphReq.requisitionId}:`, err);
            return null;
          }
        })
      );

      const pendingRequisitions = requisitionDetails
        .filter(req => req !== null)
        .filter(req => req.status === 0 || req.status === 1); // Pending or Partially Covered

      console.log("Pending requisitions:", pendingRequisitions);
      setRequisitions(pendingRequisitions);
    } catch (err) {
      console.error("Error loading pending requisitions:", err);
      showToast("Failed to load available requisitions");
    } finally {
      setLoading(false);
    }
  };

  // Helper function for safe number conversion
  const safeNumber = (val) => {
    try {
      if (val && typeof val.toNumber === "function") return val.toNumber();
      if (typeof val === 'bigint') return Number(val);
      if (typeof val === 'string') return parseInt(val);
      return Number(val);
    } catch (e) {
      console.warn("Error converting number:", val, e);
      return 0;
    }
  };

  // Helper function to convert status string to number
  const getStatusNumber = (status) => {
    if (typeof status === 'number') return status;
    
    switch (status) {
      case "Pending": return 0;
      case "PartiallyCovered": return 1;
      case "FullyCovered": return 2;
      case "Active": return 3;
      case "Completed": return 4;
      case "Defaulted": return 5;
      case "Cancelled": return 6;
      default: return 0;
    }
  };

  const stopPropagation = (e) => {
    e.stopPropagation();
  };

  const getStatusText = (status) => {
    switch (status) {
      case 0: return "Pending";
      case 1: return "Partially Covered";
      case 2: return "Fully Covered";
      case 3: return "Active";
      case 4: return "Repaid";
      case 5: return "Defaulted";
      default: return "Unknown";
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 0: return "var(--text-secondary)";
      case 1: return "var(--accent-blue)";
      case 2: return "var(--accent-green)";
      case 3: return "var(--accent-blue)";
      case 4: return "var(--accent-green)";
      case 5: return "var(--accent-red)";
      default: return "var(--text-secondary)";
    }
  };

  const formatAddress = (address) => {
    if (!address) return "Unknown";
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const isPercentageValid = (requisition, percentage) => {
    if (!requisition) return false;
    
    const coverageAmount = parseFloat(requisition.amount) * percentage / 100;
    const remainingCoverage = 100 - requisition.currentCoverage;
    const canCover = parseFloat(donationBalances.free) >= coverageAmount;
    const isValid = percentage <= remainingCoverage && percentage > 0;

    return isValid && canCover;
  };

  const formatUSDT = (amount) => {
    const num = parseFloat(amount);
    return isNaN(num) ? "0.00" : num.toFixed(2);
  };

  // Check if member data is available
  const hasMemberData = member && member.id;

  return (
    <div className="requsitionBlock">
      <Toast />
      
      <h2>Available Loan Requisitions</h2>

      {/* Show warning if no member data */}
      {!member && account && (
        <div style={{
          background: 'var(--warning-bg)',
          padding: '12px',
          borderRadius: '8px',
          marginBottom: '16px',
          border: '1px solid var(--warning-color)'
        }}>
          <p style={{ margin: 0, fontSize: '14px', color: 'var(--warning-color)' }}>
            ⚠️ Member data not loaded. Please check your wallet connection.
          </p>
        </div>
      )}

      {/* Donation Balance Display */}
      <div className="stats-box">
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: '16px', textAlign: 'center' }}>
          <div>
            <strong>Total Donations:</strong><br />
            {formatUSDT(donationBalances.total)} USDT
          </div>
          <div>
            <strong>In Coverage:</strong><br />
            <span style={{ color: 'var(--text-secondary)' }}>
              {formatUSDT(donationBalances.allocated)} USDT
            </span>
          </div>
          <div>
            <strong>Available:</strong><br />
            <span style={{ color: 'var(--accent-green)', fontWeight: 'bold' }}>
              {formatUSDT(donationBalances.free)} USDT
            </span>
          </div>
        </div>
        {parseFloat(donationBalances.free) === 0 && parseFloat(donationBalances.total) > 0 && (
          <div style={{ 
            marginTop: '12px', 
            padding: '8px', 
            backgroundColor: 'rgba(56, 97, 251, 0.1)', 
            borderRadius: '4px',
            fontSize: '0.9em',
            color: 'var(--accent-blue)'
          }}>
            💡 All your donations are currently allocated to loan coverage
          </div>
        )}
      </div>

      {loading ? (
        <p>Loading available requisitions...</p>
      ) : requisitions.length === 0 ? (
        <p>No available requisitions found.</p>
      ) : (
        <div className="requisitions-list">
          {requisitions.map((req) => (
            <div
              key={req.id}
              className="requisition-item"
              onClick={() => {
                setSelectedRequisition(selectedRequisition?.id === req.id ? null : req);
                setCustomPercentage("");
              }}
            >
              <div className="requisition-header">
                <h3>Requisition #{req.id}</h3>
                <span
                  className="status-badge"
                  style={{ color: getStatusColor(req.status) }}
                >
                  {getStatusText(req.status)}
                </span>
              </div>

              <div className="requisition-details">
                <div><strong>Amount:</strong> {formatUSDT(req.amount)} USDT</div>
                <div><strong>Borrower:</strong> {formatAddress(req.borrower)}</div>
                <div><strong>Coverage:</strong> {req.currentCoverage}% / {req.minimumCoverage}%</div>
                <div><strong>Duration:</strong> {req.durationDays} days</div>
                <div><strong>Lenders:</strong> {req.coveringLendersCount}</div>
                <div><strong>Created:</strong> {req.creationTime}</div>
              </div>

              <div className="coverage-bar-container">
                <div
                  className="coverage-bar-fill"
                  style={{
                    width: `${Math.min(req.currentCoverage, 100)}%`,
                    backgroundColor: req.currentCoverage >= req.minimumCoverage ? 'var(--accent-green)' : 'var(--accent-blue)',
                  }}
                ></div>
              </div>

              {selectedRequisition?.id === req.id && (
                <TransactionPendingRequisition
                  requisition={req}
                  donationBalances={donationBalances}
                  customPercentage={customPercentage}
                  setCustomPercentage={setCustomPercentage}
                  quickPercentages={quickPercentages}
                  contract={contract}
                  account={account}
                  member={member}
                  onCoverLoan={onCoverLoan}
                  onRefresh={() => {
                    loadPendingRequisitions();
                    loadUserDonationBalances();
                  }}
                  stopPropagation={stopPropagation}
                  isPercentageValid={isPercentageValid}
                  formatUSDT={formatUSDT}
                  showToast={showToast}
                  showSuccess={showSuccess}
                  showError={showError}
                  handleContractError={handleContractError}
                />
              )}
            </div>
          ))}
        </div>
      )}

      <button onClick={() => {
        loadPendingRequisitions();
        loadUserDonationBalances();
      }} className="wallet-button refresh-button">
        Refresh List & Balances
      </button>
    </div>
  );
}