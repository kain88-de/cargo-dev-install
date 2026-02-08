# Cargo dev install

This is a cargo plugin that can be used to have a dev install for a rust application

```bash
cargo dev-install
```

# Details

The plugin will install an app wrapper like the following in your PATH

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO="path/to/app"
exec cargo  run --manifest-path "$REPO/Cargo.toml" -- "$@"
```

by default $HOME/.local/bin is chosen. If this is not in your path the plugin
will show a warning during installation and show how to fix your PATH variable.
