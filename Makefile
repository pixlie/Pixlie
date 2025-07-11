# Pixlie Development Makefile

.PHONY: help setup check check-rust check-webapp build build-rust build-webapp test test-rust clean install-hooks

# Default target
help:
	@echo "Pixlie Development Commands"
	@echo "=========================="
	@echo ""
	@echo "Setup:"
	@echo "  setup          - Setup development environment (git hooks, dependencies)"
	@echo "  install-hooks  - Install git pre-commit hooks"
	@echo ""
	@echo "Development:"
	@echo "  check          - Run all checks (format, lint, test)"
	@echo "  check-rust     - Run Rust checks (format, clippy, test)"
	@echo "  check-webapp   - Run webapp checks (typecheck, lint)"
	@echo "  build          - Build all projects"
	@echo "  build-rust     - Build Rust backend"
	@echo "  build-webapp   - Build React webapp"
	@echo "  test           - Run all tests"
	@echo "  test-rust      - Run Rust tests"
	@echo "  clean          - Clean build artifacts"
	@echo ""
	@echo "CI Commands:"
	@echo "  ci-rust        - Run full Rust CI pipeline"
	@echo "  ci-webapp      - Run full webapp CI pipeline"

# Setup
setup: install-hooks
	@echo "🔧 Setting up development environment..."
	cd webapp && pnpm install

install-hooks:
	@echo "🪝 Installing git hooks..."
	./setup-hooks.sh

# Rust commands
check-rust:
	@echo "🦀 Running Rust checks..."
	cd pixlie && cargo fmt --all -- --check
	cd pixlie && cargo clippy --all-targets --all-features -- -D warnings
	cd pixlie && cargo test

build-rust:
	@echo "🔨 Building Rust backend..."
	cd pixlie && cargo build

test-rust:
	@echo "🧪 Running Rust tests..."
	cd pixlie && cargo test

ci-rust: check-rust build-rust
	@echo "✅ Rust CI pipeline completed"

# Webapp commands
check-webapp:
	@echo "⚛️  Running webapp checks..."
	cd webapp && pnpm install --frozen-lockfile
	cd webapp && pnpm exec tsc -b --noEmit
	cd webapp && pnpm lint

build-webapp:
	@echo "🔨 Building webapp..."
	cd webapp && pnpm build

ci-webapp: check-webapp build-webapp
	@echo "✅ Webapp CI pipeline completed"

# Combined commands
check: check-rust check-webapp
	@echo "✅ All checks passed"

build: build-rust build-webapp
	@echo "✅ All projects built"

test: test-rust
	@echo "✅ All tests passed"

# Cleanup
clean:
	@echo "🧹 Cleaning build artifacts..."
	cd pixlie && cargo clean
	cd webapp && rm -rf dist node_modules/.vite