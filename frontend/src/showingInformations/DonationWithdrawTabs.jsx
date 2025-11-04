import { useState } from "react";
import Donate from "../loan-interaction/Donate";
import Withdraw from "../loan-interaction/Withdraw";

function DonationWithdrawTabs() {
  const [activeTab, setActiveTab] = useState('donate');

  return (
    <div className="interaction-tabs">
      <div className="tab-header">
        <button 
          className={`tab-button ${activeTab === 'donate' ? 'active' : ''}`}
          onClick={() => setActiveTab('donate')}
        >
          Doar
        </button>
        <button 
          className={`tab-button ${activeTab === 'withdraw' ? 'active' : ''}`}
          onClick={() => setActiveTab('withdraw')}
        >
          Retirar
        </button>
      </div>
      
      <div className="tab-content">
        {activeTab === 'donate' && <Donate />}
        {activeTab === 'withdraw' && <Withdraw />}
      </div>
    </div>
  );
}

export default DonationWithdrawTabs;