@echo off

rem Ensure 'graph-cli' is installed on your host machine before running!

echo --- 1. Building and starting all containers... ---
rem Start all services in the background (--build forces a rebuild)
docker-compose up -d --build

echo.
echo --- 2. Waiting for blockchain node to be ready... ---
rem Give the node a few seconds to start up
timeout /t 10 /nobreak > NUL

echo.
echo --- 3. Deploying smart contracts... ---
rem Use 'docker-compose exec' to run your deploy script
rem INSIDE the already-running 'hardhat-node' container
docker-compose exec hardhat-node npx hardhat run --network localhost scripts/deploy.js

echo.
echo --- 4. Building and Deploying Subgraph (CRITICAL) ---

echo ^> 4a. Generating code and building WASM...
rem Use 'call' to ensure the batch script continues after these commands
call graph codegen
call graph build

echo ^> 4b. Creating and deploying subgraph 'loan-machine' to local Graph Node...
rem The name 'loan-machine' MUST match the name the frontend is querying.
call graph create --node http://localhost:8020/ loan-machine
call graph deploy --node http://localhost:8020/ --ipfs http://localhost:5001/ loan-machine --version-label v1.0.0

echo.
echo --- ✅ SETUP COMPLETE! ---
echo Your full stack is running:
echo ----------------------------------------
echo ^> Frontend: http://localhost:3000
echo ^> GraphQL API (Query Endpoint): http://localhost:8000/subgraphs/name/loan-machine
echo ^> Blockchain Node: http://localhost:8545
echo ^> Deployed Contracts: ./hardhat/deployment-addresses.json
echo ----------------------------------------
echo.
echo To see logs from all services, run: docker-compose logs -f
echo To stop all services, run: docker-compose down 