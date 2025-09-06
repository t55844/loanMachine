import { useState } from "react";
import WalletConnect from "./loan-interaction/WalletConnect";
import ContractInfo from "./loan-interaction/ContractInfo";
import UserStats from "./loan-interaction/UserStats";
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
        <ContractInfo />
        <WalletConnect setAccount={setAccount} />
        {account && (
          <div className="section">
            <UserStats account={account} />
            <Donate account={account} />
            <Borrow account={account} />
            <Repay account={account} />
          </div>
        )}
      </div>
    </div>
  );
}
