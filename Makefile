BINARY      := fed
INSTALL_DIR := $(HOME)/.cargo/bin

.PHONY: all build debug release install uninstall clean check fmt

all: release

build: release

debug:
	cargo build

release:
	cargo build --release

install: release
	install -m 755 target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed $(BINARY) to $(INSTALL_DIR)/$(BINARY)"

uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY)
	@echo "Removed $(INSTALL_DIR)/$(BINARY)"

clean:
	cargo clean

check:
	cargo clippy -- -D warnings

fmt:
	cargo fmt
