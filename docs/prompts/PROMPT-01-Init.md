# PROMPT-01: Initial Implementation Specification

## Purpose

Use this prompt to recreate the current `fed` implementation with high repeatability. The goal is not only to describe the feature, but to capture enough product intent, repository shape, implementation details, and project history that a new run can produce a result equivalent to the existing codebase.

`fed` is a small cross-platform command-line utility that opens one or more files or directories with the operating system's default application.

## Historical Context

The current implementation evolved through these steps:

1. Start with a Rust CLI named `fed`.
2. Implement the core behavior: accept one or more paths, check that each path exists, and open each existing path with the OS default application.
3. Add README, MIT license, Makefile, Windows PowerShell installer, `Cargo.toml`, `Cargo.lock`, and `src/main.rs`.
4. Remove accidentally committed Cargo build artifacts under `target/` and update `.gitignore` so build outputs remain untracked.
5. Add a basic GitHub Actions Rust workflow.
6. Change the Unix install target from a system install path to the user-local Cargo binary path, `~/.cargo/bin`, avoiding sudo.
7. Add a more complete CI workflow with format, Clippy, and multi-platform build/test jobs.

The reproduced repository should reflect the final state after those steps, not the intermediate mistakes. In particular, do not commit `target/`.

## Repository Shape

Create or preserve this tracked file set:

```text
.gitignore
.github/workflows/ci.yml
.github/workflows/rust.yml
Cargo.lock
Cargo.toml
LICENSE
Makefile
README.md
install.ps1
src/main.rs
```

This prompt file itself belongs at:

```text
docs/prompts/PROMPT-01-Init.md
```

`target/` may exist locally after builds, but it must be ignored and should not be part of the recreated source state.

## Rust Package

Create a Rust binary crate with:

- Package name: `fed`
- Version: `0.1.0`
- Edition: `2021`
- Description: `Open files in their default application from the command line`
- License: `MIT`
- Binary target name: `fed`
- Binary target path: `src/main.rs`

Use these dependencies:

```toml
[dependencies]
open = "5"
clap = { version = "4", features = ["derive"] }
```

Use this release profile:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

Check in `Cargo.lock` because this is an application binary.

## CLI Contract

The command is:

```text
fed <FILE>...
```

Arguments:

- `<FILE>...`: one or more file or directory paths to open.
- The argument is required.
- Multiple paths are accepted in one invocation.
- Clap should display the argument value name as `FILE`.

Generated options:

- `-h`, `--help`: print help.
- `-V`, `--version`: print version.

CLI metadata:

- Command name: `fed`
- Short description: `Open files in their default application`
- Long description:

```text
Open one or more files using the OS default application.

Examples:
  fed photo.jpg
  fed report.pdf notes.txt
  fed .
```

## Core Implementation

Implement `src/main.rs` with this structure:

- Import `clap::Parser`.
- Import `std::path::PathBuf`.
- Import `std::process`.
- Define a `Cli` struct deriving `Parser`.
- Store requested paths in `files: Vec<PathBuf>`.
- Mark `files` as required with `#[arg(required = true, value_name = "FILE")]`.
- In `main`, parse the CLI, process every path in order, and exit with status code `1` if any error occurred.

Runtime behavior:

1. Parse command-line arguments into a list of `PathBuf` values.
2. Initialize `had_error` to `false`.
3. Iterate over every requested path in input order.
4. For each path:
   - If `file.exists()` is false, print this exact stderr format:

     ```text
     fed: '<path>': no such file or directory
     ```

     Set `had_error = true` and continue to the next path.

   - If the path exists, call `open::that(file)`.
   - If `open::that(file)` returns an error, print this exact stderr format:

     ```text
     fed: could not open '<path>': <error>
     ```

     Set `had_error = true` and continue to the next path.

5. After all paths have been processed, call `process::exit(1)` only when `had_error` is true.

This is a best-effort command: a failure for one path must not prevent later paths from being processed.

## Path Handling

- Accept paths exactly as provided by the shell.
- Represent paths as `PathBuf`.
- Files and directories are both valid inputs.
- `.` is valid when it exists.
- Missing paths must not be passed to `open::that`.
- Do not canonicalize paths.
- Do not implement custom tilde expansion.
- Do not implement glob expansion beyond shell behavior.

## Platform Behavior

Use the `open` crate for platform delegation. Do not implement custom platform-specific opener commands in this crate.

User-facing behavior should be described as:

| Platform | Mechanism |
| --- | --- |
| macOS | `open` |
| Linux | `xdg-open` |
| Windows | `ShellExecute` |

## Makefile

Provide a Makefile for Unix-like development with:

- `BINARY := fed`
- `INSTALL_DIR := $(HOME)/.cargo/bin`
- `.PHONY: all build release install uninstall clean check fmt`
- `all`: depends on `build`
- `build`: runs `cargo build`
- `release`: runs `cargo build --release`
- `install`: depends on `release`, installs `target/release/fed` to `$(INSTALL_DIR)/fed` with mode `755`, and prints the installed path
- `uninstall`: removes `$(INSTALL_DIR)/fed` and prints the removed path
- `clean`: runs `cargo clean`
- `check`: runs `cargo clippy -- -D warnings`
- `fmt`: runs `cargo fmt`

The install location must be `~/.cargo/bin`, not `/usr/local/bin`.

## Windows Installer

Provide `install.ps1` with this behavior:

- Default install directory: `$env:USERPROFILE\.cargo\bin`
- Optional parameter: `-InstallDir`
- Set `$ErrorActionPreference = "Stop"`.
- Check that `cargo` exists on `PATH`; if not, print an error pointing to `https://rustup.rs` and exit `1`.
- Build with `cargo build --release`.
- Stop with an error if the build fails.
- Create the install directory if it does not exist.
- Copy `target\release\fed.exe` to the install directory.
- Check the user `Path` environment variable.
- If the install directory is missing from `Path`, ask whether to add it.
- If the user confirms with `y` or `Y`, append the directory to the user `Path`.
- Print a final success message suggesting `fed yourfile.pdf`.

## README

Write a concise README that includes:

- Project title: `fed`
- One-sentence purpose: open any file from the command line using its default application on macOS, Linux, and Windows.
- Examples:

  ```text
  fed photo.jpg
  fed report.pdf notes.txt
  fed .
  ```

- A platform mechanism table for macOS, Linux, and Windows.
- Usage block:

  ```text
  fed [FILE]...

  Arguments:
    <FILE>...  One or more files (or directories) to open

  Options:
    -h, --help     Print help
    -V, --version  Print version
  ```

- Example commands for opening a single file, multiple files, the current directory, and `~/Downloads`.
- macOS/Linux installation through `make`, `make install`, and `make uninstall`.
- Makefile target table.
- Windows installation through `.\install.ps1` and `.\install.ps1 -InstallDir "C:\Tools"`.
- Source build instructions requiring stable Rust 1.70+.
- MIT license.

## Ignore Rules

Use `.gitignore` to keep generated and local files out of the repository:

```gitignore
# Cargo build output
/target

# rustfmt backup files
**/*.rs.bk

# MSVC debug info
*.pdb

# cargo-mutants
**/mutants.out*/

# IDEs
.idea/
.vscode/
*.iml

# macOS
.DS_Store

# Windows
Thumbs.db
```

## CI Workflows

Keep both current workflow files to match the implemented repository history.

`.github/workflows/rust.yml`:

- Name: `Rust`
- Triggers: push and pull request to `main`
- Environment: `CARGO_TERM_COLOR: always`
- One Ubuntu job that runs:
  - `cargo build --verbose`
  - `cargo test --verbose`

`.github/workflows/ci.yml`:

- Name: `CI`
- Triggers: push and pull request to `main`
- Environment:
  - `CARGO_TERM_COLOR: always`
  - `RUST_BACKTRACE: 1`
- Jobs:
  - `fmt` on `ubuntu-latest`, using `actions/checkout@v4`, `dtolnay/rust-toolchain@stable` with `rustfmt`, and `cargo fmt --all -- --check`.
  - `clippy` on `ubuntu-latest`, using `actions/checkout@v4`, `dtolnay/rust-toolchain@stable` with `clippy`, `Swatinem/rust-cache@v2`, and `cargo clippy -- -D warnings`.
  - `test` on `ubuntu-latest`, `macos-latest`, and `windows-latest`, with `fail-fast: false`, using `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` keyed by OS, `cargo build --release`, `cargo test --release`, and a Bash smoke test that writes `/tmp/smoke.txt` and runs `./target/release/fed /tmp/smoke.txt || true`.

The smoke test intentionally ignores the opener exit result because CI environments may not have a usable desktop opener.

## License

Use the MIT license in `LICENSE`.

## Non-Goals

Do not add these features in the initial implementation:

- Glob expansion beyond what the user's shell already provides.
- Recursive directory opening.
- Configuration files.
- Custom application selection.
- Dry-run mode.
- Interactive CLI prompts in `fed` itself.
- Async or parallel opening.
- Structured logging.
- Custom platform-specific open commands in this crate.
- Unit tests that mock `open::that`; the current implementation has no tests.
- Extra subcommands.

## Repeatability Checks

A recreated implementation is close enough when these checks pass:

- `cargo fmt --all -- --check`
- `cargo clippy -- -D warnings`
- `cargo build`
- `cargo test`
- `cargo build --release`
- `git status --short` does not show generated `target/` files as tracked or staged.
- `fed --help` shows the command name, required `FILE` argument, and examples from the long description.
- Running `fed` without arguments shows Clap's missing-argument error.
- Running `fed <missing-path>` prints the documented missing-path error and exits with status code `1`.
- Running `fed <missing-path> <existing-path>` still attempts to open the existing path and exits with status code `1`.

Do not require exact byte-for-byte identity for generated files that depend on the current Cargo resolver output, but the source files, behavior, install paths, CI shape, and documented history should match this specification.
