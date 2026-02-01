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
        if (!rpcResponse.ok) {
          const errorText = await rpcResponse.text();
          console.error('RPC Proxy Error:', rpcResponse.status, errorText);
          throw new Error(`RPC failed: ${rpcResponse.status} - ${errorText}`);
        }
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
        if (!graphResponse.ok) {
          const errorText = await graphResponse.text();
          console.error('GraphQL Proxy Error:', graphResponse.status, errorText, { query, variables });
          throw new Error(`GraphQL failed: ${graphResponse.status} - ${errorText}`);
        }
        const graphData = await graphResponse.json();
        if (graphData.errors) {
          console.error('GraphQL Response Errors:', graphData.errors);
          // Forward errors to client for better debugging
          return res.status(200).json(graphData); // Still 200, but with errors array
        }
        return res.status(200).json(graphData);

      default:
        return res.status(400).json({ error: 'Invalid proxy type' });
    }
  } catch (error) {
    console.error('Proxy Error:', type, error.message);
    res.status(500).json({ error: 'Internal server error', details: error.message });
  }
}