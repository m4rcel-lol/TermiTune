# TermiTune Makefile

BINARY    := termitune
INSTALL   := /usr/local/bin
CARGO     := cargo

.PHONY: all build debug install uninstall clean check run

all: build

## Build release binary
build:
	@echo "  → Building TermiTune (release)..."
	@$(CARGO) build --release
	@echo "  ✓ Built: target/release/$(BINARY)"

## Build debug binary
debug:
	@$(CARGO) build
	@echo "  ✓ Debug: target/debug/$(BINARY)"

## Run (debug)
run: debug
	@./target/debug/$(BINARY)

## Run release
run-release: build
	@./target/release/$(BINARY)

## Install to /usr/local/bin
install: build
	@echo "  → Installing to $(INSTALL)..."
	@install -Dm755 target/release/$(BINARY) $(INSTALL)/$(BINARY)
	@mkdir -p $(HOME)/.config/termitune/themes
	@echo "  ✓ Installed $(INSTALL)/$(BINARY)"

## Uninstall
uninstall:
	@rm -f $(INSTALL)/$(BINARY)
	@echo "  ✓ Removed $(INSTALL)/$(BINARY)"

## Clean build artifacts
clean:
	@$(CARGO) clean
	@echo "  ✓ Cleaned"

## Check dependencies
check:
	@$(CARGO) check
	@echo "  ✓ No errors found"

## Run tests
test:
	@$(CARGO) test

## Lint
lint:
	@$(CARGO) clippy -- -D warnings

## Format
fmt:
	@$(CARGO) fmt

help:
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/^## /  /'
