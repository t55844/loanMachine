// PendingRequisitionsList.jsx
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
  const quickPercentages = [1, 3, 5, 10, 15, 20, 25, 33, 50, 75, 100];

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
      const graphRequisitions = await fetchLoanRequisitions();
      const requisitionDetails = await Promise.all(
        graphRequisitions.map(async (graphReq) => {
          try {
            const requisitionId = parseInt(graphReq.requisitionId);
            const info = await contract.getRequisitionInfo(requisitionId);
            
            const safeNumber = (val) => {
              if (val && typeof val.toNumber === "function") return val.toNumber();
              if (typeof val === 'bigint') return Number(val);
              return Number(val);
            };

            let coveringLendersCount = 0;
            try {
              const coveringLenders = info.coveringLenders;
              
              if (Array.isArray(coveringLenders)) {
                coveringLendersCount = coveringLenders.filter(addr => 
                  addr && addr !== ethers.constants.AddressZero
                ).length;
              } else if (typeof coveringLenders === 'string' && coveringLenders.startsWith('0x')) {
                coveringLendersCount = coveringLenders !== ethers.constants.AddressZero ? 1 : 0;
              } else {
                coveringLendersCount = safeNumber(coveringLenders);
              }
              
              if (isNaN(coveringLendersCount) || coveringLendersCount < 0) {
                coveringLendersCount = 0;
              }
            } catch (countErr) {
              console.warn(`Error parsing coveringLenders for requisition ${requisitionId}:`, countErr);
              coveringLendersCount = 0;
            }

            return {
              id: requisitionId.toString(),
              borrower: info.borrower,
              amount: ethers.utils.formatUnits(info.amount, 6),
              minimumCoverage: safeNumber(info.minimumCoverage),
              currentCoverage: safeNumber(info.currentCoverage),
              status: safeNumber(info.status),
              durationDays: safeNumber(info.durationDays),
              creationTime: new Date(safeNumber(info.creationTime) * 1000).toLocaleString(),
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
        .filter(req => req.status === 0 || req.status === 1);

      setRequisitions(pendingRequisitions);
    } catch (err) {
      showToast("Failed to load available requisitions");
    } finally {
      setLoading(false);
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
    return parseFloat(amount).toFixed(2);
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
                  member={member} // Pass member to child component
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