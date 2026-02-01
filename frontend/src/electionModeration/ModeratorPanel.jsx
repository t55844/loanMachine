// Updated ModeratorPanel.jsx - Removed getHashesFromGraphQL, import fetchEventHashes from query file
import React, { useState } from 'react';
import { useWeb3 } from '../Web3Context';
import { fetchEventHashes } from '../graphql-frontend-query'; // Adjust path to your query file

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
    transactionTypes: ['ALL'],
    source: 'graphql_hashes' // Default para GraphQL hashes + chain details (híbrido, eficiente)
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
      setDebugInfo('Erro: Nenhum contrato de reputação disponível');
      return;
    }
    
    if (!memberId || memberId === 0) {
      setDebugInfo('Erro: Nenhum ID de membro válido. Por favor, registre-se primeiro.');
      return;
    }

    try {
      setDebugInfo('Verificando status de moderador...');
      const moderatorStatus = await reputationContract.isModerator(memberId);
      
      setIsModerator(moderatorStatus);
      setDebugInfo(moderatorStatus ? '✓ Acesso de moderador concedido' : '✗ Não é um moderador');
      
    } catch (error) {
      //console.error('Erro ao verificar status de moderador:', error);
      setDebugInfo(`Erro: ${error.message}`);
      setIsModerator(false);
    }
  };

  // Fetch details from chain using hashes
  const fetchDetailsFromChain = async (hashMap) => {
    if (!contract || !provider) {
      setMessage('Erro: Contrato ou provedor não disponível');
      return [];
    }

    const events = [];
    const uniqueHashes = new Set(Object.values(hashMap).flat().map(item => item.transactionHash));

    for (const hash of uniqueHashes) {
      try {
        const receipt = await provider.getTransactionReceipt(hash);
        if (!receipt) continue;

        const block = await provider.getBlock(receipt.blockNumber);
        const timestamp = block.timestamp;

        for (const log of receipt.logs) {
          if (log.address.toLowerCase() !== contract.address.toLowerCase()) continue;

          let parsedLog;
          try {
            parsedLog = contract.interface.parseLog(log);
          } catch (e) {
            continue; // Log não é de um evento conhecido
          }

          if (!parsedLog) continue;

          const type = parsedLog.name;
          if (!Object.keys(hashMap).includes(type)) continue; // Não é um tipo selecionado

          events.push({
            type,
            transactionHash: hash,
            blockNumber: receipt.blockNumber,
            timestamp,
            args: parsedLog.args
          });
        }
      } catch (error) {
        //console.error(`Erro ao obter receipt para hash ${hash}:`, error);
      }
    }

    return events;
  };

  // Get events from blockchain with pagination
  const getFromBlockchain = async (fromBlock, toBlock, selectedTypes) => {
    if (!contract || !provider) {
      setMessage('Erro: Contrato ou provedor não disponível');
      return [];
    }

    try {
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

      const eventFiltersToQuery = eventFilters.filter(ef => selectedTypes.includes(ef.name));

      const batchSize = 10;
      const events = [];

      for (let start = fromBlock; start <= toBlock; start += batchSize) {
        const end = Math.min(start + batchSize - 1, toBlock);
        setMessage(`Consultando blocos ${start} a ${end}... (${Math.round((start / toBlock) * 100)}%)`);

        for (const eventFilter of eventFiltersToQuery) {
          try {
            const eventLogs = await contract.queryFilter(eventFilter.filter, start, end);
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
            //console.error(`Erro consultando eventos ${eventFilter.name} em batch ${start}-${end}:`, error);
          }
        }
      }

      return events;
    } catch (error) {
      //console.error('Erro ao obter eventos da blockchain:', error);
      setMessage('Erro ao consultar eventos da blockchain');
      return [];
    }
  };

  // Load events based on source
  const loadEvents = async () => {
    setLoading(true);
    setMessage('Consultando eventos...');

    let currentBlock = 0;
    let startTime = 0;
    let endTime = Math.floor(Date.now() / 1000);

    if (provider) {
      currentBlock = await provider.getBlockNumber();
      const fromBlockNum = filters.startBlock || Math.max(0, currentBlock - 10000);
      const toBlockNum = filters.endBlock === 'latest' ? currentBlock : Number(filters.endBlock);

      const startBlockInfo = await provider.getBlock(fromBlockNum);
      startTime = startBlockInfo ? startBlockInfo.timestamp : 0;

      const endBlockInfo = await provider.getBlock(toBlockNum);
      endTime = endBlockInfo ? endBlockInfo.timestamp : endTime;
    }

    const selectedTypes = filters.transactionTypes.includes('ALL') 
      ? availableTransactionTypes.filter(t => t !== 'ALL') 
      : filters.transactionTypes;

    let events = [];
    if (filters.source === 'blockchain') {
      const fromBlockNum = filters.startBlock || Math.max(0, currentBlock - 10000);
      const toBlockNum = filters.endBlock === 'latest' ? currentBlock : Number(filters.endBlock);
      events = await getFromBlockchain(fromBlockNum, toBlockNum, selectedTypes);
    } else {
      // Híbrido: hashes do GraphQL, details da chain
      const hashMap = await fetchEventHashes(selectedTypes, startTime, endTime);
      events = await fetchDetailsFromChain(hashMap);
    }

    const transactions = processEventsIntoTransactions(events);
    setTransactionHistory(transactions);
    setMessage(`Encontradas ${transactions.length} transações`);
    setLoading(false);
    return transactions;
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
          return { ...base, address: event.args.donor, amount: event.args.amount?.toString() };
        case 'Withdrawn':
          return { ...base, address: event.args.donor, amount: event.args.amount?.toString() };
        case 'Borrowed':
          return { ...base, address: event.args.borrower, amount: event.args.amount?.toString() };
        case 'Repaid':
          return { ...base, address: event.args.borrower, amount: event.args.amount?.toString() };
        case 'LoanRequisitionCreatedCancelled':
          return { ...base, address: event.args.borrower, amount: event.args.amount?.toString() };
        case 'LoanCovered':
          return { ...base, address: event.args.lender, amount: event.args.coverageAmount?.toString() };
        case 'LoanFunded':
          return { ...base, requisitionId: event.args.requisitionId?.toString() };
        case 'LenderRepaid':
          return { ...base, address: event.args.lender, amount: event.args.amount?.toString() };
        case 'LoanCompleted':
          return { ...base, requisitionId: event.args.requisitionId?.toString() };
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
    setMessage(`Exportando como ${format.toUpperCase()}...`);

    try {
      let transactions = transactionHistory;
      
      if (transactions.length === 0) {
        transactions = await loadEvents();
      }

      if (transactions.length === 0) {
        setMessage('Nenhuma transação encontrada para exportar');
        return;
      }

      let content, mimeType, extension;
      
      if (format === 'json') {
        content = JSON.stringify(transactions, null, 2);
        mimeType = 'application/json';
        extension = 'json';
      } else if (format === 'tsv') {
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

      setMessage(`Exportadas ${transactions.length} transações`);
      
    } catch (error) {
      //console.error('Erro ao exportar:', error);
      setMessage('Erro ao exportar dados');
    } finally {
      setExporting(false);
    }
  };

  // If not moderator, show access panel
  if (!isModerator) {
    return (
      <div className="moderatorBlock" >
        <h2>Painel do Moderador</h2>
        <div className="error-message">
          <p>Acesso negado. Você precisa de privilégios de moderador para acessar este painel.</p>
          <p>Clique no botão abaixo para verificar seu status.</p>
        </div>
        
        <div className="vinculate-section">
          <button 
            onClick={manualCheckModerator}
            className="wallet-button"
            style={{ marginBottom: '16px' }}
          >
            Verificar Status de Moderador
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
      <h2>Painel do Moderador</h2>
      
      {/* Status Header */}
      <div className="balance-info success">
        ✓ Acesso de Moderador Concedido
        <button 
          onClick={manualCheckModerator}
          className="refresh-button"
          style={{ marginLeft: '16px', padding: '4px 12px', fontSize: '12px' }}
        >
          Verificar Novamente
        </button>
      </div>

      {/* Filters */}
      <div className="vinculate-section">
        <h3>Filtros de Dados</h3>
        <div className="wallet-input-row">
          <div className="filter-group">
            <label>Bloco Inicial:</label>
            <input 
              type="number" 
              value={filters.startBlock}
              onChange={(e) => setFilters({...filters, startBlock: parseInt(e.target.value) || 0})}
              className="wallet-input"
              placeholder="0"
            />
          </div>
          
          <div className="filter-group">
            <label>Bloco Final:</label>
            <input 
              type="text" 
              value={filters.endBlock}
              onChange={(e) => setFilters({...filters, endBlock: e.target.value})}
              className="wallet-input"
              placeholder="latest"
            />
          </div>

          <div className="filter-group">
            <label>Fonte de Dados:</label>
            <select 
              value={filters.source}
              onChange={(e) => setFilters({...filters, source: e.target.value})}
              className="wallet-input"
            >
              <option value="graphql_hashes">GraphQL Hashes + Chain (Rápido)</option>
              <option value="blockchain">Blockchain Completa (Pode demorar)</option>
            </select>
          </div>

          <div className="filter-group">
            <label>Tipos de Evento:</label>
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
          onClick={loadEvents}
          disabled={loading}
          className="wallet-button"
        >
          {loading ? 'Carregando...' : 'Carregar Eventos'}
        </button>
        
        <button 
          onClick={() => exportToFile('csv')}
          disabled={exporting || transactionHistory.length === 0}
          className="donate-button"
        >
          {exporting ? 'Exportando...' : `Exportar CSV (${transactionHistory.length})`}
        </button>
        
        <button 
          onClick={() => exportToFile('json')}
          disabled={exporting || transactionHistory.length === 0}
          className="donate-button"
        >
          {exporting ? 'Exportando...' : `Exportar JSON (${transactionHistory.length})`}
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
          <h3>Resumo de Transações</h3>
          
          <div className="stats-grid">
            <div>
              <strong>Total de Transações</strong>
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
            <h4>Pré-visualização (Primeiros 10)</h4>
            
            {transactionHistory.slice(0, 10).map((transaction, index) => (
              <div key={index} className="contract-item">
                <div className="contract-header">
                  <span className="status-badge">{transaction.type}</span>
                  <span>Bloco #{transaction.blockNumber}</span>
                </div>
                <div className="details-grid">
                  <div className="detail-item">
                    <strong>Quantidade:</strong>
                    <span>{transaction.amount || '0'}</span>
                  </div>
                  <div className="detail-item">
                    <strong>Endereço:</strong>
                    <span className="user-address" style={{ fontSize: '12px' }}>
                      {transaction.address || 'N/A'}
                    </span>
                  </div>
                  <div className="detail-item">
                    <strong>Transação:</strong>
                    <span className="user-address" style={{ fontSize: '12px' }}>
                      {transaction.transactionHash?.slice(0, 10)}...
                    </span>
                  </div>
                  <div className="detail-item">
                    <strong>Horário:</strong>
                    <span>{new Date(transaction.timestamp).toLocaleString()}</span>
                  </div>
                </div>
              </div>
            ))}
            
            {transactionHistory.length > 10 && (
              <div className="vinculation-info">
                Mostrando 10 de {transactionHistory.length} transações
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ModeratorPanel;