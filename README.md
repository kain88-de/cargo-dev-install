# Cargo dev install

Linux-only Cargo subcommand that installs a wrapper script so your app runs from the working tree via `cargo run`.

## Install the plugin

```bash
cargo install --path .
# or
cargo install --git <repo-url>
```

Ensure `~/.cargo/bin` (or `$CARGO_HOME/bin`) is on your `PATH`.

## Use

```bash
cargo dev-install
```

## Behavior

- Wrapper name matches the selected binary.
- Install dir: `XDG_BIN_HOME` if set, else `$HOME/.local/bin`.
- Warns if install dir is not on `PATH`.
- Does not overwrite existing wrappers unless `--force`.
- `REPO` is an absolute crate root path (no symlink resolution).
