import { useState } from "react";
import { useWeb3 } from "./Web3Context";
import { useGasCostModal } from "./handlers/useGasCostModal";
import ContractOverview from "./showingInformations/ContractOverview";
import WalletDistribution from "./showingInformations/WalletDistribution";
import LoanRequisitionBlock from "./showingInformations/LoanRequisitionBlock";
import WalletStats from "./showingInformations/WalletStats";
import VinculateMember from "./loan-interaction/VinculateMember";
import SideMenu from "./siteStrcture/SideMenu";
import "./App.css";
import WalletVerificationBanner from "./siteStrcture/WalletVerificationBanner";
import Toast from "./handlers/Toast";
import ElectionManagement from "./electionModeration/ElectionManagement";
import ModeratorPanel from "./electionModeration/ModeratorPanel";
import WalletConnection from "./loan-interaction/WalletConnection";

export default function App() {
  // ✅ Load initial state from localStorage
  const savedWallet = localStorage.getItem('connectedWalletAddress');
  const [enteredSite, setEnteredSite] = useState(!!savedWallet); // true if wallet exists

  const { 
    account: web3Account, 
    contract, 
    reputationContract,
    loading, 
    member,
    provider,
    disconnect
  } = useWeb3();

  const { showTransactionModal, ModalWrapper } = useGasCostModal();

  const runDebtCheck = async () => {
    if (!contract) return;
    showTransactionModal({
      method: 'performPeriodicDebtCheck',
      params: [10]
    }, {
      action: 'Run Periodic Debt Check',
      description: 'Check for overdue loans in batches'
    });
  };

  const handleConfirmDebtCheck = async () => {
    try {
      const tx = await contract.performPeriodicDebtCheck(10);
      await tx.wait();
      console.log("Debt check completed successfully");
    } catch (error) {
      console.error('Debt check failed:', error);
      throw error;
    }
  };

  const handleDisconnect = () => {
    disconnect();
    setEnteredSite(false);
    localStorage.removeItem('connectedWalletAddress'); // clear persisted wallet
    localStorage.removeItem('connectedWalletType'); // clear type
  };

  if (!enteredSite) {
    return (
      <div className="app-container">
        <div className="card">
          <h1>Your DApp</h1>
          <WalletConnection onContinue={() => setEnteredSite(true)} />
        </div>
      </div>
    );
  }

  return (
    <div className="app-container" style={{ width: '100%' }}>
      <WalletVerificationBanner />
      <Toast />
      <ModalWrapper onConfirm={handleConfirmDebtCheck} />
      <SideMenu position="left">
        <WalletStats />
        <VinculateMember />
        <div style={{ marginTop: '20px', padding: '10px' }}>
          <button 
            onClick={runDebtCheck}
            style={{
              padding: '10px 15px',
              backgroundColor: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              width: '100%'
            }}
          >
            🛡️ Run Debt Check
          </button>
        </div>
        <div style={{ marginTop: '30px', padding: '10px' }}>
          <button onClick={handleDisconnect} style={{
              padding: '10px 15px',
              backgroundColor: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              width: '100%'
            }}>
            ❌ Disconnect
          </button>
        </div>
      </SideMenu>

      <div className="card">
        <h1>Loan Machine DApp</h1>
        <WalletDistribution />
        <ContractOverview />
        <LoanRequisitionBlock />
        {member && !member.hasVinculation && (
          <div className="error-message" style={{ marginBottom: '20px' }}>
            You need to vinculate your wallet to a member before you can vote in elections.
          </div>
        )}
        <ElectionManagement 
          contract={reputationContract}
          currentAccount={web3Account}
          member={member}
        />
        <ModeratorPanel />
      </div>
    </div>
  );
}
