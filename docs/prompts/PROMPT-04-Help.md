# PROMPT-04: Help Output

## Purpose

Improve the built-in command help so users get a useful overview from:

```text
fed -h
fed --help
```

The help output must describe every supported way to use `fed`, including the default file-opening mode, the `--options` discovery mode from `PROMPT-02–Options.md`, and the manual page support from `PROMPT-03-Man-Pages.md`.

This is a CLI documentation and usability improvement. It must not change the runtime behavior of opening files, listing application options, or installing man pages.

## Historical Context

This prompt builds on:

1. `PROMPT-01-Init.md`: base `fed <FILE>...` command.
2. `PROMPT-02–Options.md`: `-o` / `--options <FILE>` application discovery mode.
3. `PROMPT-03-Man-Pages.md`: Unix man page and install support.
4. This prompt: make `-h` / `--help` intentionally comprehensive and test-covered.

## CLI Contract

The help flags are:

```text
fed -h
fed --help
```

Rules:

- Both flags must print help to stdout.
- Both flags must exit with status code `0`.
- Help output must not attempt to open files.
- Help output must not run application-discovery logic.
- Help output must not require a file path.
- Help output must include all supported arguments and options.

## Help Content

Use Clap-generated help, but configure it deliberately instead of relying only on defaults.

The help should communicate:

- What `fed` does.
- How to open one file.
- How to open multiple files.
- How to open a directory.
- How to list applications that can open a file.
- That `--options` does not open the file.
- That more complete local documentation may be available through `man fed` after installation.

## Recommended Clap Metadata

Command name:

```text
fed
```

Short description:

```text
Open files in their default application
```

Long description:

```text
Open one or more files or directories using the operating system's default application.

Use --options to list applications registered on this computer that can open a file without opening it.
```

After-help text:

```text
Examples:
  fed photo.jpg
  fed report.pdf notes.txt
  fed .
  fed --options report.pdf
  fed -o report.pdf

Manual:
  man fed
```

If `man fed` is only available after installation, keep the wording concise and factual:

```text
Manual:
  man fed    available after installing the man page
```

## Usage Shape

The generated help should show a usage line equivalent to:

```text
Usage: fed [OPTIONS] [FILE]...
```

If the implementation uses Clap argument groups, make sure the help still clearly communicates that the user must provide either:

- one or more positional `FILE` values, or
- `--options <FILE>`

The error output for missing arguments should remain handled by Clap and should stay useful.

## Arguments Section

The positional file argument should be documented as:

```text
Arguments:
  [FILE]...  One or more files or directories to open
```

Use `FILE` as the value name.

Do not use vague labels such as `PATH` in one place and `FILE` in another. Keep help, README, and man page naming consistent.

## Options Section

The help output must include:

```text
Options:
  -o, --options <FILE>  List applications that can open FILE without opening it
  -h, --help            Print help
  -V, --version         Print version
```

The exact spacing may be Clap-dependent, but the wording and option coverage should be equivalent.

## Error Help

When a user runs:

```text
fed
```

Clap should reject the invocation because neither positional files nor `--options <FILE>` was provided.

The error should point users toward `--help`.

When a user runs:

```text
fed --options report.pdf other.txt
```

Clap should reject the invocation because discovery mode conflicts with positional files.

Do not implement custom ad hoc usage text for these cases unless Clap cannot express the validation cleanly.

## README Updates

Update the README usage block so it matches the help output:

```text
fed [OPTIONS] [FILE]...

Arguments:
  [FILE]...  One or more files or directories to open

Options:
  -o, --options <FILE>  List applications that can open FILE without opening it
  -h, --help            Print help
  -V, --version         Print version
```

Add or preserve examples:

```sh
fed photo.jpg
fed report.pdf notes.txt
fed .
fed --options report.pdf
fed -o report.pdf
fed --help
```

If a manual page exists from `PROMPT-03-Man-Pages.md`, mention:

```sh
man fed
```

## Man Page Alignment

If `docs/man/fed.1` exists, make sure its SYNOPSIS and OPTIONS sections agree with the CLI help:

- `fed [OPTIONS] [FILE]...`
- `fed -o FILE`
- `fed --options FILE`
- `-o, --options FILE`
- `-h, --help`
- `-V, --version`

The man page can contain more detail than `--help`, but it must not contradict the help output.

## Tests

Add tests for the help surface.

Recommended approach:

- Use `assert_cmd` and `predicates` as dev-dependencies, or another small established CLI test approach.
- Keep tests independent of machine-specific desktop application registrations.

Required coverage:

- `fed --help` exits `0`.
- `fed -h` exits `0`.
- `fed --help` includes `Usage:`.
- `fed --help` includes `fed [OPTIONS] [FILE]...` or equivalent Clap-generated usage.
- `fed --help` includes `-o, --options <FILE>`.
- `fed --help` includes `-h, --help`.
- `fed --help` includes `-V, --version`.
- `fed --help` includes examples for opening a file, opening multiple files, opening `.`, and using `--options`.
- `fed --help` includes `man fed` if prompt 3 has been implemented.
- `fed` with no arguments fails and points to help.
- `fed --options <file> <other-file>` fails before application discovery runs.

Do not assert exact full help output byte-for-byte. Clap may adjust spacing between versions. Assert important substrings.

## Implementation Notes

Prefer keeping help definition near the `Cli` struct in `src/main.rs`, using Clap attributes such as:

```rust
#[command(
    name = "fed",
    about = "...",
    long_about = "...",
    after_help = "..."
)]
```

If the help strings become too large for readability, use constants in `src/main.rs`.

Do not introduce a custom hand-rolled help printer unless Clap cannot produce the required output.

## Non-Goals

Do not add these features in this step:

- Interactive help.
- Shell completions.
- `help` subcommand.
- `--manual`, `--man`, or `man` subcommands.
- JSON help output.
- Localization.
- Terminal color styling.
- Changes to default file-opening behavior.
- Changes to `--options` discovery behavior.
- Changes to man page install locations.

## Acceptance Criteria

- `fed -h` prints useful help and exits `0`.
- `fed --help` prints useful help and exits `0`.
- Help output documents every current user-facing option and argument.
- Help output includes examples for normal open mode and `--options` mode.
- Help output references `man fed` when man page support exists.
- README usage and examples match the help surface.
- `docs/man/fed.1`, if present, does not contradict the help output.
- Missing-argument and conflict errors remain handled by Clap.
- Quality checks pass:
  - `cargo fmt --all -- --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `cargo build --release`

## Repeatability Notes

The exact whitespace in Clap-generated help may vary across Clap versions. Repeatability should be judged by:

- option coverage
- argument coverage
- examples
- exit behavior
- README and man page alignment
- tests asserting important substrings rather than full output snapshots
