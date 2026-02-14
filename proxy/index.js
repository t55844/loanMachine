const express = require('express');
const app = express();

app.use(express.json());

// Improved CORS middleware that properly handles preflight (OPTIONS)
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');           // or 'http://localhost:5173' for stricter
  res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  // Handle preflight OPTIONS request
  if (req.method === 'OPTIONS') {
    return res.sendStatus(204);  // Important: 204 No Content for preflight
  }

  next();
});


app.all('/api/proxy', async (req, res) => {
  const { type } = req.query;
  console.log('Request:', req.method, type, req.body);  // NEW: Log to debug

  console.log('=== DEBUG ===');
  console.log('Method:', req.method);
  console.log('Raw query:', req.query);
  console.log('Parsed type:', type);
  console.log('Typeof type:', typeof type);
  console.log('==============');

  if (req.method !== 'POST' && type !== 'config') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    switch (type) {
      case 'config':
        return res.status(200).json({
          contractAddress: process.env.CONTRACT_ADDRESS,
          mockUsdAddress: process.env.MOCK_USDT_ADDRESS,
          reputationContractAddress: process.env.REPUTATION_CONTRACT_ADDRESS,
          valueToMint: process.env.VALUE_TO_MINT || '10',
          // Only include rpcUrl/subgraphUrl if you really need them client-side (usually not)
        });

      case 'rpc':
        if (!req.body || !req.body.method) {
          return res.status(400).json({ error: 'Invalid RPC body' });  // NEW: Explicit 400
        }
        const { method, params, id } = req.body;
        const rpcResponse = await fetch(process.env.RPC_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ jsonrpc: '2.0', id: id || 1, method, params: params || [] }),  // NEW: Defaults
        });
        if (!rpcResponse.ok) {
          const errorText = await rpcResponse.text();
          console.error('RPC Proxy Error:', rpcResponse.status, errorText);
          return res.status(rpcResponse.status).json({ error: errorText });  // NEW: Forward status
        }
        const rpcData = await rpcResponse.json();
        return res.status(200).json(rpcData);

      case 'graphql':
        if (!req.body || !req.body.query) {
          return res.status(400).json({ error: 'Invalid GraphQL body' });  // NEW: Explicit 400
        }
        const { query, variables } = req.body;
        const graphResponse = await fetch(process.env.SUBGRAPH_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query, variables: variables || {} }),  // NEW: Defaults
        });
        if (!graphResponse.ok) {
          const errorText = await graphResponse.text();
          console.error('GraphQL Proxy Error:', graphResponse.status, errorText);
          return res.status(graphResponse.status).json({ error: errorText });  // NEW: Forward
        }
        const graphData = await graphResponse.json();
        return res.status(200).json(graphData);

      default:
        return res.status(400).json({ error: 'Invalid proxy type' });
    }
  } catch (error) {
    console.error('Proxy Error:', type, error.message);
    res.status(500).json({ error: 'Internal server error', details: error.message });
  }
});


const port = process.env.PORT || 3001;
app.listen(port, () => {
  console.log(`Proxy server running on http://localhost:${port}`);
});