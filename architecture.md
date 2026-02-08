# Architecture

This repository implements a Cargo subcommand plugin that provides a "dev install" flow for Rust applications (Linux only). It creates a wrapper script named after the selected application binary and installs it into a user-writable bin directory (prefer `XDG_BIN_HOME` if set, else `$HOME/.local/bin`). The wrapper runs the app from the working tree via `cargo run`, emulating Python editable installs.

Exit codes are simple: `0` on success, `1` on any error.

## Wrapper Script Contract

The installed wrapper is a POSIX-ish bash script with a fixed repo root:

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="/absolute/path/to/crate-root"
exec cargo run --manifest-path "$REPO/Cargo.toml" -- "$@"
```

Requirements:

- Wrapper filename equals the selected binary name (so it behaves like a normal installed command).
- `REPO` is the crate root directory (where the selected `Cargo.toml` lives).
- `REPO` is an absolute path and must NOT resolve symlinks (use an absolute version of the discovered path, but do not canonicalize).
- Arguments are forwarded exactly via `-- "$@"`.

## Install Location

Install directory selection:

- If `XDG_BIN_HOME` is set, use it (not an XDG standard, but supported).
- Otherwise use `$HOME/.local/bin`.

If the chosen directory is not on `PATH`, print a warning with a ready-to-paste snippet, e.g.:

```bash
export PATH="<INSTALL_DIR>:$PATH"
```

Suggest adding it to a shell startup file (e.g. `~/.profile` for many setups).

## Choosing the Wrapper Name

- If the crate defines exactly one binary target, select it automatically.
- If multiple binaries exist:
  - If `--bin <name>` is provided, use it (validate it exists).
  - Otherwise prompt with a numbered selection (interactive).

## File Layout

- `src/main.rs`
  - Linux-only gate.
  - Calls `cargo_dev_install::run()`.
  - Prints any error to stderr and exits `1`.

- `src/lib.rs`
  - Owns the high-level workflow and keeps most decisions pure/testable via an `InstallPlan`.

- `src/cli.rs`
  - CLI parsing (suggested dependency: `clap`).
  - Minimal flags: `--bin <name>`, `--force`, plus `--help`/`--version`.
  - Produces `CliArgs { bin: Option<String>, force: bool }`.

- `src/project.rs`
  - Crate root discovery and binary enumeration (suggested dependency: `cargo_metadata`).
  - Responsibilities:
    - `find_crate_root(cwd) -> crate_root` by walking parents to `Cargo.toml` (absolute path, do not resolve symlinks).
    - `list_bins(manifest_path) -> Vec<String>` of binary target names for selection.

- `src/tui_select.rs`
  - Interactive selection (numbered prompt; no fancy TUI).
  - `select_bin(bin_names: &[String]) -> Result<String>`.
  - Keep I/O injectable (read/write handles) so it’s easy to unit test.

- `src/install.rs`
  - Install directory selection, wrapper rendering, wrapper write.
  - Responsibilities:
    - `install_dir(env) -> PathBuf`: prefer `XDG_BIN_HOME` if set, else `$HOME/.local/bin`.
    - `is_on_path(dir, PATH) -> bool` for warning behavior.
    - `render_wrapper(crate_root) -> String` that forwards args and uses `--manifest-path "$REPO/Cargo.toml"`.
    - `write_wrapper(wrapper_path, contents, force) -> Result<()>`:
      - create parent dir if needed
      - if wrapper exists and `!force`: warn + error
      - otherwise write atomically and set executable perms (e.g. `0o755`)

## Core Types

### `InstallPlan` (in `src/lib.rs`)

A pure data structure describing what will be installed.

Suggested fields:

- `crate_root: PathBuf` (absolute; no symlink resolution)
- `manifest_path: PathBuf` (`crate_root/Cargo.toml`)
- `bin_name: String`
- `install_dir: PathBuf`
- `wrapper_path: PathBuf` (`install_dir/bin_name`)
- `wrapper_contents: String`
- `warn_path_missing: bool`

## Control Flow

1. `main` calls `run()` and handles exit code mapping.
2. `run()`:
   - parse CLI args (`cli`)
   - snapshot environment (`HOME`, `PATH`, `XDG_BIN_HOME`) and cwd
   - build an `InstallPlan` (`make_plan`)
   - apply the plan (`apply_plan`)
3. `make_plan(args, env, cwd)`:
   - `project::find_crate_root(cwd)`
   - `project::list_bins(manifest_path)`
   - choose `bin_name`:
     - if exactly one bin: select it
     - else if `--bin` provided: validate it exists
     - else: prompt via `tui_select::select_bin`
   - `install::install_dir(env)` and compute `wrapper_path`
   - `install::render_wrapper(crate_root)` and set `warn_path_missing`
4. `apply_plan(plan, force)`:
   - `install::write_wrapper(plan.wrapper_path, plan.wrapper_contents, force)`
   - if `warn_path_missing`: print PATH guidance

## Suggested Dependencies

- `clap` for CLI parsing (keeps the surface small but robust).
- `cargo_metadata` for enumerating binary targets safely (avoids hand-parsing `Cargo.toml`).
