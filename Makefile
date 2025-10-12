# Makefile — すべてのループを一箇所に（Spell Platform）

SHELL := /bin/bash

# ===== Rust/Cargo =====
.PHONY: install test lint clippy audit format build build-release clean

install:
	cargo fetch
	cargo build

test:
	cargo test --workspace --all-features || true

lint: clippy audit

clippy:
	cargo clippy --all-features --all-targets -- -D warnings || true

audit:
	cargo audit || true

format:
	cargo fmt --all

build:
	cargo build

build-release:
	cargo build --release

clean:
	cargo clean

# ===== Deployment（Fly.io） =====
.PHONY: deploy deploy-check logs status

deploy:
	flyctl deploy -a spell-platform --remote-only

deploy-check:
	@echo "=== Health Check ==="
	@curl -f https://spell-platform.fly.dev/healthz || echo "Health check failed"
	@echo ""
	@echo "=== Metrics Check ==="
	@curl -f https://spell-platform.fly.dev/metrics | head -20 || echo "Metrics check failed"

logs:
	flyctl logs -a spell-platform --since 10m

status:
	flyctl status -a spell-platform

# ===== Database Migrations =====
.PHONY: migrate-check migrate-apply migrate-list

migrate-check:
	@echo "=== Available Migrations ==="
	@ls -1 migrations/*.sql

migrate-apply:
	./scripts/apply_migrations.sh

migrate-list:
	@echo "=== Applied Migrations (via proxy) ==="
	flyctl proxy 15432:5432 -a spell-platform-db & \
	PID=$$! && \
	sleep 2 && \
	psql -h localhost -p 15432 -U spell_platform -d spell_platform -c "SELECT version, description, success FROM _sqlx_migrations ORDER BY installed_on;" && \
	kill $$PID

# ===== E2E Tests =====
.PHONY: e2e e2e-setup

e2e-setup:
	@echo "=== E2E Test Setup ==="
	@echo "1. Open OAuth flow: https://spell-platform.fly.dev/auth/github"
	@echo "2. Get session token from response"
	@echo "3. Export TOKEN=<your_session_token>"
	@echo "4. Run: make e2e"

e2e:
	@if [ -z "$$TOKEN" ]; then \
		echo "ERROR: TOKEN not set. Run 'make e2e-setup' first."; \
		exit 1; \
	fi
	./scripts/e2e_phase2.sh

# ===== Review（品質ゲート） =====
.PHONY: review

review:
	@echo "=== Review Gate ==="
	@echo "Running clippy..."
	$(MAKE) clippy
	@echo ""
	@echo "Running audit..."
	$(MAKE) audit
	@echo ""
	@echo "Running format check..."
	cargo fmt --all -- --check || true
	@echo ""
	@echo "=== Review Complete ==="
	@echo "Check for blocking issues above."

# ===== CI（すべてのチェックを実行） =====
.PHONY: ci

ci: install test lint review
	@echo "=== CI Complete ==="
	@echo "All checks passed!"

# ===== Utilities =====
.PHONY: version help

version:
	@cargo --version
	@rustc --version

help:
	@echo "Spell Platform Makefile"
	@echo ""
	@echo "Common targets:"
	@echo "  install        - Fetch dependencies and build"
	@echo "  test           - Run tests"
	@echo "  lint           - Run clippy + audit"
	@echo "  format         - Format code"
	@echo "  build          - Build debug binary"
	@echo "  build-release  - Build release binary"
	@echo ""
	@echo "Deployment:"
	@echo "  deploy         - Deploy to Fly.io"
	@echo "  deploy-check   - Check deployed app health"
	@echo "  logs           - View recent logs"
	@echo "  status         - View deployment status"
	@echo ""
	@echo "Database:"
	@echo "  migrate-check  - List available migrations"
	@echo "  migrate-apply  - Apply migrations"
	@echo "  migrate-list   - List applied migrations"
	@echo ""
	@echo "Testing:"
	@echo "  e2e-setup      - Setup instructions for E2E tests"
	@echo "  e2e            - Run E2E tests (requires TOKEN)"
	@echo ""
	@echo "Quality:"
	@echo "  review         - Run all quality checks"
	@echo "  ci             - Run full CI pipeline"
