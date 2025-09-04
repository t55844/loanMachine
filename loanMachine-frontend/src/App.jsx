import { useState } from "react";
import WalletConnect from "./loan-interaction/WalletConnect";
import ContractInfo from "./loan-interaction/ContractInfo";
import Donate from "./loan-interaction/Donate";
import Borrow from "./loan-interaction/Borrow";
import Repay from "./loan-interaction/Repay";
import UserStats from "./loan-interaction/UserStats";

function App() {
  const [account, setAccount] = useState(null);

  return (
    <div className="p-4">
      <h1 className="text-2xl mb-4">Loan Machine DApp</h1>
      <WalletConnect setAccount={setAccount} />
      {account && (
        <>
          <ContractInfo account={account} />
          <UserStats account={account} />
          <Donate account={account} />
          <Borrow account={account} />
          <Repay account={account} />
        </>
      )}
    </div>
  );
}

export default App;
