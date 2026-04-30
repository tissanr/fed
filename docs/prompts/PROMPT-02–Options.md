# PROMPT-02: Options Discovery

## Purpose

Extend the current `fed` implementation with a mode that lists applications installed on the local computer which can open a specific file.

The new command-line parameter is:

```text
fed -o <FILE>
fed --options <FILE>
```

The feature must be repeatable across implementation runs: preserve the existing file-opening behavior from `PROMPT-01-Init.md`, add the options-discovery behavior described here, and update the documentation and validation checks accordingly.

## Historical Context

This prompt builds on the implementation captured by `PROMPT-01-Init.md`.

The current `fed` behavior is:

- `fed <FILE>...` opens one or more existing files or directories with the operating system default application.
- Missing paths are reported to stderr.
- The process exits with status code `1` if any requested path fails.
- Opening is delegated to the `open` crate.

The new history step is:

1. Add `-o` / `--options` as an explicit discovery mode.
2. Require `-o` / `--options` to be called with exactly one file path.
3. When discovery mode is used, do not open the file.
4. Print the candidate programs that the operating system knows can open that file.

## CLI Contract

Support both modes:

```text
fed <FILE>...
fed -o <FILE>
fed --options <FILE>
```

Arguments and options:

- `<FILE>...`: one or more files or directories to open in the existing default mode.
- `-o <FILE>`, `--options <FILE>`: list programs that could open the provided file.

Rules:

- `-o` and `--options` are aliases for the same option.
- The option value must be a file path, represented as `PathBuf`.
- Discovery mode accepts exactly one path through the option.
- Discovery mode should not also accept positional files in the same invocation.
- The user must provide either positional files or `--options <FILE>`.
- If neither positional files nor `--options <FILE>` is provided, Clap should show a missing-argument error.
- If both `--options <FILE>` and positional files are provided, Clap should reject the invocation.

Recommended Clap structure:

- Keep `#[derive(Parser)]`.
- Change positional `files` from required to optional at the field level.
- Add `options: Option<PathBuf>` with `#[arg(short = 'o', long = "options", value_name = "FILE", conflicts_with = "files")]`.
- Use a Clap arg group or equivalent validation to require either `files` or `options`.

## Help Text

The generated help should make both modes clear.

Command metadata should become:

- Name: `fed`
- Short description: `Open files in their default application`
- Long description:

```text
Open one or more files using the OS default application, or list applications that can open a file.

Examples:
  fed photo.jpg
  fed report.pdf notes.txt
  fed .
  fed --options report.pdf
```

Argument help:

```text
One or more files (or directories) to open
```

Options help:

```text
List applications that can open FILE
```

## Runtime Behavior

Implement `main` as a branch between two modes:

1. Parse CLI.
2. If `cli.options` is present:
   - Validate that the provided path exists.
   - If it does not exist, print this exact stderr format and exit `1`:

     ```text
     fed: '<path>': no such file or directory
     ```

   - Discover programs that can open the file.
   - Print the discovered programs to stdout.
   - Exit `0` if discovery succeeds, even if the candidate list is empty.
   - Exit `1` if the platform discovery mechanism itself fails.
3. If `cli.options` is not present:
   - Preserve the existing default opening behavior from `PROMPT-01-Init.md`.

Discovery mode must not call `open::that`.

## Output Format

Use a stable, line-oriented stdout format so the output is scriptable:

```text
<program name>
<program name>
<program name>
```

Rules:

- Print one candidate program per line.
- Sort candidates alphabetically by display name.
- Remove duplicates by display name and executable or application identity when available.
- Print human-readable application names, not raw registry keys or desktop file IDs, when a display name is available.
- Do not print headings, bullets, numbering, or explanatory prose in normal successful output.
- If no candidates are found, print nothing and exit `0`.

Errors go to stderr using the existing `fed:` prefix.

## Discovery Semantics

The phrase "programs that could open this file" means applications registered with the operating system for the file's type or extension.

The implementation should prefer operating-system registration data over guessing from executable names. Do not scan every executable on `PATH` and infer capabilities manually.

Discovery can be platform-specific. Keep the common CLI and output formatting platform-neutral, and isolate platform discovery behind small helper functions.

Recommended internal model:

```rust
struct OpenOption {
    name: String,
    id: Option<String>,
    path: Option<PathBuf>,
}
```

Only `name` is required for printing. `id` and `path` are useful for deduplication and tests.

## macOS Strategy

On macOS, use Launch Services metadata.

Preferred implementation approach:

- Determine the file's Uniform Type Identifier from the file path or extension.
- Query Launch Services for applications that can open that content type.
- Include the default app and alternate registered apps.
- Return display names, bundle identifiers, and application paths where available.

Acceptable implementation routes:

- A small macOS-specific module using system commands and plist parsing.
- A Rust FFI wrapper around Launch Services if kept minimal and well-contained.

Do not hard-code common apps such as Preview, TextEdit, or Safari.

If the machine has no Launch Services result for the file type, return an empty list rather than guessing.

## Linux Strategy

On Linux, use the XDG MIME and desktop-entry ecosystem.

Preferred implementation approach:

1. Determine the MIME type for the file using `xdg-mime query filetype <FILE>`.
2. Read application `.desktop` files from:
   - `$XDG_DATA_HOME/applications`
   - each `$XDG_DATA_DIRS` entry followed by `/applications`
   - fallback defaults: `~/.local/share/applications` and `/usr/share/applications`
3. Select desktop entries whose `MimeType=` list contains the detected MIME type.
4. Use the localized or plain `Name=` field as the printed display name.
5. Include the desktop file path as identity for deduplication.

Also include the default application from:

```text
xdg-mime query default <mime-type>
```

Do not require a graphical session. If `xdg-mime` is missing or fails, return a clear stderr error and exit `1`.

## Windows Strategy

On Windows, use file association data from the registry.

Preferred implementation approach:

1. Determine the file extension.
2. Query registered file associations for that extension from the current user and local machine registry views.
3. Include:
   - the default ProgID handler for the extension
   - `OpenWithProgids`
   - `OpenWithList`
   - registered `Applications\*.exe` handlers when associated
4. Resolve friendly application names when possible from registry values or executable metadata.
5. Fall back to executable names when no friendly name is available.

Do not scan all installed programs from `Program Files`; use association data.

If the file has no extension and Windows cannot determine associations, return an empty list and exit `0`.

## Dependencies

Keep dependencies conservative and cross-platform.

Allowed additions if they simplify robust implementation:

- `dirs` or `directories` for user data directory discovery.
- `plist` for macOS plist parsing when using command output that returns plist data.
- `walkdir` for scanning application metadata directories.
- `winreg` for Windows registry access behind `cfg(windows)`.

Do not add a large GUI or desktop framework dependency for this feature.

## README Updates

Update the README usage block to include the new option:

```text
fed [OPTIONS] [FILE]...

Arguments:
  <FILE>...  One or more files (or directories) to open

Options:
  -o, --options <FILE>  List applications that can open FILE
  -h, --help            Print help
  -V, --version         Print version
```

Add examples:

```sh
# List applications that can open a PDF
fed --options report.pdf

# Short form
fed -o report.pdf
```

Clarify that `--options` lists registered applications and does not open the file.

## Tests

Add focused tests for CLI behavior that do not depend on machine-specific application registrations.

Recommended coverage:

- `fed --help` includes `-o, --options <FILE>`.
- `fed --options <missing-path>` exits `1` and prints the existing missing-path error format.
- `fed --options <file> <other-file>` is rejected by Clap.
- `fed` without arguments remains rejected by Clap.
- The discovery helper's sorting and deduplication logic is deterministic.

Avoid asserting that specific applications are installed. Platform discovery should be smoke-tested without depending on a particular desktop setup.

## Non-Goals

Do not add these features in this step:

- Opening a file with a chosen application.
- Interactive selection from the discovered list.
- JSON output.
- Machine-wide exhaustive executable scans.
- Network lookups.
- Application icons.
- Ranking beyond alphabetical output.
- Recursive directory handling.
- Shell completion generation.

## Acceptance Criteria

- Existing default behavior still works:
  - `fed <existing-path>` opens the path.
  - `fed <missing-path>` prints the existing missing-path error and exits `1`.
  - `fed <missing-path> <existing-path>` still attempts the existing path and exits `1`.
- New behavior works:
  - `fed --options <existing-file>` does not open the file.
  - `fed --options <existing-file>` prints zero or more registered candidate applications, one per line.
  - `fed -o <existing-file>` behaves the same as `fed --options <existing-file>`.
  - `fed --options <missing-file>` prints the existing missing-path error and exits `1`.
  - `fed --options <file> <positional-file>` is rejected.
- Quality checks pass:
  - `cargo fmt --all -- --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `cargo build --release`

## Repeatability Notes

The exact candidate applications are machine-specific and should not be treated as repeatable output. Repeatability applies to:

- the CLI surface
- validation behavior
- output format
- sorting and deduplication
- platform discovery strategy
- existing default file-opening behavior
- documentation updates

The implementation should produce equivalent behavior on different machines even though each machine may list different applications for the same file type.
