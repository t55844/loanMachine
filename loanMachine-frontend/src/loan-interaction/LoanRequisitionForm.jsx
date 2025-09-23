import { useState } from "react";
import { ethers } from "ethers";

export default function LoanRequisitionForm({ contract, account, onRequisitionCreated }) {
  const [amount, setAmount] = useState("");
  const [minimumCoverage, setMinimumCoverage] = useState("80");
  const [durationDays, setDurationDays] = useState("30");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!contract || !account) {
      setError("Please connect to Hardhat node first");
      return;
    }
    
    setLoading(true);
    setError("");

    try {
      const amountWei = ethers.utils.parseEther(amount);
      const tx = await contract.createLoanRequisition(
        amountWei,
        parseInt(minimumCoverage),
        parseInt(durationDays)
      );
      
      await tx.wait();
      
      // Reset form
      setAmount("");
      setMinimumCoverage("80");
      setDurationDays("30");
      
      if (onRequisitionCreated) {
        onRequisitionCreated();
      }
      
    } catch (err) {
      console.error("Error creating loan requisition:", err);
      setError(err.message || "Failed to create loan requisition");
    } finally {
      setLoading(false);
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
      <h2>Create Loan Requisition</h2>
      
      {error && <div style={{
        color: 'var(--accent-red)', 
        marginBottom: '16px',
        padding: '12px',
        backgroundColor: 'rgba(242, 54, 69, 0.1)',
        borderRadius: '6px',
        border: '1px solid var(--accent-red)'
      }}>{error}</div>}
      
      <form onSubmit={handleSubmit}>
        <div style={{ marginBottom: '16px', textAlign: 'left' }}>
          <label htmlFor="amount" style={{display: 'block', marginBottom: '8px'}}>Loan Amount (ETH)</label>
          <input
            id="amount"
            type="number"
            step="0.001"
            min="0.001"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="wallet-input"
            placeholder="Enter amount"
            required
            style={{width: '100%'}}
          />
        </div>
        
        <div style={{ marginBottom: '16px', textAlign: 'left' }}>
          <label htmlFor="minimumCoverage" style={{display: 'block', marginBottom: '8px'}}>Minimum Coverage (%)</label>
          <select
            id="minimumCoverage"
            value={minimumCoverage}
            onChange={(e) => setMinimumCoverage(e.target.value)}
            className="donate-select"
            style={{width: '100%'}}
            required
          >
            <option value="71">71%</option>
            <option value="75">75%</option>
            <option value="80">80%</option>
            <option value="85">85%</option>
            <option value="90">90%</option>
            <option value="95">95%</option>
            <option value="100">100%</option>
          </select>
        </div>
        
        <div style={{ marginBottom: '16px', textAlign: 'left' }}>
          <label htmlFor="durationDays" style={{display: 'block', marginBottom: '8px'}}>Loan Duration (Days)</label>
          <select
            id="durationDays"
            value={durationDays}
            onChange={(e) => setDurationDays(e.target.value)}
            className="donate-select"
            style={{width: '100%'}}
            required
          >
            <option value="15">15 days</option>
            <option value="30">30 days</option>
            <option value="60">60 days</option>
            <option value="90">90 days</option>
          </select>
        </div>
        
        <button 
          type="submit" 
          className="borrow-button"
          disabled={loading}
          style={{width: '100%'}}
        >
          {loading ? "Creating..." : "Create Loan Requisition"}
        </button>
      </form>
    </div>
  );
}