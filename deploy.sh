#!/bin/bash

# Ensure 'graph-cli' is installed on your host machine before running!

echo "--- 1. Building and starting all containers... ---"
# Start all services in the background (--build forces a rebuild)
docker-compose up -d --build

echo "\n--- 2. Waiting for blockchain node to be ready... ---"
# Give the node a few seconds to start up
sleep 10

echo "\n--- 3. Deploying smart contracts... ---"
# Use 'docker-compose exec' to run your deploy script
# INSIDE the already-running 'hardhat-node' container
docker-compose exec hardhat-node npx hardhat run --network localhost scripts/deploy.js

# --- 4. BUILD AND DEPLOY SUBGRAPH (CRITICAL ADDITION) ---
echo "\n--- 4. Building and Deploying Subgraph... ---"
# This step is essential for Graph Node to start indexing the chain.
echo "-> 4a. Generating code and building WASM..."
graph codegen
graph build

echo "-> 4b. Creating and deploying subgraph 'loan-machine' to local Graph Node..."
# The name 'loan-machine' MUST match the name the frontend is querying.
graph create --node http://localhost:8020/ loan-machine || true
graph deploy --node http://localhost:8020/ --ipfs http://localhost:5001/ loan-machine --version-label v1.0.0

echo "\n--- ✅ SETUP COMPLETE! ---"
echo "Your full stack is running:"
echo "----------------------------------------"
echo "🚀 Frontend: http://localhost:3000"
echo "🧠 GraphQL API (Query Endpoint): http://localhost:8000/subgraphs/name/loan-machine"
echo "⛓️  Blockchain Node: http://localhost:8545"
echo "📄 Deployed Contracts: ./hardhat/deployment-addresses.json"
echo "----------------------------------------"
echo "\nTo see logs from all services, run: docker-compose logs -f"
echo "To stop all services, run: docker-compose down"