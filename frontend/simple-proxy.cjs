const express = require('express');
const app = express();

RPC_URL='https://eth-sepolia.g.alchemy.com/v2/0fqMwbDl3V0vDtxg1tXTk'
SUBGRAPH_URL='https://api.studio.thegraph.com/query/1714606/thiago-first-project/v0.3.6'
CONTRACT_ADDRESS='0xE797948c05aa26369825bA03D2b5e0eBB4ed28C1'
MOCK_USDT_ADDRESS='0x2107997bd769396b1B1f05A872f4e0a2BF16d54A' 
REPUTATION_CONTRACT_ADDRESS='0xf9B64b3242DDFc7627cd764825617e6d9310Ce95'
VALUE_TO_MINT=10

// Permitir CORS para o frontend
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.header('Access-Control-Allow-Headers', 'Content-Type');
  next();
});

app.use(express.json());

// Rota idêntica ao que você tem no Vercel
app.all('/api/proxy', async (req, res) => {
  const { type } = req.query;

  try {
    if (type === 'config') {
      return res.json({
        rpcUrl: RPC_URL,
        subgraphUrl: SUBGRAPH_URL,
        contractAddress: CONTRACT_ADDRESS,
        mockUsdtAddress: MOCK_USDT_ADDRESS,
        reputationContractAddress: REPUTATION_CONTRACT_ADDRESS,
        valueToMint: VALUE_TO_MINT || '10'
      });
    }

    if (type === 'graphql') {
      const { query, variables } = req.body;
      // Aqui você coloca sua lógica GraphQL real
      return res.json({ data: { /* seus dados aqui */ } });
    }
    
    return res.json({ status: 'ok' });
  } catch (error) {
    res.json({ error: error.message });
  }
});

app.listen(3001, () => {
  console.log('✅ Proxy local rodando em: http://localhost:3001');
});