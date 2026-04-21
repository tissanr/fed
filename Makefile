BINARY     := fed
INSTALL_DIR := /usr/local/bin

.PHONY: all build release install uninstall clean check fmt

all: build

build:
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
