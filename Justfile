# Converge Application - Common Tasks
# Run `just --list` to see all available recipes

# Default recipe: show help
default:
    @just --list

# ============================================================================
# BUILD & TEST
# ============================================================================

# Build the project
build:
    cargo build

# Build in release mode
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run clippy lints
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

# ============================================================================
# EVALS
# ============================================================================

# Run all eval fixtures with mock LLM (fast, deterministic)
eval:
    cargo run -- eval run --mock

# Run a specific eval
eval-one ID:
    cargo run -- eval run {{ID}} --mock

# List available eval fixtures
eval-list:
    cargo run -- eval list

# Run evals with real LLM (integration test)
eval-real:
    cargo run -- eval run

# ============================================================================
# RUN COMMANDS
# ============================================================================

# Run a job with mock LLM
run-mock TEMPLATE="growth-strategy":
    cargo run -- run --template {{TEMPLATE}} --mock

# Run a job with streaming output
run-stream TEMPLATE="growth-strategy":
    cargo run -- run --template {{TEMPLATE}} --mock --stream

# Run a job with JSON streaming
run-stream-json TEMPLATE="growth-strategy":
    cargo run -- run --template {{TEMPLATE}} --mock --stream --json

# Run a job with JSON output
run-json TEMPLATE="growth-strategy":
    cargo run -- run --template {{TEMPLATE}} --mock --json

# Run a job in quiet mode (exit code only)
run-quiet TEMPLATE="growth-strategy":
    cargo run -- run --template {{TEMPLATE}} --mock --quiet

# Run a job with real LLM
run-real TEMPLATE="growth-strategy":
    cargo run -- run --template {{TEMPLATE}}

# Run with custom seeds from file
run-seeds TEMPLATE FILE:
    cargo run -- run --template {{TEMPLATE}} --seeds @{{FILE}} --mock

# ============================================================================
# PACKS
# ============================================================================

# List available domain packs
packs:
    cargo run -- packs list

# Show info about a specific pack
pack-info NAME="growth-strategy":
    cargo run -- packs info {{NAME}}

# ============================================================================
# TUI
# ============================================================================

# Launch interactive TUI
tui:
    cargo run -- tui

# ============================================================================
# DEMO
# ============================================================================

# Run a demo job with sample seeds
demo:
    cargo run -- run --template growth-strategy \
        --seeds '[{"id": "company", "content": "B2B SaaS startup in Nordic market"}, {"id": "goal", "content": "Expand into enterprise segment"}]' \
        --mock --stream

# Run demo with JSON output
demo-json:
    cargo run -- run --template growth-strategy \
        --seeds '[{"id": "company", "content": "B2B SaaS startup"}, {"id": "goal", "content": "Grow revenue"}]' \
        --mock --json

# ============================================================================
# DEVELOPMENT
# ============================================================================

# Watch for changes and rebuild
watch:
    cargo watch -x build

# Watch and run tests on change
watch-test:
    cargo watch -x test

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree

# Generate documentation
doc:
    cargo doc --open

# ============================================================================
# CI
# ============================================================================

# Run CI checks (what CI would run)
ci: fmt-check lint test eval
    @echo "All CI checks passed!"
