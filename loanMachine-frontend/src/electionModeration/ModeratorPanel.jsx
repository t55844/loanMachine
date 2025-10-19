// components/ModeratorPanel.jsx
import React, { useState, useEffect } from 'react';
import { ethers } from 'ethers';

const ModeratorPanel = ({ reputationSystem, loanMachine, userAddress, member, provider }) => {
  const [isModerator, setIsModerator] = useState(false);
  const [loading, setLoading] = useState(false);
  const [transactionHistory, setTransactionHistory] = useState([]);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState('');
  const [filters, setFilters] = useState({
    startBlock: 0,
    endBlock: 'latest',
    transactionTypes: ['ALL']
  });
  const memberId = member?.memberId || 0;

  const availableTransactionTypes = [
    'ALL',
    'Donated',
    'Withdrawn',
    'Borrowed',
    'Repaid',
    'LoanRequisitionCreated',
    'LoanCovered',
    'LoanFunded',
    'LenderRepaid',
    'LoanCompleted'
  ];

  // Check if user is moderator
  useEffect(() => {
    const checkModeratorStatus = async () => {
      if (reputationSystem && memberId) {
        try {
          const moderatorStatus = await reputationSystem.isModerator(memberId);
          setIsModerator(moderatorStatus);
        } catch (error) {
          console.error('Error checking moderator status:', error);
        }
      }
    };

    checkModeratorStatus();
  }, [reputationSystem, memberId]);

  // Get events from blockchain
  const getBlockchainEvents = async () => {
    if (!loanMachine || !provider) return [];

    try {
      setLoading(true);
      setMessage('Querying blockchain events...');
      
      const events = [];
      const currentBlock = await provider.getBlockNumber();
      const fromBlock = filters.startBlock || Math.max(0, currentBlock - 10000); // Last 10k blocks if not specified

      // Define event filters
      const eventFilters = [
        { name: 'Donated', filter: loanMachine.filters.Donated() },
        { name: 'Withdrawn', filter: loanMachine.filters.Withdrawn() },
        { name: 'Borrowed', filter: loanMachine.filters.Borrowed() },
        { name: 'Repaid', filter: loanMachine.filters.Repaid() },
        { name: 'LoanRequisitionCreated', filter: loanMachine.filters.LoanRequisitionCreated() },
        { name: 'LoanCovered', filter: loanMachine.filters.LoanCovered() },
        { name: 'LoanFunded', filter: loanMachine.filters.LoanFunded() },
        { name: 'LenderRepaid', filter: loanMachine.filters.LenderRepaid() },
        { name: 'LoanCompleted', filter: loanMachine.filters.LoanCompleted() }
      ];

      // Determine which event types to query
      const selectedTypes = filters.transactionTypes.includes('ALL') 
        ? availableTransactionTypes.filter(t => t !== 'ALL') 
        : filters.transactionTypes;

      const eventFiltersToQuery = eventFilters.filter(ef => selectedTypes.includes(ef.name));

      // Query selected events
      for (const eventFilter of eventFiltersToQuery) {
        try {
          const eventLogs = await loanMachine.queryFilter(
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
              args: event.args,
              eventData: event
            });
          }
        } catch (error) {
          console.error(`Error querying ${eventFilter.name} events:`, error);
        }
      }

      // Process events into transaction format
      const transactions = processEventsIntoTransactions(events);
      setTransactionHistory(transactions);
      setMessage(`Found ${transactions.length} transactions from ${events.length} blockchain events`);
      return transactions;

    } catch (error) {
      console.error('Error getting blockchain events:', error);
      setMessage('Error querying blockchain events');
      return [];
    } finally {
      setLoading(false);
    }
  };

  // Process raw events into transaction format
  const processEventsIntoTransactions = (events) => {
    return events.map(event => {
      const baseTransaction = {
        type: event.type,
        transactionHash: event.transactionHash,
        blockNumber: event.blockNumber,
        timestamp: new Date(event.timestamp * 1000).toISOString(),
        blockTimestamp: event.timestamp
      };

      // Add event-specific data
      switch (event.type) {
        case 'Donated':
          return {
            ...baseTransaction,
            donor: event.args.donor,
            amount: event.args.amount.toString(),
            totalDonation: event.args.totalDonation?.toString() || '0'
          };

        case 'Withdrawn':
          return {
            ...baseTransaction,
            donor: event.args.donor,
            amount: event.args.amount.toString(),
            donations: event.args.donations?.toString() || '0'
          };

        case 'Borrowed':
          return {
            ...baseTransaction,
            borrower: event.args.borrower,
            amount: event.args.amount.toString(),
            totalBorrowing: event.args.totalBorrowing?.toString() || '0'
          };

        case 'Repaid':
          return {
            ...baseTransaction,
            borrower: event.args.borrower,
            amount: event.args.amount.toString(),
            remainingDebt: event.args.remainingDebt?.toString() || '0'
          };

        case 'LoanRequisitionCreated':
          return {
            ...baseTransaction,
            requisitionId: event.args.requisitionId.toString(),
            borrower: event.args.borrower,
            amount: event.args.amount.toString(),
            minimumCoverage: event.args.minimumCoverage?.toString() || '0'
          };

        case 'LoanCovered':
          return {
            ...baseTransaction,
            requisitionId: event.args.requisitionId.toString(),
            lender: event.args.lender,
            coverageAmount: event.args.coverageAmount.toString()
          };

        case 'LoanFunded':
          return {
            ...baseTransaction,
            requisitionId: event.args.requisitionId.toString()
          };

        case 'LenderRepaid':
          return {
            ...baseTransaction,
            requisitionId: event.args.requisitionId.toString(),
            lender: event.args.lender,
            amount: event.args.amount.toString()
          };

        case 'LoanCompleted':
          return {
            ...baseTransaction,
            requisitionId: event.args.requisitionId.toString()
          };

        default:
          return baseTransaction;
      }
    });
  };

  // Handle transaction type selection change
  const handleTypeChange = (e) => {
    const selectedOptions = Array.from(e.target.options)
      .filter(o => o.selected)
      .map(o => o.value);
    setFilters(prev => ({
      ...prev,
      transactionTypes: selectedOptions.length ? selectedOptions : ['ALL']
    }));
  };

  // Export functions (same as previous component)
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
      } else {
        content = convertToCSV(transactions);
        mimeType = 'text/csv';
        extension = 'csv';
      }

      const blob = new Blob([content], { type: mimeType });
      const url = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `financial_transactions_${new Date().toISOString().split('T')[0]}.${extension}`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(url);

      setMessage(`Successfully exported ${transactions.length} transactions as ${format.toUpperCase()}`);
      
    } catch (error) {
      console.error('Error exporting transactions:', error);
      setMessage('Error exporting transaction data');
    } finally {
      setExporting(false);
    }
  };

  const convertToCSV = (transactions) => {
    if (transactions.length === 0) return '';
    
    const headers = Object.keys(transactions[0]);
    const csvRows = [headers.join(',')];
    
    for (const transaction of transactions) {
      const values = headers.map(header => {
        const value = transaction[header] || '';
        return `"${String(value).replace(/"/g, '""')}"`;
      });
      csvRows.push(values.join(','));
    }
    
    return csvRows.join('\n');
  };

  if (!isModerator) {
    return (
      <div className="moderator-panel">
        <h2>Moderator Panel</h2>
        <div className="error-message">
          Access denied. You are not a moderator or your member ID is not verified.
        </div>
      </div>
    );
  }

  return (
    <div className="moderator-panel">
      <h2>Moderator Panel - Financial Transactions Export</h2>
      
      {/* Filters Section */}
      <div className="filters-section">
        <h3>Data Filters</h3>
        <div className="filter-controls">
          <div className="filter-group">
            <label>Start Block:</label>
            <input 
              type="number" 
              value={filters.startBlock}
              onChange={(e) => setFilters({...filters, startBlock: parseInt(e.target.value) || 0})}
              className="filter-input"
              placeholder="0"
            />
          </div>
          
          <div className="filter-group">
            <label>End Block:</label>
            <input 
              type="text" 
              value={filters.endBlock}
              onChange={(e) => setFilters({...filters, endBlock: e.target.value})}
              className="filter-input"
              placeholder="latest"
            />
          </div>

          <div className="filter-group">
            <label>Transaction Types:</label>
            <select 
              multiple
              value={filters.transactionTypes}
              onChange={handleTypeChange}
              className="filter-select"
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

      {/* Action Buttons */}
      <div className="action-buttons">
        <button 
          onClick={getBlockchainEvents}
          disabled={loading}
          className="moderator-button primary"
        >
          {loading ? 'Querying Blockchain...' : 'Load Blockchain Events'}
        </button>
        
        <button 
          onClick={() => exportToFile('csv')}
          disabled={exporting || transactionHistory.length === 0}
          className="moderator-button success"
        >
          {exporting ? 'Exporting...' : `Export CSV (${transactionHistory.length})`}
        </button>
        
        <button 
          onClick={() => exportToFile('json')}
          disabled={exporting || transactionHistory.length === 0}
          className="moderator-button success"
        >
          {exporting ? 'Exporting...' : `Export JSON (${transactionHistory.length})`}
        </button>
      </div>

      {message && (
        <div className={`message ${message.includes('Error') ? 'error' : 'success'}`}>
          {message}
        </div>
      )}

      {/* Transaction Summary */}
      {transactionHistory.length > 0 && (
        <div className="transaction-summary">
          <h3>Transaction Summary</h3>
          
          <div className="summary-stats">
            <div className="stat-card">
              <div className="stat-value">{transactionHistory.length}</div>
              <div className="stat-label">Total Transactions</div>
            </div>
            
            {availableTransactionTypes.filter(type => type !== 'ALL').map(type => {
              const count = transactionHistory.filter(t => t.type === type).length;
              return count > 0 ? (
                <div key={type} className="stat-card">
                  <div className="stat-value">{count}</div>
                  <div className="stat-label">{type}</div>
                </div>
              ) : null;
            })}
          </div>

          <div className="transaction-preview">
            <h4>Transaction Preview</h4>
            <div className="preview-table">
              <div className="preview-header">
                <span>Type</span>
                <span>Amount</span>
                <span>Address</span>
                <span>Block</span>
              </div>
              {transactionHistory.slice(0, 10).map((transaction, index) => (
                <div key={index} className="preview-row">
                  <span className="type-badge">{transaction.type}</span>
                  <span>{transaction.amount || transaction.coverageAmount || '0'}</span>
                  <span className="address">{transaction.borrower || transaction.lender || transaction.donor || 'N/A'}</span>
                  <span>#{transaction.blockNumber}</span>
                </div>
              ))}
            </div>
            {transactionHistory.length > 10 && (
              <div className="preview-note">
                Showing first 10 of {transactionHistory.length} transactions
              </div>
            )}
          </div>
        </div>
      )}

      <style jsx>{`
        .moderator-panel {
          background: var(--bg-secondary);
          padding: 24px;
          border-radius: 12px;
          border: 1px solid var(--border-color);
          margin-bottom: 20px;
        }

        .filters-section {
          margin-bottom: 20px;
          padding: 16px;
          background: var(--bg-tertiary);
          border-radius: 8px;
        }

        .filter-controls {
          display: flex;
          gap: 16px;
          flex-wrap: wrap;
        }

        .filter-group {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .filter-group label {
          font-weight: 600;
          color: var(--text-secondary);
          font-size: 14px;
        }

        .filter-input {
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 6px;
          background: var(--bg-primary);
          color: var(--text-primary);
          width: 150px;
        }

        .filter-select {
          padding: 8px 12px;
          border: 1px solid var(--border-color);
          border-radius: 6px;
          background: var(--bg-primary);
          color: var(--text-primary);
          height: 150px;
          width: 200px;
        }

        .action-buttons {
          display: flex;
          gap: 12px;
          margin-bottom: 20px;
          flex-wrap: wrap;
        }

        .moderator-button {
          padding: 12px 20px;
          border: none;
          border-radius: 6px;
          color: white;
          cursor: pointer;
          font-weight: 600;
          transition: all 0.2s ease;
          min-width: 200px;
        }

        .moderator-button.primary {
          background: var(--accent-blue);
        }

        .moderator-button.success {
          background: var(--accent-green);
        }

        .moderator-button:hover:not(:disabled) {
          opacity: 0.9;
          transform: translateY(-1px);
        }

        .moderator-button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
          transform: none;
        }

        .message {
          padding: 12px;
          border-radius: 6px;
          margin-bottom: 16px;
          text-align: center;
        }

        .message.success {
          background-color: rgba(0, 192, 135, 0.1);
          border: 1px solid var(--accent-green);
          color: var(--accent-green);
        }

        .message.error {
          background-color: rgba(242, 54, 69, 0.1);
          border: 1px solid var(--accent-red);
          color: var(--accent-red);
        }

        .transaction-summary {
          margin-top: 20px;
          padding: 16px;
          background: var(--bg-tertiary);
          border-radius: 8px;
        }

        .summary-stats {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
          gap: 12px;
          margin-bottom: 20px;
        }

        .stat-card {
          padding: 16px;
          background: var(--bg-primary);
          border-radius: 8px;
          text-align: center;
          border: 1px solid var(--border-color);
        }

        .stat-value {
          font-size: 24px;
          font-weight: 700;
          color: var(--accent-blue);
          margin-bottom: 4px;
        }

        .stat-label {
          font-size: 12px;
          color: var(--text-secondary);
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .transaction-preview h4 {
          margin-bottom: 12px;
          color: var(--text-primary);
        }

        .preview-table {
          border: 1px solid var(--border-color);
          border-radius: 8px;
          overflow: hidden;
        }

        .preview-header {
          display: grid;
          grid-template-columns: 1fr 1fr 2fr 1fr;
          gap: 8px;
          padding: 12px;
          background: var(--bg-primary);
          font-weight: 600;
          border-bottom: 1px solid var(--border-color);
        }

        .preview-row {
          display: grid;
          grid-template-columns: 1fr 1fr 2fr 1fr;
          gap: 8px;
          padding: 12px;
          border-bottom: 1px solid var(--border-color);
          font-size: 14px;
        }

        .preview-row:last-child {
          border-bottom: none;
        }

        .type-badge {
          background: var(--accent-blue);
          color: white;
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          text-align: center;
        }

        .address {
          font-family: monospace;
          font-size: 12px;
          word-break: break-all;
        }

        .preview-note {
          text-align: center;
          margin-top: 12px;
          color: var(--text-secondary);
          font-size: 14px;
        }

        @media (max-width: 768px) {
          .action-buttons {
            flex-direction: column;
          }

          .moderator-button {
            width: 100%;
          }

          .filter-controls {
            flex-direction: column;
          }

          .filter-input,
          .filter-select {
            width: 100%;
          }

          .preview-header,
          .preview-row {
            grid-template-columns: 1fr;
            gap: 4px;
          }
        }
      `}</style>
    </div>
  );
};

export default ModeratorPanel;