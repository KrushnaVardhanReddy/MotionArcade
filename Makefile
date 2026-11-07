# ==============================================================================
# MotionArcade — Developer & Automation Makefile
# ==============================================================================

.PHONY: all build run test check clippy fmt fetch-model clean help run-mock run-banana run-math

# Default target
all: check test

# ------------------------------------------------------------------------------
# Running & Playing
# ------------------------------------------------------------------------------

## run: Launch MotionArcade with default live camera and hands-free portal menu
run: fetch-model
	cargo run --release -p app

## run-mock: Launch without camera using deterministic recorded pose playback
run-mock:
	cargo run --release -p app -- --mock-input assets/recordings/sample_wave.jsonl

## run-banana: Launch directly into Jungle Jump (Banana Grab animal game)
run-banana:
	cargo run --release -p app -- --game banana

## run-math: Launch directly into Deflecto-Bot with Early Math addition facts
run-math:
	cargo run --release -p app -- --game deflecto --pack content/bundled/math/number_facts_k2.toml

## run-phonics: Launch directly into Deflecto-Bot with Literacy Letter Sounds
run-phonics:
	cargo run --release -p app -- --game deflecto --pack content/bundled/literacy/letter_sounds_k1.toml

## run-multiplication: Launch directly into Deflecto-Bot with Multiplication tables
run-multiplication:
	cargo run --release -p app -- --game deflecto --pack content/bundled/math/multiplication_basics.toml

# ------------------------------------------------------------------------------
# Models & Assets
# ------------------------------------------------------------------------------

## fetch-model: Download the official MoveNet-Lightning float32 ONNX model (~7MB)
fetch-model:
	@chmod +x scripts/fetch_models.sh
	./scripts/fetch_models.sh

# ------------------------------------------------------------------------------
# Build & Quality Assurance Gates
# ------------------------------------------------------------------------------

## build: Build the complete workspace in release mode
build:
	cargo build --workspace --release

## test: Run all 102+ workspace unit and integration tests
test:
	cargo test --workspace

## check: Fast compilation check across all workspace crates
check:
	cargo check --workspace

## clippy: Run strict linter asserting zero warnings (-D warnings)
clippy:
	cargo clippy --workspace --all-targets -- -D warnings

## fmt: Check formatting against standard rustfmt
fmt:
	cargo fmt --check

## fmt-fix: Automatically format all Rust source files
fmt-fix:
	cargo fmt

## verify: Run the complete pre-commit CI verification gate
verify: fmt clippy test
	@echo "All MotionArcade quality gates passed cleanly! (0 panics, 0 warnings)"

# ------------------------------------------------------------------------------
# Cleanup
# ------------------------------------------------------------------------------

## clean: Remove all cargo build artifacts and temporary files
clean:
	cargo clean

# ------------------------------------------------------------------------------
# Help
# ------------------------------------------------------------------------------

## help: Display available make commands
help:
	@echo "MotionArcade Command Line Interface:"
	@echo ""
	@sed -n 's/^##//p' $(MAKEFILE_LIST) | column -t -s ':'
