import { useState } from "react";
import { useWeb3 } from "./Web3Context";
import WalletConnect from "./loan-interaction/WalletConnect";
import ContractOverview from "./showingInformations/ContractOverview";
import WalletStats from "./showingInformations/WalletStats";
import Donate from "./loan-interaction/Donate";
import Borrow from "./loan-interaction/Borrow";
import Repay from "./loan-interaction/Repay";
import WalletDistribution from "./showingInformations/WalletDistribution"
import LoanRequisitionBlock from "./showingInformations/LoanRequisitionBlock";
import "./App.css";

export default function App() {
  const [account, setAccount] = useState('0x14dC79964da2C08b23698B3D3cc7Ca32193d9955');
  const { account: web3Account, contract, loading } = useWeb3();

  // Use the web3 account if available, otherwise use the local state
  const currentAccount = web3Account || account;

  return (
    <div className="app-container">
      <div className="card">
        <h1>Loan Machine DApp</h1>
        <WalletDistribution />
        <WalletConnect setAccount={setAccount} />
        {currentAccount && (
          <div className="section">
            {/* Your existing section content */}
          </div>
        )}
        <WalletStats account={currentAccount} />
        <Donate account={currentAccount} contract={contract} />
        <Borrow account={currentAccount} contract={contract} />
        <Repay account={currentAccount} contract={contract} />
        
        <ContractOverview />
        <LoanRequisitionBlock />
      </div>
    </div>
  );
}