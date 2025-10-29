import { useState, useEffect } from "react";
import { fetchBorrowerRequisitions } from "../graphql-frontend-query";
import { useToast } from "../handlers/useToast";

export default function LoanRequisitionMonitor({ contract, account, memberId }) {
  const [requisitions, setRequisitions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [cancellingId, setCancellingId] = useState(null);
  
  const { showSuccess, showError, handleContractError } = useToast();

  useEffect(() => {
    loadRequisitions();
  }, [account]);

  const loadRequisitions = async () => {
    if (!account) return;
    
    setLoading(true);
    setError("");
    
    try {
      const requisitionDetails = await fetchBorrowerRequisitions(account);
      setRequisitions(requisitionDetails);
    } catch (err) {
      console.error("Error loading requisitions:", err);
      setError("Failed to load loan requisitions");
    } finally {
      setLoading(false);
    }
  };

  const handleCancelRequisition = async (requisitionId) => {
    if (!contract || !memberId) {
      showError("Contract or member ID not available");
      return;
    }

    setCancellingId(requisitionId);
    
    try {
      const tx = await contract.cancelLoanRequisition(requisitionId, memberId);
      await tx.wait();
      
      // Show success message
      showSuccess(`Requisition #${requisitionId} cancelled successfully!`);
      
      // Refresh the list
      await loadRequisitions();
      
    } catch (err) {
      handleContractError(err, "cancelling loan requisition");
    } finally {
      setCancellingId(null);
    }
  };

  // ... rest of your component functions remain the same ...
  const getStatusText = (status) => {
    switch (status) {
      case 0: return "Pending";
      case 1: return "Partially Covered";
      case 2: return "Fully Covered";
      case 3: return "Active";
      case 4: return "Repaid";
      case 5: return "Cancelled";
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

  const isCancellable = (status) => {
    return status === 0 || status === 1;
  };

  const formatUSDT = (amount) => {
    return parseFloat(amount).toFixed(2);
  };

  // JSX remains exactly the same
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

              {isCancellable(req.status) && (
                <div style={{ textAlign: 'right' }}>
                  <button 
                    onClick={() => handleCancelRequisition(req.id)}
                    disabled={cancellingId === req.id}
                    style={{
                      backgroundColor: 'var(--accent-red)',
                      color: 'white',
                      border: 'none',
                      padding: '8px 16px',
                      borderRadius: '4px',
                      cursor: cancellingId === req.id ? 'not-allowed' : 'pointer',
                      opacity: cancellingId === req.id ? 0.6 : 1
                    }}
                  >
                    {cancellingId === req.id ? 'Cancelling...' : 'Cancel Requisition'}
                  </button>
                </div>
              )}
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