#!/bin/bash

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

echo "\n--- ✅ SETUP COMPLETE! ---"
echo "Your full stack is running:"
echo "----------------------------------------"
echo "🚀 Frontend: http://localhost:3000"
echo "🧠 GraphQL API: http://localhost:8000"
echo "⛓️  Blockchain Node: http://localhost:8545"
echo "📄 Deployed Contracts: ./hardhat/deployment-addresses.json"
echo "----------------------------------------"
echo "\nTo see logs from all services, run: docker-compose logs -f"
echo "To stop all services, run: docker-compose down"