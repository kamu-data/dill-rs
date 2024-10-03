###############################################################################
# Lint
###############################################################################

.PHONY: lint
lint:
	cargo fmt --check
	# cargo udeps --all-targets
	cargo clippy --workspace --all-targets -- -D warnings


###############################################################################
# Lint (with fixes)
###############################################################################

.PHONY: lint-fix
lint-fix:
	cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged --broken-code
	cargo fmt --all

###############################################################################
# Test
###############################################################################

.PHONY: test
test:
	cargo test --verbose --workspace
