import { useState } from "react";
import { useWeb3 } from "../Web3Context"; 
import LoanRequisitionForm from "../loan-interaction/LoanRequisitionForm";
import LoanRequisitionMonitor from "./LoanRequisitionMonitor";
import PendingRequisitionsList from "./PendingRequisitionsList";
import UserLoanContracts from "./UserLoanContracts";

function LoanRequisitionBlock() {
  const [activeTab, setActiveTab] = useState("create");
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  // 👇 UPDATE: Include loading and error from the hook
  const { account, contract, member, loading, error,provider } = useWeb3();

  const handleRequisitionCreated = () => {
    setRefreshTrigger(prev => prev + 1);
  };

  const handleCoverLoan = () => {
    setRefreshTrigger(prev => prev + 1);
  };

  const handleLoanUpdate = () => {
    setRefreshTrigger(prev => prev + 1);
  };

  // 🛑 CORE FIX: Render loading/error states before component logic
  if (loading) {
    return <div className="requsitionBlock"><p>Conectando ao Web3 e carregando dados do membro...</p></div>;
  }

  if (error) {
    return <div className="requsitionBlock"><p style={{color: 'var(--accent-red)'}}>Erro de Conexão: {error}</p></div>;
  }
  
  // Optional but highly recommended: Handle the case where the wallet is connected 
  // but the member profile does not exist (member is not null, but id is null)
  if (!member || !member.walletAddress) {
      // This state should only be reachable if 'account' is also null, 
      // or if there's a serious data fetch error not captured by 'loading/error'.
      return <div className="requsitionBlock"><p>Por favor, conecte sua carteira.</p></div>;
  }


  return (
    <div className="requsitionBlock">
      <h1>Sistema de Requisição de Empréstimos</h1>
      
      {/* ... Buttons and Tab Navigation (no changes needed here) ... */}
      <div style={{ display: 'flex', gap: '16px', marginBottom: '24px', flexWrap: 'wrap' }}>
        <button 
          onClick={() => setActiveTab("create")}
          pw-test-id="create-requisition-button-tab"
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
          Criar Requisição
        </button>
        <button 
          onClick={() => setActiveTab("monitor")}
          pw-test-id="monitor-requisitions-button-tab"
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
          Minhas Requisições
        </button>
        <button 
          onClick={() => setActiveTab("pending")}
          pw-test-id="pending-requisitions-button-tab"
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
          Cobrir Empréstimos
        </button>
        <button 
          onClick={() => setActiveTab("contracts")}
          pw-test-id="user-contracts-button-tab"
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
          Meus Contratos
        </button>
      </div>
      
      {activeTab === "create" && (
        <LoanRequisitionForm 
          contract={contract}
          account={account}
          onRequisitionCreated={handleRequisitionCreated}
          member={member}
          provider={provider}
        />
      )}
      
      {activeTab === "monitor" && (
        <LoanRequisitionMonitor 
          contract={contract}
          account={account}
          // If LoanRequisitionMonitor needs member ID, access it safely
          memberId={member?.memberId || member?.id} 
          key={refreshTrigger}
        />
      )}
      
      {activeTab === "pending" && (
        <PendingRequisitionsList 
          contract={contract}
          account={account}
          key={refreshTrigger}
          onCoverLoan={handleCoverLoan}
          member={member}
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