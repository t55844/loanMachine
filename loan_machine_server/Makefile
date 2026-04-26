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