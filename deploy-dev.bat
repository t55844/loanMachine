@echo off
rem deploy-dev.bat - Script to deploy the full stack in DEV mode
echo --- 0. Resetting GraphQL data... ---
rmdir /s /q .\data\postgres 2>nul
rmdir /s /q .\data\ipfs 2>nul
mkdir .\data\postgres 2>nul
mkdir .\data\ipfs 2>nul

echo --- 1. Building and starting all DEV containers... ---
rem Use -f to specify your new dev-compose file
docker-compose -f docker-compose.dev.yml up -d --build

echo.
echo --- 2. Waiting for blockchain node and proxy to be ready... ---
timeout /t 15 > NUL # Longer for proxy

echo.
echo --- 3. Deploying smart contracts... ---
rem This command is the same as before
docker-compose -f docker-compose.dev.yml exec hardhat-node npx hardhat run --network localhost scripts/deploy.js

echo.
echo --- 4. Copying deployment addresses to frontend... ---
rem This copies the addresses to a place the dev server can find them
COPY .\hardhat\deployment-addresses.json .\frontend\public\deployment-addresses.json

echo.
echo --- 5. Building and Deploying Subgraph (CRITICAL) ---

echo ^> 5a. Generating code and building WASM...
cd hardhat\scripts\graphql
call npx graph codegen
call npx graph build

echo ^> 5b. Creating and deploying subgraph to local Graph Node...
call graph create --node http://localhost:8020/ loan-machine
call graph deploy --node http://localhost:8020/ --ipfs http://localhost:5001/ loan-machine --version-label v0.0.1
cd ..\..\..

echo.
echo --- DEV SETUP COMPLETE! ---
echo Your full stack is running in DEV mode:
echo ----------------------------------------
echo ^> Frontend (HOT-RELOAD): http://localhost:5173
echo ^> Proxy: http://localhost:3001/api/proxy
echo ^> GraphQL API (Query Endpoint): http://localhost:8000/subgraphs/name/loan-machine
echo ^> Blockchain Node:     http://localhost:8545
echo ^> Deployed Contracts:  ./hardhat/deployment-addresses.json
echo ----------------------------------------
echo.
echo "To see logs, run: docker-compose -f docker-compose.dev.yml logs -f"
echo "To stop all services, run: docker-compose -f docker-compose.dev.yml down"