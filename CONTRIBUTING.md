# Contributing to Rotiv

Thank you for your interest in contributing. This document covers how to get started, the project conventions, and what to expect from the process.

---

## Getting Started

### Prerequisites

- **Rust** (stable, via [rustup](https://rustup.rs)) — for building the CLI and crates
- **Node.js 22+** and **pnpm 10+** — for the TypeScript packages
- **sccache** (recommended) — speeds up incremental Rust builds

```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

### Setup

```bash
git clone https://github.com/vitorleo/rotiv.git
cd rotiv

# Install TypeScript dependencies
pnpm install

# Build the CLI
cargo build -p rotiv-cli

# Run all tests
cargo test --workspace
pnpm typecheck
```

---

## Project Layout

```
crates/           Rust workspace (CLI, core server, ORM, compiler)
packages/@rotiv/  TypeScript packages (SDK, types, ORM, signals, MCP, ...)
reference-apps/   Annotated example applications
e2e-tests/        End-to-end test projects (one per implementation phase)
docs/             Plans, changelogs, architectural decisions
```

Each Rust crate must compile independently. Keep cross-crate dependencies minimal — Rust clean builds are expensive on low-power hardware.

---

## Making Changes

### Rust (crates/)

- Format: `cargo fmt`
- Lint: `cargo clippy --workspace -- -D warnings`
- Test: `cargo test --workspace`
- Every new public function should have at least a unit test.
- Error messages must use the structured format (`code`, `message`, `suggestion`). See `rotiv_core::RotivError`.

### TypeScript (packages/)

- Typecheck: `pnpm typecheck`
- All public types should be exported from the relevant `@rotiv/*` package.
- Avoid runtime dependencies that would bloat the installed package size.

### CLI commands

When adding or changing a CLI command:

1. Add/update the variant in `crates/rotiv-cli/src/cli.rs`
2. Implement in `crates/rotiv-cli/src/commands/<name>.rs`
3. Dispatch in `crates/rotiv-cli/src/main.rs`
4. If the command produces knowledge-base content, add a `.md` file under `crates/rotiv-cli/src/knowledge/` and register it in `explain.rs`
5. Update the MCP tool manifest at `packages/@rotiv/mcp/index.json`

---

## Commits and PRs

- Keep commits focused — one logical change per commit.
- Write commit messages in the imperative mood: `add deploy command`, not `added deploy command`.
- Open a PR against `main`. The PR description should explain the *why*, not just the *what*.
- CI must pass (Rust check + test + clippy, TypeScript typecheck) before merging.

---

## Reporting Issues

Use [GitHub Issues](https://github.com/vitorleo/rotiv/issues). Include:

- What you ran (the exact command)
- What you expected
- What actually happened (paste the full output, especially if `--json` was used)
- Your OS and `rotiv --version`

---

## Scope and Roadmap

Rotiv is an experimental, opinionated framework. Contributions that add flexibility or escape hatches where the framework intentionally has none are unlikely to be accepted. When in doubt, open an issue first to discuss before writing code.

Good candidates for contributions:

- Bug fixes
- Improving structured error messages and auto-fix suggestions
- Additional `rotiv explain` topics
- Expanding the `rotiv validate` diagnostic rules
- Improving the reference apps
- Documentation corrections

---

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
