# https://github.com/aws-samples/serverless-rust-demo/blob/main/Makefile


FUNCTION_NAME ?= CostNotifier
ARCH          := aarch64-unknown-linux-gnu
BUILD_DIR     := target/lambda/$(FUNCTION_NAME)
ZIP_FILE      := $(BUILD_DIR)/function.zip
BINARY_NAME   := bootstrap
STACK_NAME    ?= aws-cost-notify
REGION        ?= ap-northeast-1

.PHONY: help setup build package deploy clean test check fmt clippy validate-env

help:
	@echo "Available targets:"
	@echo ""
	@echo "  make setup        Install required tools (cargo-lambda, zig, etc.)"
	@echo "  make build        Build Lambda binary for AWS (ARM64)"
	@echo "  make package      Create ZIP package"
	@echo "  make clean        Clean build artifacts"
	@echo "  make test         Run tests"
	@echo "  make check        Check build (cargo check)"
	@echo "  make clippy       Lint with clippy"
	@echo "  make fmt          Format code"


# -----------------------------------------------
# Setup
# -----------------------------------------------

setup:
ifeq (,$(shell which rustc))
	$(error "Could not found Rust compiler, please install it")
endif
ifeq (,$(shell which cargo))
	$(error "Could not found Cargo, please install it")
endif
ifeq (,$(shell which zig))
	$(error "Could not found Zig compiler, please install it")
endif
ifeq (,$(shell which cargo-lambda))
	@echo "Installing cargo-lambda..."
	cargo install cargo-lambda
endif
	@echo "Setup complete."

# -----------------------------------------------
# Build
# -----------------------------------------------

build: ## Build Lambda binary
	@echo "Building Lambda binary (cargo lambda build)..."
	cargo lambda build --release --target $(ARCH)

package: build ## Zip the Lambda binary
	@echo "Packaging..."
	mkdir -p $(BUILD_DIR)
	cd $(BUILD_DIR) && zip -j function.zip $(BINARY_NAME)
	@echo "Created: $(ZIP_FILE)"


# -----------------------------------------------
# Utility
# -----------------------------------------------

clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/lambda
	@echo "Clean complete."

test:
	cargo test

check:
	cargo check --release --target $(ARCH)

clippy:
	cargo clippy --release --target $(ARCH)

fmt:
	cargo fmt
