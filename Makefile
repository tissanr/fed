BINARY      := fed
INSTALL_DIR := $(HOME)/.cargo/bin
MAN_DIR     ?= $(HOME)/.local/share/man/man1
MAN_PAGE    := docs/man/$(BINARY).1

.PHONY: all build debug release install uninstall clean check fmt

all: release

build: release

debug:
	cargo build

release:
	cargo build --release

install: release install-man
	mkdir -p $(INSTALL_DIR)
	install -m 755 target/release/$(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed $(BINARY) to $(INSTALL_DIR)/$(BINARY)"

install-man:
	mkdir -p $(MAN_DIR)
	install -m 644 $(MAN_PAGE) $(MAN_DIR)/$(BINARY).1
	@echo "Installed man page to $(MAN_DIR)/$(BINARY).1"

uninstall: uninstall-man
	rm -f $(INSTALL_DIR)/$(BINARY)
	@echo "Removed $(INSTALL_DIR)/$(BINARY)"

uninstall-man:
	rm -f $(MAN_DIR)/$(BINARY).1
	@echo "Removed man page from $(MAN_DIR)/$(BINARY).1"

clean:
	cargo clean

check:
	cargo clippy -- -D warnings

fmt:
	cargo fmt
