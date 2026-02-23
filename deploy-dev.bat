@echo off
rem deploy-dev.bat - FULL DOCKER (Rust + Subgraph no container)
echo --- 0. Resetting GraphQL data... ---
rmdir /s /q .\data\postgres 2>nul
rmdir /s /q .\data\ipfs 2>nul
mkdir .\data\postgres 2>nul
mkdir .\data\ipfs 2>nul

echo --- 1. Building and starting all DEV containers... ---
docker-compose -f docker-compose.dev.yml up -d --build

echo.
echo --- 2. Waiting for services to be ready... ---
ping 127.0.0.1 -n 61 > nul   # ~60 segundos, compatível com qualquer Windows

echo.
echo --- 3. Deploying smart contracts... ---
docker-compose -f docker-compose.dev.yml exec hardhat-node npx hardhat run --network localhost scripts/deploy.js

rem echo.
rem echo --- 4. Copying deployment addresses to frontend... ---
rem docker-compose -f docker-compose.dev.yml cp hardhat-node:/app/deployment-addresses.json ./frontend/public/deployment-addresses.json

echo.
echo --- 5. Deploying Subgraph (100% no Docker)... ---
docker-compose -f docker-compose.dev.yml run --rm subgraph-deploy

echo.
echo --- DEV SETUP COMPLETE! ---
echo Your full stack is running in DEV mode:
echo ----------------------------------------
echo ^> Frontend (HOT-RELOAD): http://localhost:5173
echo ^> Rust API: http://localhost:3002
echo ^> GraphQL API: http://localhost:8000/subgraphs/name/loan-machine
echo ^> Blockchain Node: http://localhost:8545
echo ^> Deployed Contracts: ./hardhat/deployment-addresses.json
echo ----------------------------------------
echo.
echo "To see logs: docker-compose -f docker-compose.dev.yml logs -f"
echo "To stop: docker-compose -f docker-compose.dev.yml down"
echo "To redeploy subgraph only: docker-compose -f docker-compose.dev.yml run --rm subgraph-deploy"