// DonationWithdrawTabs.jsx
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
          Donate
        </button>
        <button 
          className={`tab-button ${activeTab === 'withdraw' ? 'active' : ''}`}
          onClick={() => setActiveTab('withdraw')}
        >
          Withdraw
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