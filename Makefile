.PHONY: setup-hooks fmt check all


# Run formatting, checks, and tests
all: fmt check test

# Configure Git to use the repository-managed hooks directory
setup-hooks:
	@chmod +x .githooks/pre-commit
	@git config core.hooksPath .githooks
	@printf "Git hooks path configured to .githooks/\n"

# Format all code
fmt:
	@cargo fmt --all
	@cargo sort-derives

# Run all validation checks
check:
	@cargo fmt --all --check
	@cargo sort-derives --check
	@cargo clippy --quiet

# Run all tests
test:
	@cargo test
