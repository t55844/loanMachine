import { useState } from "react";
import WalletConnect from "./loan-interaction/WalletConnect";
import ContractOverview from "./showingInformations/ContractOverview";
import WalletStats from "./showingInformations/WalletStats";
import Donate from "./loan-interaction/Donate";
import Borrow from "./loan-interaction/Borrow";
import Repay from "./loan-interaction/Repay";
import WalletDistribution from "./showingInformations/WalletDistribution"
import "./App.css";

export default function App() {
  const [account, setAccount] = useState(null);

  return (
    <div className="app-container">
      <div className="card">
        <h1>Loan Machine DApp</h1>
        <WalletDistribution />
       {/* <WalletConnect setAccount={setAccount} />*/}
        {account && (
          <div className="section">
          </div>
        )}
        <WalletStats account={account} />
        <Donate account={account} />
        <Borrow account={account} />
        <Repay account={account} />
        
        <ContractOverview />
      </div>
    </div>
  );
}
