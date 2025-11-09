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
import { useToast } from "./handlers/useToast"; // NEW: Import useToast

export default function App() {
  const savedWallet = localStorage.getItem('connectedWalletAddress');
  const [enteredSite, setEnteredSite] = useState(!!savedWallet); // true if wallet exists

  const { 
    account: web3Account, 
    contract, 
    reputationContract,
    loading, 
    member,
    provider,
    disconnect,
    loanInterface // NEW: Get loanInterface for error decoding
  } = useWeb3();

  const { showTransactionModal, ModalWrapper } = useGasCostModal();

  const { showError } = useToast(provider, contract); // NEW: Use toast with provider/contract

  const runDebtCheck = async () => {
    if (!contract) return;
    showTransactionModal({
      method: 'performPeriodicDebtCheck',
      params: [10]
    }, {
      action: 'Executar Verificação Periódica de Dívida',
      description: 'Verificar empréstimos em atraso em lotes'
    });
  };

  const handleConfirmDebtCheck = async () => {
    try {
      const tx = await contract.performPeriodicDebtCheck(10);
      await tx.wait();
      console.log("Verificação de dívida concluída com sucesso");
    } catch (error) {
      await showError(error); // UPDATED: Use showError for proper handling
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
          <h1>Sua DApp</h1>
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
            🛡️ Executar Verificação de Dívida
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
            ❌ Desconectar
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
            Você precisa vincular sua carteira a um membro antes de poder votar nas eleições.
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