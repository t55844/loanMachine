import { useState, useEffect } from "react";
import { ethers } from "ethers";

export default function PendingRequisitionsList({ contract, account, onCoverLoan }) {
  const [requisitions, setRequisitions] = useState([]);
  const [selectedRequisition, setSelectedRequisition] = useState(null);
  const [coveragePercentage, setCoveragePercentage] = useState("10");
  const [loading, setLoading] = useState(true);
  const [covering, setCovering] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    loadPendingRequisitions();
  }, [contract]);

  const loadPendingRequisitions = async () => {
    if (!contract) return;
    
    setLoading(true);
    setError("");
    
    try {
      // In a real implementation, you would query all requisitions and filter by status
      // For this example, we'll simulate getting pending requisitions
      // This would need to be implemented based on your subgraph or contract events
      
      // Simulated data - replace with actual implementation
      const simulatedRequisitions = [
        {
          id: "1",
          borrower: "0x1234...abcd",
          amount: "1.5",
          minimumCoverage: 80,
          currentCoverage: 45,
          status: 1, // Partially Covered
          creationTime: new Date().toLocaleString()
        },
        {
          id: "2",
          borrower: "0x5678...efgh",
          amount: "2.0",
          minimumCoverage: 75,
          currentCoverage: 0,
          status: 0, // Pending
          creationTime: new Date().toLocaleString()
        }
      ];
      
      setRequisitions(simulatedRequisitions);
    } catch (err) {
      console.error("Error loading pending requisitions:", err);
      setError("Failed to load pending requisitions");
    } finally {
      setLoading(false);
    }
  };

  const handleCoverLoan = async (requisitionId) => {
    if (!contract || !account) {
      setError("Please connect your wallet first");
      return;
    }
    
    setCovering(true);
    setError("");
    
    try {
      const tx = await contract.coverLoan(
        requisitionId,
        parseInt(coveragePercentage)
      );
      
      await tx.wait();
      
      if (onCoverLoan) {
        onCoverLoan();
      }
      
      setSelectedRequisition(null);
      loadPendingRequisitions();
      
    } catch (err) {
      console.error("Error covering loan:", err);
      setError(err.message || "Failed to cover loan");
    } finally {
      setCovering(false);
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 0: return "Pending";
      case 1: return "Partially Covered";
      case 2: return "Fully Covered";
      default: return "Unknown";
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 0: return "var(--text-secondary)";
      case 1: return "var(--accent-blue)";
      case 2: return "var(--accent-green)";
      default: return "var(--text-secondary)";
    }
  };

  return (
    <div style={{
      background: 'var(--bg-tertiary)',
      padding: '24px',
      borderRadius: '12px',
      border: '1px solid var(--border-color)',
      marginTop: '16px'
    }}>
      <h2>Pending Loan Requisitions</h2>
      
      {error && <div style={{
        color: 'var(--accent-red)', 
        marginBottom: '16px',
        padding: '12px',
        backgroundColor: 'rgba(242, 54, 69, 0.1)',
        borderRadius: '6px',
        border: '1px solid var(--accent-red)'
      }}>{error}</div>}
      
      {loading ? (
        <p>Loading pending requisitions...</p>
      ) : requisitions.length === 0 ? (
        <p>No pending requisitions found.</p>
      ) : (
        <div style={{ textAlign: 'left' }}>
          {requisitions.map((req) => (
            <div key={req.id} style={{
              border: '1px solid var(--border-color)',
              borderRadius: '8px',
              padding: '16px',
              marginBottom: '16px',
              backgroundColor: 'var(--bg-secondary)',
              cursor: 'pointer',
              transition: 'all 0.2s ease'
            }}
            onClick={() => setSelectedRequisition(selectedRequisition?.id === req.id ? null : req)}
            >
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
                  <strong>Amount:</strong> {req.amount} ETH
                </div>
                <div>
                  <strong>Borrower:</strong> {req.borrower.slice(0, 6)}...{req.borrower.slice(-4)}
                </div>
                <div>
                  <strong>Coverage:</strong> {req.currentCoverage}% / {req.minimumCoverage}%
                </div>
                <div>
                  <strong>Created:</strong> {req.creationTime}
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
              
              {selectedRequisition?.id === req.id && (
                <div style={{ 
                  padding: '16px', 
                  backgroundColor: 'var(--bg-tertiary)', 
                  borderRadius: '8px',
                  marginTop: '12px'
                }}>
                  <h4 style={{ margin: '0 0 12px 0' }}>Cover This Loan</h4>
                  
                  <div style={{ marginBottom: '12px' }}>
                    <label htmlFor={`coverage-${req.id}`} style={{display: 'block', marginBottom: '8px'}}>
                      Coverage Percentage (%)
                    </label>
                    <select
                      id={`coverage-${req.id}`}
                      value={coveragePercentage}
                      onChange={(e) => setCoveragePercentage(e.target.value)}
                      className="donate-select"
                      style={{width: '100%'}}
                    >
                      <option value="5">5%</option>
                      <option value="10">10%</option>
                      <option value="15">15%</option>
                      <option value="20">20%</option>
                      <option value="25">25%</option>
                      <option value="30">30%</option>
                    </select>
                  </div>
                  
                  <div style={{ marginBottom: '12px' }}>
                    <strong>Coverage Amount:</strong> {(parseFloat(req.amount) * parseInt(coveragePercentage) / 100).toFixed(4)} ETH
                  </div>
                  
                  <button 
                    onClick={() => handleCoverLoan(req.id)}
                    className="donate-button"
                    disabled={covering}
                    style={{width: '100%'}}
                  >
                    {covering ? "Processing..." : `Cover ${coveragePercentage}%`}
                  </button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
      
      <button 
        onClick={loadPendingRequisitions} 
        className="wallet-button"
        style={{ marginTop: '16px' }}
      >
        Refresh List
      </button>
    </div>
  );
}