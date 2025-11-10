// api/proxy.js - Single serverless proxy for all sensitive vars and requests
export default async function handler(req, res) {
  const { type } = req.query; // Route based on ?type= (e.g., /api/proxy?type=rpc)

  if (req.method !== 'POST' && type !== 'config') {
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    switch (type) {
      case 'config':
        // Return public-ish config (addresses) - Still hidden behind server
        return res.status(200).json({
          contractAddress: process.env.CONTRACT_ADDRESS,
          mockUsdtAddress: process.env.MOCK_USDT_ADDRESS,
          reputationContractAddress: process.env.REPUTATION_CONTRACT_ADDRESS,
          valueToMint: process.env.VALUE_TO_MINT,
        });

      case 'rpc':
        // Proxy JSON-RPC calls
        const { method, params, id } = req.body;
        const rpcResponse = await fetch(process.env.RPC_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ jsonrpc: '2.0', id, method, params }),
        });
        if (!rpcResponse.ok) throw new Error('RPC failed');
        const rpcData = await rpcResponse.json();
        return res.status(200).json(rpcData);

      case 'graphql':
        // Proxy GraphQL requests
        const { query, variables } = req.body;
        const graphResponse = await fetch(process.env.SUBGRAPH_URL, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query, variables }),
        });
        if (!graphResponse.ok) throw new Error('GraphQL failed');
        const graphData = await graphResponse.json();
        return res.status(200).json(graphData);

      default:
        return res.status(400).json({ error: 'Invalid proxy type' });
    }
  } catch (error) {
    console.error('Proxy Error:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
}