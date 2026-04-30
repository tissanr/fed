# fed

Open any file from the command line using its default application — on macOS, Linux, and Windows.

```
fed photo.jpg
fed report.pdf notes.txt
fed .
fed --options report.pdf
```

No configuration needed. `fed` delegates to the OS:

| Platform | Mechanism       |
|----------|-----------------|
| macOS    | `open`          |
| Linux    | `xdg-open`      |
| Windows  | `ShellExecute`  |

---

## Usage

```
fed [OPTIONS] [FILE]...

Arguments:
  <FILE>...  One or more files (or directories) to open

Options:
  -o, --options <FILE>  List applications that can open FILE
  -h, --help            Print help
  -V, --version         Print version
```

**Examples**

```sh
# Open a single file
fed invoice.pdf

# Open multiple files at once
fed index.html style.css

# Open the current directory in Finder / Explorer / Nautilus
fed .

# Open a specific directory
fed ~/Downloads

# List registered applications that can open a PDF without opening it
fed --options report.pdf

# Short form
fed -o report.pdf
```

`--options` lists registered applications known to the operating system and does not open the file.

---

## Roadmap

Development is tracked through prompt specifications in `docs/prompts/`.

| Prompt | Focus | Prompt status | Implementation status |
|--------|-------|---------------|-----------------------|
| `PROMPT-01-Init.md` | Initial `fed <FILE>...` CLI that opens files and directories with the OS default application | Delivered | Implemented |
| `PROMPT-02–Options.md` | Add `-o, --options <FILE>` to list applications that can open a file | Delivered | Implemented |
| `PROMPT-03-Man-Pages.md` | Add a Unix man page and user-local man page install targets | Delivered | Planned |
| `PROMPT-04-Help.md` | Improve `-h, --help` output with complete usage, examples, and tests | Delivered | Planned |

Planned items describe the intended direction and acceptance criteria, but are not part of the current runtime behavior until implemented.

---

## Installation

### macOS / Linux — Makefile

```sh
# Clone and build
git clone https://github.com/tissanr/fed.git
cd fed

# Debug build (fast)
make

# Release build + install to /usr/local/bin
make install

# Uninstall
make uninstall
```

#### Makefile targets

| Target      | Description                                 |
|-------------|---------------------------------------------|
| `build`     | Debug build (`cargo build`)                 |
| `release`   | Optimised release build                     |
| `install`   | Release build → `~/.cargo/bin/fed`          |
| `uninstall` | Remove `~/.cargo/bin/fed`                   |
| `clean`     | Remove build artifacts                      |
| `check`     | Run Clippy lints                            |
| `fmt`       | Format source with `rustfmt`                |

### Windows — PowerShell

```powershell
# From the project root (PowerShell):
.\install.ps1

# Install to a custom directory:
.\install.ps1 -InstallDir "C:\Tools"
```

The script will:

1. Verify `cargo` is on `PATH`
2. Run `cargo build --release`
3. Copy `fed.exe` to the install directory (`~/.cargo/bin` by default)
4. Offer to add the directory to your user `PATH` if it isn't there already

---

## Building from source

Requires [Rust](https://rustup.rs) (stable, 1.70+).

```sh
cargo build --release
# binary is at: target/release/fed  (or fed.exe on Windows)
```

---

## License

MIT
