import { useState, useEffect } from "react";
import { ethers } from "ethers";

export default function LoanRequisitionMonitor({ contract, account }) {
  const [requisitions, setRequisitions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    loadRequisitions();
  }, [contract, account]);

  const loadRequisitions = async () => {
    if (!contract || !account) return;
    
    setLoading(true);
    setError("");
    
    try {
      // Get requisition IDs for the current user
      const requisitionIds = await contract.getBorrowerRequisitions(account);
      
      // Get details for each requisition
      const requisitionDetails = await Promise.all(
        requisitionIds.map(async (id) => {
          const info = await contract.getRequisitionInfo(id);

          const safeNumber = (val) => {
            if (val && typeof val.toNumber === "function") return val.toNumber(); // BigNumber case
            if (typeof val === "bigint") return Number(val); // BigInt case
            return Number(val); // String or number
          };

          return {
            id: id.toString(),
            amount: ethers.utils.formatUnits(info.amount, 6), // USDT (6 decimals)
            minimumCoverage: safeNumber(info.minimumCoverage),
            currentCoverage: safeNumber(info.currentCoverage),
            status: safeNumber(info.status),
            durationDays: safeNumber(info.durationDays),
            creationTime: new Date(safeNumber(info.creationTime) * 1000).toLocaleString(),
            coveringLendersCount: safeNumber(info.coveringLendersCount)
          };
        })
      );

      
      setRequisitions(requisitionDetails);
    } catch (err) {
      console.error("Error loading requisitions:", err);
      setError("Failed to load loan requisitions");
    } finally {
      setLoading(false);
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 0: return "Pending";
      case 1: return "Partially Covered";
      case 2: return "Fully Covered";
      case 3: return "Active";
      case 4: return "Repaid";
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
      default: return "var(--text-secondary)";
    }
  };

  // Format USDT amount for display
  const formatUSDT = (amount) => {
    return parseFloat(amount).toFixed(2);
  };

  return (
    <div style={{
      background: 'var(--bg-tertiary)',
      padding: '24px',
      borderRadius: '12px',
      border: '1px solid var(--border-color)',
      marginTop: '16px'
    }}>
      <h2>My Loan Requisitions</h2>
      
      {error && <div style={{
        color: 'var(--accent-red)', 
        marginBottom: '16px',
        padding: '12px',
        backgroundColor: 'rgba(242, 54, 69, 0.1)',
        borderRadius: '6px',
        border: '1px solid var(--accent-red)'
      }}>{error}</div>}
      
      {loading ? (
        <p>Loading requisitions...</p>
      ) : requisitions.length === 0 ? (
        <p>No loan requisitions found.</p>
      ) : (
        <div style={{ textAlign: 'left' }}>
          {requisitions.map((req) => (
            <div key={req.id} style={{
              border: '1px solid var(--border-color)',
              borderRadius: '8px',
              padding: '16px',
              marginBottom: '16px',
              backgroundColor: 'var(--bg-secondary)'
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
                <h3 style={{ margin: 0 }}>Requisition #{req.id}</h3>
                <span style={{ 
                  color: getStatusColor(req.status),
                  fontWeight: 'bold',
                  fontSize: '14px'
                }}>
                  {getStatusText(req.status)}
                </span>
              </div>
              
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', marginBottom: '12px' }}>
                <div>
                  <strong>Amount:</strong> {formatUSDT(req.amount)} USDT
                </div>
                <div>
                  <strong>Created:</strong> {req.creationTime}
                </div>
                <div>
                  <strong>Coverage:</strong> {req.currentCoverage}% / {req.minimumCoverage}%
                </div>
                <div>
                  <strong>Duration:</strong> {req.durationDays} days
                </div>
                <div>
                  <strong>Lenders:</strong> {req.coveringLendersCount}
                </div>
              </div>
              
              <div style={{ 
                height: '8px', 
                backgroundColor: 'var(--bg-primary)', 
                borderRadius: '4px',
                overflow: 'hidden',
                marginBottom: '12px'
              }}>
                <div style={{
                  height: '100%',
                  width: `${req.currentCoverage}%`,
                  backgroundColor: req.currentCoverage >= req.minimumCoverage ? 
                    'var(--accent-green)' : 'var(--accent-blue)',
                  transition: 'width 0.3s ease'
                }}></div>
              </div>
            </div>
          ))}
        </div>
      )}
      
      <button 
        onClick={loadRequisitions} 
        className="wallet-button"
        style={{ marginTop: '16px' }}
      >
        Refresh Requisitions
      </button>
    </div>
  );
}