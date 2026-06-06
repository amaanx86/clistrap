# CLI_NAME - installed binary name (defaults to the name in Cargo.toml)
# BIN_DIR  - install destination

CARGO_BIN ?= $(shell grep -m1 '^name' Cargo.toml | sed 's/name\s*=\s*"\(.*\)"/\1/')
CLI_NAME  ?= $(CARGO_BIN)
BIN_DIR   ?= /usr/local/bin

RELEASE_BIN = target/release/$(CARGO_BIN)

# Optional: put your baked config vars in .build.env (gitignored)
# Example .build.env:
#   CLISTRAP_TENANT_ID = 7b64e511-...
#   CLISTRAP_CLIENT_ID = 76245417-...
#   CLISTRAP_DOMAIN    = acme.com
#   CLISTRAP_COMPANY   = Acme Corp
-include .build.env
export CLISTRAP_TENANT_ID
export CLISTRAP_CLIENT_ID
export CLISTRAP_DOMAIN
export CLISTRAP_COMPANY

.DEFAULT_GOAL := help

.PHONY: help build install reinstall uninstall check fmt clean

help:
	@echo "usage: make <target> [CLI_NAME=myapp]"
	@echo ""
	@echo "  build      compile release binary (bakes config from .build.env or env vars)"
	@echo "  install    build and install to $(BIN_DIR)/\$$CLI_NAME"
	@echo "  reinstall  build and replace existing binary in $(BIN_DIR)"
	@echo "  uninstall  remove $(BIN_DIR)/\$$CLI_NAME"
	@echo "  check      cargo check + fmt check"
	@echo "  fmt        cargo fmt"
	@echo "  clean      remove build artifacts"
	@echo ""
	@echo "  baked config vars: CLISTRAP_TENANT_ID, CLISTRAP_CLIENT_ID, CLISTRAP_DOMAIN, CLISTRAP_COMPANY"
	@echo "  example:   make install CLI_NAME=acme"

build:
	cargo build --release

install: build
	sudo install -m 755 $(RELEASE_BIN) $(BIN_DIR)/$(CLI_NAME)
	@echo "installed $(BIN_DIR)/$(CLI_NAME)"

reinstall: build
	sudo rm -f $(BIN_DIR)/$(CLI_NAME)
	sudo install -m 755 $(RELEASE_BIN) $(BIN_DIR)/$(CLI_NAME)
	@echo "replaced $(BIN_DIR)/$(CLI_NAME)"

uninstall:
	sudo rm -f $(BIN_DIR)/$(CLI_NAME)
	@echo "removed $(BIN_DIR)/$(CLI_NAME)"

check:
	cargo check
	cargo fmt --check

fmt:
	cargo fmt

clean:
	cargo clean
