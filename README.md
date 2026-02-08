# Cargo dev install

This is a Cargo subcommand plugin that provides a "dev install" flow for Rust applications.
The goal is to emulate Python editable installs (e.g. `pip install -e`): you get a stable command on your `PATH`, but execution happens from your working tree via `cargo run`.

Linux only.

To install your app run.

```bash
cargo dev-install
```

Cargo discovers subcommands by looking for an executable named `cargo-dev-install` on your `PATH`. When present, `cargo dev-install` runs that executable.

Exit codes: `0` on success, `1` on any error.

# Details

The plugin installs a small wrapper script into your `PATH`.
The wrapper has the same name as the application binary, so you can run it like a normal installed command while still executing from source.

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO="path/to/app"
exec cargo  run --manifest-path "$REPO/Cargo.toml" -- "$@"
```

## Install location

The install target directory is chosen with good defaults:

- If `XDG_BIN_HOME` is set, use that (not an XDG standard, but commonly used).
- Otherwise use `$HOME/.local/bin`.

If the chosen directory is not on your `PATH`, the plugin warns during installation and prints the snippet you need to add to your shell profile.

## Choosing the wrapper name

- If the crate defines exactly one binary, the wrapper name is that binary name.
- If multiple binaries exist, the tool either prompts you to select one (interactive TTY) or you must pass `--bin <name>` for non-interactive usage.

To keep the tool minimal, configuration is intentionally limited to the cases that need it.

## Overwrite behavior

If a wrapper with the chosen name already exists in the install directory, the tool does not overwrite it by default and prints a warning.
Re-run with `--force` to overwrite.

## Repo path

`REPO` is set to the crate root directory (where the selected `Cargo.toml` lives) using an absolute path.
Symlinks are not resolved.
