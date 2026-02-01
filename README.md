
to com preguiça de mexer e traduzir, mas como tudo ta em docker e tem um .bat que roda tudo automatico, só ignore o texto abaixo

========================================================================================================================
1. Blockchain Sector (Your Business Logic)

This is your smart contract running on Ethereum (or local Hardhat/Anvil/Ganache).

It handles:

Business rules (loans, donations, repayments).

Storing only the minimum required data on-chain.

Emitting events whenever something important happens.
Example: Donated(donor, amount).

Think of this as the official record keeper. It’s expensive and slow because it’s decentralized and permanent.

https://etherscan.io/apidashboard

https://www.alchemy.com/faucets/ethereum-sepolia
https://faucet.triangleplatform.com/ethereum/sepolia

https://thegraph.com/studio/subgraph/

2. The Graph Node Sector (Your Data Indexer)

Watches the blockchain for your events.

Runs your subgraph mappings → translates blockchain data into structured entities (like tables in SQL).

Stores everything in a PostgreSQL database.

Gives you a GraphQL API to query that data super fast.

This sector is read-heavy, optimized for fast queries, not for writing logic.

3. Frontend Sector (Input/Output Layer)

Users interact with this layer (website, dApp, etc.).

For actions that change state (donate, borrow, repay):

Frontend sends transactions to Blockchain Sector (costs gas).

For data display (list donations, show stats):

Frontend queries The Graph Node via GraphQL (free & instant).

4. Summary Diagram
   [Frontend UI]
   │
   │ (Transactions → write data, costs gas)
   ▼
   [Blockchain: Smart Contracts] <── emits events ──┐
   │ │
   │ (Read queries, free) │
   ▼ │
   [The Graph Node] ───► [PostgreSQL DB] ◄── IPFS (stores subgraph files)
   │
   │ (GraphQL API, free queries)
   ▼
   [Frontend UI displays data]

So yes, that’s the full picture:

Blockchain → official records, expensive writes.

Graph Node → event listener + data indexer, free queries.

Frontend → connects everything together.

---

explication expanded about Graphql Node:

1. What started up

Postgres → stores your subgraph data (indexed events → queryable tables).

IPFS → stores your subgraph metadata and mappings.

Graph Node → connects to your blockchain, listens for events, transforms them into GraphQL entities, and stores them in Postgres.

2. Key URLs to use now

GraphQL endpoint:
http://localhost:8000 → you’ll query your data here.

Admin endpoint:
http://localhost:8020 → used by graph deploy CLI.

IPFS Web UI:
http://localhost:5001/webui

Subgraph Index Node:
http://localhost:8030 → internal index info.

Metrics:
http://localhost:8040 → Prometheus metrics.

3. Next steps: Deploy your subgraph

Now that the node is running, you can:

graph create --node http://localhost:8020/ loan-machine
graph deploy --node http://localhost:8020/ --ipfs http://localhost:5001 loan-machine

This will take your subgraph.yaml + schema.graphql + mappings and register them with the node.

4. Query your data

Once deployed, open:

http://localhost:8000/subgraphs/name/loan-machine/graphql

You can run GraphQL queries like:

{
donations {
id
amount
donor
timestamp
}
}

5. Big picture

Your blockchain emits events → Graph Node listens → stores them in Postgres → exposes a GraphQL API.

Queries on GraphQL are fast and don’t cost gas, because they read from Postgres, not the chain.
