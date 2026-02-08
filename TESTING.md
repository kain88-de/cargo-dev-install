# Testing

This repo implements a Cargo subcommand plugin (`cargo dev-install`) that writes a small wrapper script into a user-writable `bin` directory (by default `$HOME/.local/bin`). The wrapper is intended to emulate Python editable installs: the command is stable on `PATH`, but execution happens from the working tree via `cargo run`.

The most valuable tests are:

## Unit Tests

Keep as much logic as possible in pure functions and test them without touching the filesystem.

Suggested targets:

- Argument parsing and validation (e.g. handling of `--help`, `--version`, unknown flags)
- Detecting the crate root / choosing the correct `Cargo.toml`
- Deriving the wrapper name (wrapper should have the same name as the application binary)
- Choosing a binary:
  - single-bin crate: no extra flags required
  - multi-bin crate: interactive selection when a TTY is available, otherwise require `--bin <name>`
- Rendering the wrapper contents:
  - hard-coded absolute `REPO=...`
  - `cargo run --manifest-path "$REPO/Cargo.toml" -- "$@"` (note the `--` and arg forwarding)
  - safe quoting (repo paths with spaces)
- Determining install destination (`$HOME/.local/bin` by default; and any override flags/env you add)
- Determining install destination with `XDG_BIN_HOME` preference when set (even though it is not an XDG standard)
- Determining whether the install dir is on `PATH` and producing the warning text/snippet

## Integration Tests

Use `assert_cmd`, `tempfile`, and `predicates` to run the built plugin against a throwaway Rust application in a temporary directory.

Key ideas:

- Create a temp Rust app (write a minimal `Cargo.toml` and `src/main.rs`). The app should print a known marker and echo args so you can assert forwarding.
- For multi-bin selection tests, generate a crate with multiple `[[bin]]` targets.
- Run the plugin binary (preferred) or `cargo run -p cargo-dev-install -- dev-install ...`.
- Set `HOME` to a temp directory so the install target becomes `tmp_home/.local/bin`.
- Set `XDG_BIN_HOME` in tests that should install elsewhere.
- Set `PATH` explicitly for each test so you can cover both cases:
  - install dir included: no warning
  - install dir excluded: warning printed with suggested export/snippet
- Assertions:
  - wrapper file exists at `$HOME/.local/bin/<appname>`
  - wrapper is executable (on Unix: mode includes an execute bit)
  - wrapper content contains the expected `REPO=` and `--manifest-path` line
  - existing wrapper without `--force`: warning and non-overwrite behavior; with `--force`: overwritten

## End-to-End Smoke Test

After the integration test installs the wrapper, execute the wrapper itself.

- Run `$HOME/.local/bin/<appname> -- somearg`.
- Set `CARGO_TARGET_DIR` to a temp directory to keep builds isolated and avoid cross-test interference.
- Assert the app output contains the marker and that `somearg` is visible, proving arg forwarding works.

## Edge Cases To Cover

- Multiple binaries in `Cargo.toml` / workspace members:
- Multiple binaries in `Cargo.toml` / workspace members:
  - interactive selection on TTY; `--bin <name>` required for non-interactive
- Workspace root vs member crate:
  - ensure the wrapper runs with the correct `--manifest-path`
- Repo paths with spaces:
  - wrapper quoting must remain correct
- Existing wrapper already present:
  - decide whether you overwrite, skip, or back up; test the behavior

## Notes

- Keep tests hermetic by setting `HOME`, `PATH`, and `CARGO_TARGET_DIR`.
- Linux only: on other platforms ensure the tool fails with a clear error message.
- Exit codes: `0` on success, `1` on any error.
