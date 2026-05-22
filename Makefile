# ─────────────────────────────────────────────────────────────
# Run from repo root: ~/projects/loan-machine/
# ─────────────────────────────────────────────────────────────

.PHONY: test-web test-core test-one test-all \
        fund-anvil-wallet deploy-local \
        graph-up graph-down graph-clean graph-logs graph-status graph-query \
        stack-up

ENV_FILE  := loan_machine_server/.env
COMPOSE   := docker compose -f docker-compose.dev.yml --env-file $(ENV_FILE)
CARGO     := cargo

# ── Tests ─────────────────────────────────────────────────────

# ── Tests ─────────────────────────────────────────────────────

test-web:
	cd loan_machine_server && $(CARGO) test -p loan_machine_web --no-fail-fast

test-core:
	cd loan_machine_server && $(CARGO) test -p loan_machine_core --features deployable --no-fail-fast

test-models:
	cd loan_machine_server && $(CARGO) test -p loan_machine_models --no-fail-fast

# Run every package's tests. Don't stop on failure — report at the end.
test-all:
	@cd loan_machine_server; \
	set +e; \
	echo "═══ loan_machine_models ═══"; \
	$(CARGO) test -p loan_machine_models --no-fail-fast; models=$$?; \
	echo "═══ loan_machine_core ═══"; \
	$(CARGO) test -p loan_machine_core --features deployable --no-fail-fast; core=$$?; \
	echo "═══ loan_machine_web ═══"; \
	$(CARGO) test -p loan_machine_web --no-fail-fast; web=$$?; \
	echo; \
	echo "═══ Summary ═══"; \
	[ $$models -eq 0 ] && echo "  models  PASS" || echo "  models  FAIL"; \
	[ $$core   -eq 0 ] && echo "  core    PASS" || echo "  core    FAIL"; \
	[ $$web    -eq 0 ] && echo "  web     PASS" || echo "  web     FAIL"; \
	exit $$((models + core + web))

# Usage: make test-one NAME=my_test
test-one:
	cd loan_machine_server && $(CARGO) test -p loan_machine_core --features deployable $(NAME) -- --exact --nocapture

# Usage: make test-web-one NAME=auth_header_value
test-web-one:
	cd loan_machine_server && $(CARGO) test -p loan_machine_web $(NAME) -- --nocapture

# ── Chain helpers ─────────────────────────────────────────────
compile:
	cd ./hardhat && npx hardhat clean && npx hardhat compile

fund-anvil-wallet:
	curl -X POST http://localhost:8545 \
	  -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"anvil_setBalance","params":["$(WALLET)","0x8AC7230489E80000"],"id":1}'

fund-used-wallets:
	$(MAKE) fund-anvil-wallet WALLET=0x8Fb852022882B2AA3C3a0fE39f3169CdC01C887D
	$(MAKE) fund-anvil-wallet WALLET=0x9535424e6F3F82C9c3aCA0d747C59E187f216Ac4
	$(MAKE) fund-anvil-wallet WALLET=0x19dc8f07697c114ce0303272e494f8eb1bd635f1


deploy-local:
	@curl -sS -o /dev/null http://localhost:8545 \
	  -X POST -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"web3_clientVersion","id":1}' \
	  || (echo "✗ Anvil not running — start it first: anvil --host 0.0.0.0"; exit 1)
	@curl -sS -X POST http://localhost:8545 \
	  -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"anvil_setBalance","params":["0xD177510Ae747648f897F1DeD140Ee057EfffDebe","0x8AC7230489E80000"],"id":1}' \
	  > /dev/null
	cd loan_machine_server && $(CARGO) run -p loan_machine_core --bin deploy_local --features deployable

# ── Subgraph ──────────────────────────────────────────────────
local-graph-codegen:
	rm -rf ./scripts/graphql/generated ./generated && cd ./hardhat/scripts/graphql && graph codegen --output-dir ./generated

local-graph-build:
	rm -rf ./scripts/graphql/build ./build && cd ./hardhat/scripts/graphql && graph build --output-dir ./build


graph-up:
	@grep -qE '^COOP_REGISTRY_ADDRESS=["\x27]?0x' $(ENV_FILE) 2>/dev/null \
	  || (echo "✗ COOP_REGISTRY_ADDRESS not set in $(ENV_FILE) — run: make deploy-local"; exit 1)
	$(COMPOSE) up -d ipfs postgres graph-node
	@echo "⏳  Waiting for graph-node on :8020..."
	@until [ "$$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8020 2>/dev/null)" != "000" ]; do sleep 1; done
	@echo "✓  graph-node ready"
	$(COMPOSE) up subgraph-deploy

graph-down:
	$(COMPOSE) down

graph-clean:
	$(COMPOSE) down -v
	rm -rf data/postgres data/ipfs

graph-logs:
	$(COMPOSE) logs -f graph-node

graph-status:
	@echo "── Indexing status ──────────────────────────────────"
	@curl -s -X POST http://localhost:8030/graphql \
	  -H "Content-Type: application/json" \
	  -d '{"query":"{ indexingStatuses { subgraph health synced chains { latestBlock { number } } } }"}' \
	  | jq .
	@echo "── Meta ─────────────────────────────────────────────"
	@curl -s -X POST http://localhost:8000/subgraphs/name/loan-machine \
	  -H "Content-Type: application/json" \
	  -d '{"query":"{ _meta { block { number } hasIndexingErrors } }"}' \
	  | jq .

graph-query:
	@curl -s -X POST http://localhost:8000/subgraphs/name/loan-machine \
	  -H "Content-Type: application/json" \
	  -d '{"query":"{ cooperatives { id name loanMachine active registeredAt } }"}' \
	  | jq .

# ── Full stack ────────────────────────────────────────────────

# anvil must already be running: anvil --host 0.0.0.0
stack-up:
	@curl -sS -o /dev/null http://localhost:8545 \
	  -X POST -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"web3_clientVersion","id":1}' \
	  || (echo "✗ Anvil not running — start it first: anvil --host 0.0.0.0"; exit 1)
	@curl -sS -X POST http://localhost:8545 \
	  -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"anvil_setBalance","params":["0xD177510Ae747648f897F1DeD140Ee057EfffDebe","0x8AC7230489E80000"],"id":1}' \
	  > /dev/null
	cd loan_machine_server && $(CARGO) run -p loan_machine_core --bin deploy_local --features deployable
	$(COMPOSE) down -v
	sudo rm -rf data/postgres data/ipfs
	$(MAKE) graph-up
	cd loan_machine_server && RUST_LOG=debug cargo-leptos watch
# ── Misc ──────────────────────────────────────────────────────

tree-print:
	tree -I 'node_modules|target|dist|build|venv|.venv|__pycache__|.git|.ds_store|data'

# 1- rever como o AppState se relaciona com os itens e se há a necessidade de 
#colocar todo o service em um Arc

# 2- rever todos os retornos de função e seguir o principio de que tudo retornado
#tem que ser ownable