.PHONY: test-web test-core test-one test-all

# Web crate: SSR + hydrate snapshot tests via cargo-leptos
test-web:
	cargo leptos test

# Core crate: all unit + integration tests. Needs `deployable` feature
# for the Anvil-backed fixtures in tests/common/deploy.rs.
test-core:
	cargo test -p loan_machine_core --features deployable --no-fail-fast

# Single test by name. Usage:  make test-one NAME=my_test
test-one:
	cargo test -p loan_machine_core --features deployable $(NAME) -- --exact --nocapture

test-all: test-web test-core

fund-anvil-wallet:
	curl -X POST http://localhost:8545  -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"anvil_setBalance", "params":["$(WALLET)","0x8AC7230489E80000"],  "id":1}'

deploy-local:
	@curl -sS -o /dev/null http://localhost:8545 -X POST \
	  -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"web3_clientVersion","id":1}' \
	  || (echo "✗ Anvil not running. Start it first: anvil"; exit 1)
	@curl -sS -X POST http://localhost:8545 \
	  -H "Content-Type: application/json" \
	  -d '{"jsonrpc":"2.0","method":"anvil_setBalance","params":["0xD177510Ae747648f897F1DeD140Ee057EfffDebe","0x8AC7230489E80000"],"id":1}' \
	  > /dev/null
	cargo run -p loan_machine_core --bin deploy_local --features deployable

deploy-and-serve: deploy-local
	RUST_LOG=debug cargo-leptos watch

# cast balance 0x9535424e6F3F82C9c3aCA0d747C59E187f216Ac4 --rpc-url http://localhost:8545
# tree -I 'node_modules|target|dist|build|venv|.venv|__pycache__|.git|.ds_store|data'