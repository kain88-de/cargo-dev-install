# AGENTS

This file is guidance for future coding agents working on this repository.

## Project

- This repository builds a Rust application (a Cargo subcommand plugin).
- Use stable Rust.
- Build and run everything with Cargo.

## Dependency policy

- Prefer the Rust standard library.
- Avoid new dependencies unless they meaningfully reduce complexity.
- If you add a dependency, justify it in the PR/commit message and keep scope small.

## Implementation workflow (TDD)

Use a red/green/refactor loop:

1. Red: write/extend a test that fails.
2. Green: implement the smallest change to make it pass.
3. Refactor: clean up structure/naming, keep behavior identical, keep tests passing.

Guidelines:

- Keep core logic in pure functions where possible; unit test those functions.
- Add integration tests for filesystem and process behavior.
- Keep tests hermetic by controlling `HOME`, `PATH`, and `CARGO_TARGET_DIR`.
- Exit codes are simple: `0` on success, `1` on any error.

## Commands

- Run unit/integration tests:
  - `cargo test`
- Lint (if configured):
  - `cargo clippy -- -D warnings`
- Format:
  - `cargo fmt`
