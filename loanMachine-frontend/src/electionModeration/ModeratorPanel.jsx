// components/ModeratorPanel.jsx
import React, { useState } from 'react';
import { ethers } from 'ethers';
import { useWeb3 } from '../Web3Context';

const ModeratorPanel = () => {
  const [isModerator, setIsModerator] = useState(false);
  const [loading, setLoading] = useState(false);
  const [transactionHistory, setTransactionHistory] = useState([]);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState('');
  const [debugInfo, setDebugInfo] = useState('');
  const [filters, setFilters] = useState({
    startBlock: 0,
    endBlock: 'latest',
    transactionTypes: ['ALL']
  });

  const { 
    account, 
    contract, 
    reputationContract,
    member,
    provider
  } = useWeb3();

  const memberId = member?.id || 0;

  const availableTransactionTypes = [
    'ALL',
    'Donated',
    'Withdrawn',
    'Borrowed',
    'Repaid',
    'LoanRequisitionCreatedCancelled',
    'LoanCovered',
    'LoanFunded',
    'LenderRepaid',
    'LoanCompleted'
  ];

  // Manual moderator check function
  const manualCheckModerator = async () => {
    if (!reputationContract) {
      setDebugInfo('Error: No reputation contract available');
      return;
    }
    
    if (!memberId || memberId === 0) {
      setDebugInfo('Error: No valid member ID. Please register first.');
      return;
    }

    try {
      setDebugInfo('Checking moderator status...');
      const moderatorStatus = await reputationContract.isModerator(memberId);
      
      setIsModerator(moderatorStatus);
      setDebugInfo(moderatorStatus ? '✓ Moderator access granted' : '✗ Not a moderator');
      
    } catch (error) {
      console.error('Error checking moderator status:', error);
      setDebugInfo(`Error: ${error.message}`);
      setIsModerator(false);
    }
  };

  // Get events from blockchain
  const getBlockchainEvents = async () => {
    if (!contract || !provider) {
      setMessage('Error: Contract or provider not available');
      return [];
    }

    try {
      setLoading(true);
      setMessage('Querying blockchain events...');
      
      const events = [];
      const currentBlock = await provider.getBlockNumber();
      const fromBlock = filters.startBlock || Math.max(0, currentBlock - 10000);

      const eventFilters = [
        { name: 'Donated', filter: contract.filters.Donated() },
        { name: 'Withdrawn', filter: contract.filters.Withdrawn() },
        { name: 'Borrowed', filter: contract.filters.Borrowed() },
        { name: 'Repaid', filter: contract.filters.Repaid() },
        { name: 'LoanRequisitionCreatedCancelled', filter: contract.filters.LoanRequisitionCreatedCancelled() },
        { name: 'LoanCovered', filter: contract.filters.LoanCovered() },
        { name: 'LoanFunded', filter: contract.filters.LoanFunded() },
        { name: 'LenderRepaid', filter: contract.filters.LenderRepaid() },
        { name: 'LoanCompleted', filter: contract.filters.LoanCompleted() }
      ];

      const selectedTypes = filters.transactionTypes.includes('ALL') 
        ? availableTransactionTypes.filter(t => t !== 'ALL') 
        : filters.transactionTypes;

      const eventFiltersToQuery = eventFilters.filter(ef => selectedTypes.includes(ef.name));

      for (const eventFilter of eventFiltersToQuery) {
        try {
          const eventLogs = await contract.queryFilter(
            eventFilter.filter, 
            fromBlock, 
            filters.endBlock === 'latest' ? currentBlock : filters.endBlock
          );
          
          for (const event of eventLogs) {
            const block = await event.getBlock();
            events.push({
              type: eventFilter.name,
              transactionHash: event.transactionHash,
              blockNumber: event.blockNumber,
              timestamp: block.timestamp,
              args: event.args
            });
          }
        } catch (error) {
          console.error(`Error querying ${eventFilter.name} events:`, error);
        }
      }

      const transactions = processEventsIntoTransactions(events);
      setTransactionHistory(transactions);
      setMessage(`Found ${transactions.length} transactions`);
      return transactions;

    } catch (error) {
      console.error('Error getting blockchain events:', error);
      setMessage('Error querying blockchain events');
      return [];
    } finally {
      setLoading(false);
    }
  };

  // Process events into transaction format
  const processEventsIntoTransactions = (events) => {
    return events.map(event => {
      const base = {
        type: event.type,
        transactionHash: event.transactionHash,
        blockNumber: event.blockNumber,
        timestamp: new Date(event.timestamp * 1000).toISOString()
      };

      switch (event.type) {
        case 'Donated':
          return { ...base, address: event.args.donor, amount: event.args.amount.toString() };
        case 'Withdrawn':
          return { ...base, address: event.args.donor, amount: event.args.amount.toString() };
        case 'Borrowed':
          return { ...base, address: event.args.borrower, amount: event.args.amount.toString() };
        case 'Repaid':
          return { ...base, address: event.args.borrower, amount: event.args.amount.toString() };
        case 'LoanRequisitionCreatedCancelled':
          return { ...base, address: event.args.borrower, amount: event.args.amount.toString() };
        case 'LoanCovered':
          return { ...base, address: event.args.lender, amount: event.args.coverageAmount.toString() };
        case 'LenderRepaid':
          return { ...base, address: event.args.lender, amount: event.args.amount.toString() };
        default:
          return base;
      }
    });
  };

  // Handle transaction type selection
  const handleTypeChange = (e) => {
    const selectedOptions = Array.from(e.target.selectedOptions, o => o.value);
    setFilters(prev => ({
      ...prev,
      transactionTypes: selectedOptions.length ? selectedOptions : ['ALL']
    }));
  };

  // Export functions
  const exportToFile = async (format = 'csv') => {
  setExporting(true);
  setMessage(`Exporting as ${format.toUpperCase()}...`);

  try {
    let transactions = transactionHistory;
    
    if (transactions.length === 0) {
      transactions = await getBlockchainEvents();
    }

    if (transactions.length === 0) {
      setMessage('No transactions found to export');
      return;
    }

    let content, mimeType, extension;
    
    if (format === 'json') {
      content = JSON.stringify(transactions, null, 2);
      mimeType = 'application/json';
      extension = 'json';
    } else if (format === 'tsv') {
      // Tab-separated for better Excel compatibility
      const headers = ['Type', 'Transaction Hash', 'Block Number', 'Timestamp', 'Address', 'Amount'];
      const rows = transactions.map(transaction => [
        transaction.type,
        transaction.transactionHash,
        transaction.blockNumber,
        transaction.timestamp,
        transaction.address || 'N/A',
        transaction.amount || '0'
      ]);
      content = [headers, ...rows].map(row => row.join('\t')).join('\n');
      mimeType = 'text/tab-separated-values';
      extension = 'tsv';
    } else {
      // Improved CSV with BOM for Excel
      const headers = ['Type', 'Transaction Hash', 'Block Number', 'Timestamp', 'Address', 'Amount'];
      const csvRows = transactions.map(transaction => [
        transaction.type,
        `"${transaction.transactionHash}"`,
        transaction.blockNumber,
        `"${transaction.timestamp}"`,
        transaction.address || 'N/A',
        transaction.amount || '0'
      ]);
      
      content = '\uFEFF' + [
        headers.join(','),
        ...csvRows.map(row => row.join(','))
      ].join('\n');
      
      mimeType = 'text/csv;charset=utf-8;';
      extension = 'csv';
    }

    const blob = new Blob([content], { type: mimeType });
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `transactions_${Date.now()}.${extension}`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);

    setMessage(`Exported ${transactions.length} transactions`);
    
  } catch (error) {
    console.error('Error exporting:', error);
    setMessage('Error exporting data');
  } finally {
    setExporting(false);
  }
};

  // If not moderator, show access panel
  if (!isModerator) {
    return (
      <div className="moderatorBlock" >
        <h2>Moderator Panel</h2>
        <div className="error-message">
          <p>Access denied. You need moderator privileges to access this panel.</p>
          <p>Click the button below to check your status.</p>
        </div>
        
        <div className="vinculate-section">
          <button 
            onClick={manualCheckModerator}
            className="wallet-button"
            style={{ marginBottom: '16px' }}
          >
            Check Moderator Status
          </button>
          
          {debugInfo && (
            <div className={`vinculation-info ${
              debugInfo.includes('✓') ? 'success' : 
              debugInfo.includes('Error') ? 'error' : 'info'
            }`}>
              {debugInfo}
            </div>
          )}
        </div>
      </div>
    );
  }

  // Main moderator panel
  return (
    <div className="card">
      <h2>Moderator Panel</h2>
      
      {/* Status Header */}
      <div className="balance-info success">
        ✓ Moderator Access Granted
        <button 
          onClick={manualCheckModerator}
          className="refresh-button"
          style={{ marginLeft: '16px', padding: '4px 12px', fontSize: '12px' }}
        >
          Re-check
        </button>
      </div>

      {/* Filters */}
      <div className="vinculate-section">
        <h3>Data Filters</h3>
        <div className="wallet-input-row">
          <div className="filter-group">
            <label>Start Block:</label>
            <input 
              type="number" 
              value={filters.startBlock}
              onChange={(e) => setFilters({...filters, startBlock: parseInt(e.target.value) || 0})}
              className="wallet-input"
              placeholder="0"
            />
          </div>
          
          <div className="filter-group">
            <label>End Block:</label>
            <input 
              type="text" 
              value={filters.endBlock}
              onChange={(e) => setFilters({...filters, endBlock: e.target.value})}
              className="wallet-input"
              placeholder="latest"
            />
          </div>

          <div className="filter-group">
            <label>Event Types:</label>
            <select 
              multiple
              value={filters.transactionTypes}
              onChange={handleTypeChange}
              className="wallet-input"
              style={{ height: '120px', backgroundImage: 'none' }}
            >
              {availableTransactionTypes.map(type => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="wallet-input-row">
        <button 
          onClick={getBlockchainEvents}
          disabled={loading}
          className="wallet-button"
        >
          {loading ? 'Loading...' : 'Load Events'}
        </button>
        
        <button 
          onClick={() => exportToFile('csv')}
          disabled={exporting || transactionHistory.length === 0}
          className="donate-button"
        >
          {exporting ? 'Exporting...' : `Export CSV (${transactionHistory.length})`}
        </button>
        
        <button 
          onClick={() => exportToFile('json')}
          disabled={exporting || transactionHistory.length === 0}
          className="donate-button"
        >
          {exporting ? 'Exporting...' : `Export JSON (${transactionHistory.length})`}
        </button>
      </div>

      {/* Messages */}
      {message && (
        <div className={`vinculation-info ${
          message.includes('Error') ? 'error' : 'success'
        }`}>
          {message}
        </div>
      )}

      {/* Transaction Summary */}
      {transactionHistory.length > 0 && (
        <div className="stats-box">
          <h3>Transaction Summary</h3>
          
          <div className="stats-grid">
            <div>
              <strong>Total Transactions</strong>
              <span>{transactionHistory.length}</span>
            </div>
            
            {availableTransactionTypes
              .filter(type => type !== 'ALL')
              .map(type => {
                const count = transactionHistory.filter(t => t.type === type).length;
                return count > 0 ? (
                  <div key={type}>
                    <strong>{type}</strong>
                    <span>{count}</span>
                  </div>
                ) : null;
              })}
          </div>

          <div className="transactions-box">
            <h4>Preview (First 10)</h4>
            
            {/* Simple list using existing contract-item style */}
            {transactionHistory.slice(0, 10).map((transaction, index) => (
              <div key={index} className="contract-item">
                <div className="contract-header">
                  <span className="status-badge">{transaction.type}</span>
                  <span>Block #{transaction.blockNumber}</span>
                </div>
                <div className="details-grid">
                  <div className="detail-item">
                    <strong>Amount:</strong>
                    <span>{transaction.amount || '0'}</span>
                  </div>
                  <div className="detail-item">
                    <strong>Address:</strong>
                    <span className="user-address" style={{ fontSize: '12px' }}>
                      {transaction.address || 'N/A'}
                    </span>
                  </div>
                  <div className="detail-item">
                    <strong>Transaction:</strong>
                    <span className="user-address" style={{ fontSize: '12px' }}>
                      {transaction.transactionHash?.slice(0, 10)}...
                    </span>
                  </div>
                  <div className="detail-item">
                    <strong>Time:</strong>
                    <span>{new Date(transaction.timestamp).toLocaleString()}</span>
                  </div>
                </div>
              </div>
            ))}
            
            {transactionHistory.length > 10 && (
              <div className="vinculation-info">
                Showing 10 of {transactionHistory.length} transactions
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ModeratorPanel;