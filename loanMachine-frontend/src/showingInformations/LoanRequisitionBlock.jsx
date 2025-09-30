import { useState } from "react";
import { useWeb3 } from "../Web3Context";
import LoanRequisitionForm from "../loan-interaction/LoanRequisitionForm";
import LoanRequisitionMonitor from "./LoanRequisitionMonitor";
import PendingRequisitionsList from "./PendingRequisitionsList";
import UserLoanContracts from "./UserLoanContracts"; // Add this import

function LoanRequisitionBlock() {
  const [activeTab, setActiveTab] = useState("create");
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const { account, contract } = useWeb3();

  const handleRequisitionCreated = () => {
    setRefreshTrigger(prev => prev + 1);
  };

  const handleCoverLoan = () => {
    setRefreshTrigger(prev => prev + 1);
  };

  const handleLoanUpdate = () => {
    setRefreshTrigger(prev => prev + 1);
  };

  return (
    <div className="requsitionBlock">
      <h1>Loan Requisition System</h1>
      
      <div style={{ display: 'flex', gap: '16px', marginBottom: '24px', flexWrap: 'wrap' }}>
        <button 
          onClick={() => setActiveTab("create")}
          style={{ 
            padding: '12px 20px',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontWeight: '600',
            fontSize: '14px',
            backgroundColor: activeTab === "create" ? "var(--accent-blue)" : "var(--bg-tertiary)",
            color: activeTab === "create" ? "white" : "var(--text-primary)"
          }}
        >
          Create Requisition
        </button>
        <button 
          onClick={() => setActiveTab("monitor")}
          style={{ 
            padding: '12px 20px',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontWeight: '600',
            fontSize: '14px',
            backgroundColor: activeTab === "monitor" ? "var(--accent-blue)" : "var(--bg-tertiary)",
            color: activeTab === "monitor" ? "white" : "var(--text-primary)"
          }}
        >
          My Requisitions
        </button>
        <button 
          onClick={() => setActiveTab("pending")}
          style={{ 
            padding: '12px 20px',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontWeight: '600',
            fontSize: '14px',
            backgroundColor: activeTab === "pending" ? "var(--accent-blue)" : "var(--bg-tertiary)",
            color: activeTab === "pending" ? "white" : "var(--text-primary)"
          }}
        >
          Cover Loans
        </button>
        <button 
          onClick={() => setActiveTab("contracts")}
          style={{ 
            padding: '12px 20px',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontWeight: '600',
            fontSize: '14px',
            backgroundColor: activeTab === "contracts" ? "var(--accent-blue)" : "var(--bg-tertiary)",
            color: activeTab === "contracts" ? "white" : "var(--text-primary)"
          }}
        >
          My Contracts
        </button>
      </div>
      
      {activeTab === "create" && (
        <LoanRequisitionForm 
          contract={contract}
          account={account}
          onRequisitionCreated={handleRequisitionCreated}
        />
      )}
      
      {activeTab === "monitor" && (
        <LoanRequisitionMonitor 
          contract={contract}
          account={account}
          key={refreshTrigger}
        />
      )}
      
      {activeTab === "pending" && (
        <PendingRequisitionsList 
          contract={contract}
          account={account}
          key={refreshTrigger}
          onCoverLoan={handleCoverLoan}
        />
      )}

      {activeTab === "contracts" && (
        <UserLoanContracts 
          contract={contract}
          account={account}
          key={refreshTrigger}
          onLoanUpdate={handleLoanUpdate}
        />
      )}
    </div>
  );
}

export default LoanRequisitionBlock;