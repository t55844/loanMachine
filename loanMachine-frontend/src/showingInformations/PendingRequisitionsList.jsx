import { useState, useEffect } from "react";
import { ethers } from "ethers";
import { fetchLoanRequisitions, fetchUserDonations } from "../graphql-frontend-query";

export default function PendingRequisitionsList({ contract, account, onCoverLoan }) {
  const [requisitions, setRequisitions] = useState([]);
  const [selectedRequisition, setSelectedRequisition] = useState(null);
  const [customPercentage, setCustomPercentage] = useState("");
  const [loading, setLoading] = useState(true);
  const [covering, setCovering] = useState(false);
  const [error, setError] = useState("");
  const [userDonationBalance, setUserDonationBalance] = useState("0");

  // Quick percentage options + custom input
  const quickPercentages = [1, 3, 5, 10, 15, 20, 25, 33, 50, 75, 100];

  useEffect(() => {
    if (contract && account) {
      loadPendingRequisitions();
      loadUserDonationBalance();
    }
  }, [contract, account]);

  const loadUserDonationBalance = async () => {
    if (!account) return;
    
    try {
      const donations = await fetchUserDonations(account);
      const totalBalance = donations.reduce((total, donation) => {
        return total + parseFloat(ethers.utils.formatEther(donation.amount || "0"));
      }, 0);
      setUserDonationBalance(totalBalance.toString());
    } catch (err) {
      console.error("Error loading donation balance from subgraph:", err);
      setUserDonationBalance("0");
    }
  };

  const loadPendingRequisitions = async () => {
    if (!contract || !account) return;
    
    setLoading(true);
    setError("");
    
    try {
      const graphRequisitions = await fetchLoanRequisitions();
      const requisitionDetails = await Promise.all(
        graphRequisitions.map(async (graphReq) => {
          try {
            const requisitionId = parseInt(graphReq.requisitionId);
            const info = await contract.getRequisitionInfo(requisitionId);

            const safeNumber = (val) => {
              if (val && typeof val.toNumber === "function") return val.toNumber();
              if (typeof val === "bigint") return Number(val);
              return Number(val);
            };

            return {
              id: requisitionId.toString(),
              borrower: info.borrower,
              amount: ethers.utils.formatEther(info.amount),
              minimumCoverage: safeNumber(info.minimumCoverage),
              currentCoverage: safeNumber(info.currentCoverage),
              status: safeNumber(info.status),
              durationDays: safeNumber(info.durationDays),
              creationTime: new Date(safeNumber(info.creationTime) * 1000).toLocaleString(),
              coveringLendersCount: safeNumber(info.coveringLendersCount)
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
      console.error("Error loading pending requisitions:", err);
      setError("Failed to load available requisitions");
    } finally {
      setLoading(false);
    }
  };

  const handleCoverLoan = async (requisitionId, percentage) => {
    if (!contract || !account) {
      setError("Please connect your wallet first");
      return;
    }
    
    setCovering(true);
    setError("");
    
    try {
      const requisition = requisitions.find(r => r.id === requisitionId);
      if (!requisition) {
        throw new Error("Requisition not found");
      }

      const coverageAmount = parseFloat(requisition.amount) * percentage / 100;

      if (parseFloat(userDonationBalance) < coverageAmount) {
        throw new Error(`Insufficient donation balance. You have ${userDonationBalance} ETH but need ${coverageAmount.toFixed(4)} ETH`);
      }

      const tx = await contract.coverLoan(requisitionId, percentage);
      await tx.wait();
      
      if (onCoverLoan) {
        onCoverLoan();
      }
      
      setSelectedRequisition(null);
      setCustomPercentage("");
      await Promise.all([loadPendingRequisitions(), loadUserDonationBalance()]);
      
    } catch (err) {
      console.error("Error covering loan:", err);
      setError(err.message || "Failed to cover loan");
    } finally {
      setCovering(false);
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

  return (
    <div className="requsitionBlock">
      <h2>Available Loan Requisitions</h2>
      
      <div className="stats-box">
        <strong>Your Donation Balance:</strong> {parseFloat(userDonationBalance).toFixed(4)} ETH
      </div>
      
      {error && <div className="error-message">{error}</div>}
      
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
                <div><strong>Amount:</strong> {req.amount} ETH</div>
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
                    backgroundColor: req.currentCoverage >= req.minimumCoverage ? 
                      'var(--accent-green)' : 'var(--accent-blue)',
                  }}
                ></div>
              </div>
              
              {selectedRequisition?.id === req.id && (
                <div className="cover-loan-section" onClick={stopPropagation}>
                  <h4>Cover This Loan</h4>
                  
                  <div className="quick-percentages">
                    <p>Quick select:</p>
                    <div className="percentage-grid">
                      {quickPercentages.map((percentage) => {
                        const coverageAmount = parseFloat(req.amount) * percentage / 100;
                        const canCover = parseFloat(userDonationBalance) >= coverageAmount;
                        const remainingCoverage = 100 - req.currentCoverage;
                        const isValid = percentage <= remainingCoverage && percentage > 0;
                        
                        return (
                          <button
                            key={percentage}
                            onClick={() => isValid && handleCoverLoan(req.id, percentage)}
                            className={`percentage-button ${!isValid ? 'disabled' : ''} ${!canCover ? 'insufficient' : ''}`}
                            disabled={covering || !isValid || !canCover}
                            title={!isValid ? 
                              `Cannot cover more than ${remainingCoverage}%` : 
                              !canCover ? `Need ${coverageAmount.toFixed(4)} ETH` :
                              `Cover ${percentage}% (${coverageAmount.toFixed(4)} ETH)`
                            }
                          >
                            {percentage}%
                          </button>
                        );
                      })}
                    </div>
                  </div>
                  
                  <div className="custom-percentage">
                    <p>Or enter custom percentage (1-100):</p>
                    <div className="custom-input-group">
                      <input
                        type="number"
                        min="1"
                        max="100"
                        value={customPercentage}
                        onChange={(e) => setCustomPercentage(e.target.value)}
                        onClick={stopPropagation}
                        placeholder="Enter percentage"
                        className="custom-percentage-input"
                      />
                      <span>%</span>
                      <button
                        onClick={() => {
                          const percentage = parseInt(customPercentage);
                          if (percentage >= 1 && percentage <= 100) {
                            handleCoverLoan(req.id, percentage);
                          }
                        }}
                        disabled={covering || !customPercentage || parseInt(customPercentage) < 1 || parseInt(customPercentage) > 100}
                        className="custom-cover-button"
                      >
                        Cover
                      </button>
                    </div>
                    {customPercentage && (
                      <div className="custom-amount">
                        Coverage amount: {(parseFloat(req.amount) * parseInt(customPercentage) / 100).toFixed(4)} ETH
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
      
      <button onClick={loadPendingRequisitions} className="wallet-button refresh-button">
        Refresh List
      </button>
    </div>
  );
}