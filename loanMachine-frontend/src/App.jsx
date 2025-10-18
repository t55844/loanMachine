// App.js
import { useState } from "react";
import { useWeb3 } from "./Web3Context";
import ContractOverview from "./showingInformations/ContractOverview";
import WalletDistribution from "./showingInformations/WalletDistribution"
import LoanRequisitionBlock from "./showingInformations/LoanRequisitionBlock";
import WalletStats from "./showingInformations/WalletStats";
import VinculateMember from "./loan-interaction/VinculateMember";
import SideMenu from "./siteStrcture/SideMenu";
import "./App.css";
import WalletVerificationBanner from "./siteStrcture/WalletVerificationBanner";
import Toast from "./handlers/Toast";
import ElectionManagement from "./electionModeration/ElectionManagement";

export default function App() {
  const [account, setAccount] = useState(null);
  const { account: web3Account, contract, loading, memberId } = useWeb3();

  // Use the web3 account if available, otherwise use the local state
  const currentAccount = web3Account || account;

  return (
    <div className="app-container" style={{width: '100%'}}>
    <WalletVerificationBanner />
    <Toast />
      {/* Side Menu */}
      <SideMenu position="left">
        <WalletStats />
        <VinculateMember />
      </SideMenu>

      {/* Main Content */}
      <div className="card">
        <h1>Loan Machine DApp</h1>
        <WalletDistribution />
        <ContractOverview />
        <LoanRequisitionBlock />

          
      {/* Election Management with Tabs */}
        <ElectionManagement 
          contract={contract}
          currentAccount={currentAccount}
          memberId={memberId}
        />
      </div>
    </div>
  );
}