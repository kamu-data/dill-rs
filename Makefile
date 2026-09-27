STABLE_FEATURES="--features=tokio"

###############################################################################
# Lint
###############################################################################

.PHONY: lint
lint:
	cargo fmt --check
	# cargo udeps --all-targets
	cargo clippy --workspace --all-targets ${STABLE_FEATURES} -- -D warnings


###############################################################################
# Lint (with fixes)
###############################################################################

.PHONY: lint-fix
lint-fix:
	cargo clippy --workspace --all-targets ${STABLE_FEATURES} --fix --allow-dirty --allow-staged --broken-code
	cargo fmt --all

###############################################################################
# Test
###############################################################################

.PHONY: test
test:
	cargo test --workspace ${STABLE_FEATURES}


.PHONY: test-nightly
test-nightly:
	cargo +nightly test --workspace --all-features
