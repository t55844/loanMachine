@echo off

echo --- 1. Building and starting all DEV containers... ---
rem Use -f to specify your new dev-compose file
docker-compose -f docker-compose.dev.yml up -d --build

echo.
echo --- 2. Waiting for blockchain node to be ready... ---
timeout /t 10 /nobreak > NUL

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
rem NOTE: You must have 'graph-cli' installed globally or locally on your host machine to run these commands
call graph codegen
call graph build

echo ^> 5b. Creating and deploying subgraph to local Graph Node...
rem The name 'loan-machine' MUST MATCH the name the frontend is querying (which is 'loan-machine')
rem The port 8020 is for deployments (as per your docker-compose file)

rem Use 'call' to ensure the batch script continues after these commands
call graph create --node http://localhost:8020/ loan-machine
call graph deploy --node http://localhost:8020/ --ipfs http://localhost:5001/ loan-machine --version-label v0.0.1

echo.
echo --- DEV SETUP COMPLETE! ---
echo Your full stack is running in DEV mode:
echo ----------------------------------------
echo ^> Frontend (HOT-RELOAD): http://localhost:5173
echo ^> GraphQL API (Query Endpoint): http://localhost:8000/subgraphs/name/loan-machine
echo ^> Blockchain Node:     http://localhost:8545
echo ^> Deployed Contracts:  ./hardhat/deployment-addresses.json
echo ----------------------------------------
echo.
echo "To see logs, run: docker-compose -f docker-compose.dev.yml logs -f"
echo "To stop all services, run: docker-compose -f docker-compose.dev.yml down"