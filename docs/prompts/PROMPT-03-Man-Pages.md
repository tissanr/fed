# PROMPT-03: Man Pages

## Purpose

Extend the `fed` project with a Unix manual page so users can read local command documentation with:

```text
man fed
```

This prompt builds on the implementation history captured in:

- `docs/prompts/PROMPT-01-Init.md`
- `docs/prompts/PROMPT-02–Options.md`

The goal is repeatability: a later implementation run should add the same documentation artifact, install behavior, README updates, and validation checks without changing the runtime semantics of `fed`.

## Historical Context

The current project history is:

1. `PROMPT-01-Init.md`: create the base Rust CLI that opens files with their default application.
2. `PROMPT-02–Options.md`: add `-o` / `--options <FILE>` to list registered applications that can open a file.
3. This prompt: add a Unix man page for the implemented CLI.

The man-page work is documentation and packaging support only. It must not change the behavior of opening files or listing options.

## Repository Shape

Add this tracked file:

```text
docs/man/fed.1
```

Keep the existing prompt file at:

```text
docs/prompts/PROMPT-03-Man-Pages.md
```

Update these existing files:

```text
Makefile
README.md
```

Do not commit generated build artifacts under `target/`.

## Manual Page Format

Create `docs/man/fed.1` as a section 1 man page using portable roff/man macros.

Use plain ASCII roff source. Do not generate HTML, Markdown-only documentation, or compressed `.gz` output as the tracked source file.

Required top-level sections:

```text
.TH FED 1
.SH NAME
.SH SYNOPSIS
.SH DESCRIPTION
.SH OPTIONS
.SH EXAMPLES
.SH EXIT STATUS
.SH FILES
.SH SEE ALSO
.SH AUTHOR
.SH LICENSE
```

The exact `.TH` line may include a date and version, but it should identify:

- command: `FED`
- section: `1`
- version: `fed 0.1.0`

## Man Page Content

The man page must document the CLI after `PROMPT-02–Options.md`.

### NAME

Use:

```text
fed \- open files with their default application
```

### SYNOPSIS

Document both modes:

```text
fed [OPTIONS] FILE...
fed -o FILE
fed --options FILE
```

### DESCRIPTION

Explain that `fed` opens one or more files or directories with the operating system's default application.

Also explain that `--options` lists applications registered on the current computer that can open the given file, and does not open the file.

### OPTIONS

Document:

- `-o, --options FILE`: list applications that can open `FILE`.
- `-h, --help`: print help.
- `-V, --version`: print version.

### EXAMPLES

Include examples for:

```text
fed photo.jpg
fed report.pdf notes.txt
fed .
fed --options report.pdf
fed -o report.pdf
```

### EXIT STATUS

Document:

- `0`: all requested work completed successfully.
- `1`: at least one file was missing, opening failed, or options discovery failed.

Also mention that default open mode is best-effort: when multiple paths are provided, later paths are still attempted after earlier failures.

### FILES

Mention:

- `~/.cargo/bin/fed`: default install location used by the Makefile and Windows installer documentation where applicable.
- `docs/man/fed.1`: source man page in the repository.

### SEE ALSO

Mention relevant platform mechanisms:

```text
open(1), xdg-open(1), xdg-mime(1)
```

Avoid claiming these commands are available on every platform.

### AUTHOR and LICENSE

Use the existing package/project authorship style. If no explicit author is configured, use a neutral project-level author such as:

```text
The fed contributors
```

License must be MIT.

## Makefile Updates

Add install and uninstall support for the man page on Unix-like systems.

Recommended variables:

```make
MAN_DIR ?= $(HOME)/.local/share/man/man1
MAN_PAGE := docs/man/$(BINARY).1
```

Update `.PHONY` to include:

```text
install-man uninstall-man
```

Add targets:

```make
install-man:
	mkdir -p $(MAN_DIR)
	install -m 644 $(MAN_PAGE) $(MAN_DIR)/$(BINARY).1
	@echo "Installed man page to $(MAN_DIR)/$(BINARY).1"

uninstall-man:
	rm -f $(MAN_DIR)/$(BINARY).1
	@echo "Removed man page from $(MAN_DIR)/$(BINARY).1"
```

Update `install` so it installs both:

- the release binary
- the man page

Recommended dependency:

```make
install: release install-man
```

Update `uninstall` so it removes both:

- the binary
- the man page

Recommended dependency:

```make
uninstall: uninstall-man
```

Keep the binary install location from `PROMPT-01-Init.md`:

```make
INSTALL_DIR := $(HOME)/.cargo/bin
```

Do not require `sudo`. The default man page install location should be user-local.

## README Updates

Update the README installation section to mention that `make install` also installs the man page to:

```text
~/.local/share/man/man1/fed.1
```

Add a short man page section:

```markdown
## Manual Page

After `make install`, read the manual with:

```sh
man fed
```

If your system does not include `~/.local/share/man` in `MANPATH`, either add it to `MANPATH` or view the page directly:

```sh
man ./docs/man/fed.1
```
```

Update the Makefile target table to include:

- `install-man`: install the man page only.
- `uninstall-man`: remove the installed man page only.

## Validation

Add validation instructions to the project documentation or Makefile only if the existing style supports it. At minimum, document these checks for maintainers:

```sh
mandoc -Tlint docs/man/fed.1
man ./docs/man/fed.1
```

`mandoc` may not be installed everywhere, so absence of `mandoc` should not make the normal Rust build fail.

Do not add a mandatory CI dependency on `mandoc` unless the CI workflow also installs it explicitly.

## Non-Goals

Do not add these features in this step:

- Runtime `--man` or `man` subcommands.
- Shell completions.
- HTML documentation generation.
- Compressed tracked man page source.
- System-wide installs requiring `sudo`.
- Windows help integration.
- Changes to file-opening behavior.
- Changes to `--options` discovery behavior.

## Acceptance Criteria

- `docs/man/fed.1` exists and is valid roff/man source.
- `man ./docs/man/fed.1` renders a readable manual page on systems with `man`.
- `make install-man` installs `fed.1` to `~/.local/share/man/man1/fed.1` by default.
- `make uninstall-man` removes the installed user-local man page.
- `make install` still builds and installs the binary to `~/.cargo/bin/fed`, and also installs the man page.
- `make uninstall` removes both the binary and man page.
- README documents the man page and updated Makefile targets.
- Rust quality checks still pass:
  - `cargo fmt --all -- --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `cargo build --release`

## Repeatability Notes

The man page should describe the implemented CLI, not a future design.

If the implementation from `PROMPT-02–Options.md` has not yet been applied, the man page should either:

- be created after that implementation, or
- clearly align with the intended post-prompt-2 CLI surface.

Keep documentation language concise and factual. Avoid duplicating the full README; the man page should be a local command reference.
