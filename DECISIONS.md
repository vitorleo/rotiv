# Architectural Decision Log

## D1: Template Embedding

**Decision:** Templates are embedded in the CLI binary using Rust's `include_str!` macro.

**Rationale:** No runtime file-system dependency for the CLI. Simple string substitution for `{{project_name}}` and `{{created_at}}` placeholders.

**Alternative considered:** Runtime file lookup (ship templates alongside the binary). Rejected — adds install complexity and makes single-binary distribution impossible.

---

## D2: rotiv-core in Phase 1

**Decision:** `rotiv-core` is a stub crate in Phase 1, only defining shared error types and `RotivError`.

**Rationale:** Keeps `rotiv-cli` compilable without pulling in `axum`/`tokio` yet. The actual HTTP server lives in Phase 2.

**Alternative considered:** Bundle all core types into `rotiv-cli` directly. Rejected — would require large-scale refactoring when Phase 2 lands.

---

## D3: No napi-rs in Phase 1

**Decision:** Phase 1's TypeScript SDK is pure TypeScript stubs — no native module.

**Rationale:** The napi-rs bridge between Rust and TypeScript is Phase 2 work. `rotiv dev` (the command needing the bridge) is not implemented yet.

**Alternative considered:** Add napi-rs bindings now as a stub. Rejected — adds build complexity (cross-compilation, node-gyp) with no benefit until Phase 2.

---

## D4: sccache Configuration

**Decision:** `.cargo/config.toml` sets `rustc-wrapper = "sccache"` and uses sparse registry.

**Rationale:** Incremental Rust builds can be reduced from ~15–20 min to seconds with sccache on a warm cache. Sparse registry speeds up `cargo update` on slow connections.

**Alternative considered:** No caching. Rejected — unacceptably slow iteration on Windows.

---

## D5: pnpm Workspaces

**Decision:** pnpm workspaces for the TypeScript monorepo. Packages reference each other via `workspace:*`.

**Rationale:** pnpm's strict hoisting prevents phantom dependency issues. `workspace:*` protocol gives correct version pinning in monorepo context.

**Alternative considered:** npm workspaces. Rejected — pnpm's performance and strict dependency isolation are superior for multi-package repos.

---

## D6: Spec JSON Schema

**Decision:** `.rotiv/spec.json` uses a versioned schema URL (`https://rotiv.dev/spec/v1`) but is also bundled locally at `packages/@rotiv/spec/spec.schema.json` for offline use.

**Rationale:** The `@rotiv/spec` package exports both TypeScript types and the JSON Schema. This enables IDE validation via `$schema` and programmatic validation via the `validateSpec` function.

**Alternative considered:** TOML for the spec file. Rejected — JSON has better agent tooling support and requires no extra Rust dependency.
