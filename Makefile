# Default target
.PHONY: all
all: build

# Build the project
.PHONY: build
build:
	cargo build

# Clean the project
.PHONY: clean
clean:
	cargo clean

# Run tests
.PHONY: test
test:
	cargo test

# Format the code
.PHONY: format
format:
	cargo fmt

# Update dependencies
.PHONY: update
update:
	cargo update

# Dry run upgrade
.PHONY: dry_run_upgrade
dry_run_upgrade:
	cargo upgrade -i --dry-run

# Upgrade dependencies
.PHONY: upgrade
upgrade:
	cargo upgrade

# Lint the code
.PHONY: lint
lint:
	cargo clippy -- -D warnings
