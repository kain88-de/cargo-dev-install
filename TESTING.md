# Testing

This repo implements a Cargo subcommand plugin (`cargo dev-install`) that writes a wrapper script into a user-writable `bin` directory. The wrapper is intended to emulate Python editable installs: the command is stable on `PATH`, but execution happens from the working tree via `cargo run`.

## Unit tests

- Argument parsing and validation (`--bin`, `--force`, unknown flags)
- Crate root detection and binary enumeration
- Wrapper rendering (`REPO`, `--manifest-path`, arg forwarding, quoting)
- Install destination (`XDG_BIN_HOME` vs `$HOME/.local/bin`)
- PATH warning detection

## Integration tests

- Create temp crates (single + multi-bin)
- Set `HOME`, `PATH`, and `XDG_BIN_HOME`
- Assert wrapper file content, permissions, and overwrite behavior

## End-to-end smoke test

- Execute the installed wrapper
- Set `CARGO_TARGET_DIR` for isolation
- Assert output marker and forwarded args

## Notes

- Keep tests hermetic by setting `HOME`, `PATH`, and `CARGO_TARGET_DIR`.
- Linux only: on other platforms ensure the tool fails with a clear error message.
- Exit codes: `0` on success, `1` on any error.
