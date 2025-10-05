import { useState } from "react";
import { ethers } from "ethers";
import { useToast } from "../handlers/useToast";
import Toast from "../handlers/Toast";

export default function LoanRequisitionForm({ contract, account, onRequisitionCreated }) {
  const [amount, setAmount] = useState("");
  const [minimumCoverage, setMinimumCoverage] = useState("80");
  const [parcelsQuantity, setParcelsQuantity] = useState("1");
  const [durationDays, setDurationDays] = useState("30");
  const [loading, setLoading] = useState(false);
  
  const { toast, showToast, hideToast, handleContractError } = useToast();

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    if (!contract || !account) {
      showToast("Please connect to Hardhat node first");
      return;
    }
    
    setLoading(true);

    try {
      // Convert to USDT (6 decimals) instead of ETH (18 decimals)
      const amountWei = ethers.utils.parseUnits(amount, 6);
      const tx = await contract.createLoanRequisition(
        amountWei,
        parseInt(minimumCoverage),
        parseInt(durationDays),
        parseInt(parcelsQuantity)
      );
      
      await tx.wait();
      
      // Reset form
      setAmount("");
      setMinimumCoverage("80");
      setDurationDays("30");
      
      showToast("Loan requisition created successfully!", "success");
      
      if (onRequisitionCreated) {
        onRequisitionCreated();
      }
      
    } catch (err) {
      handleContractError(err, "createLoanRequisition");
    } finally {
      setLoading(false);
    }
  };

  return (
    <>
      <Toast toast={toast} onClose={hideToast} />
      
      <div style={{
        background: 'var(--bg-tertiary)',
        padding: '24px',
        borderRadius: '12px',
        border: '1px solid var(--border-color)',
        marginTop: '16px'
      }}>
        <h2>Create Loan Requisition</h2>
        
        <form onSubmit={handleSubmit}>
          <div style={{ marginBottom: '16px', textAlign: 'center' }}>
            <label htmlFor="amount" style={{display: 'block', marginBottom: '8px'}}>Loan Amount (USDT)</label>
            <input
              id="amount"
              type="number"
              step="0.01"
              min="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              className="wallet-input"
              placeholder="Enter amount"
              required
              style={{width: '100%'}}
            />
          </div>

          <div style={{ display:'flex', justifyContent: 'space-around' }}>
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
            <label htmlFor="parcelsQuantity" style={{display: 'block', marginBottom: '8px'}}>Parcels Count</label>
            <select
              id="parcelsQuantity"
              value={parcelsQuantity}
              onChange={(e) => setParcelsQuantity(e.target.value)}
              className="donate-select"
              style={{width: '100%'}}
              required
            >
              <option value="1">1</option>
              <option value="2">2</option>
              <option value="3">3</option>
              <option value="4">4</option>
              <option value="5">5</option>
              <option value="6">6</option>
              <option value="7">7</option>
              <option value="8">8</option>
              <option value="9">9</option>
              <option value="10">10</option>
              <option value="11">11</option>
              <option value="12">12</option>
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
    </>
  );
}