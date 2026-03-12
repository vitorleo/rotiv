# Rotiv Framework — Phase 1 Implementation Plan

## Context

Rotiv is an AI-native full-stack web framework built with TypeScript + Rust. The project is starting from a completely empty directory. Phase 1 establishes the monorepo scaffold, the CLI binary with the `rotiv new` command, the `.rotiv/spec.json` schema, and the TypeScript SDK type stubs. Nothing else exists yet.

**Environment:**
- Rust 1.87.0, Node.js 22.16.0, pnpm 10.9.2
- Windows 11 + WSL2 (Ubuntu 24.04) — use sccache, minimize Rust rebuilds
- Clean Rust builds: ~15–20 min. Incremental with sccache: fast.

---

## Complete File Tree (Phase 1)

```
rotiv/
  .cargo/
    config.toml                    # sccache rustc-wrapper + sparse registry
  .gitignore
  .github/
    workflows/
      ci.yml                       # cargo check + tsc --noEmit (no full build)
  Cargo.toml                       # workspace root
  Cargo.lock
  pnpm-workspace.yaml
  package.json                     # root package (private, workspace scripts)
  tsconfig.base.json               # shared TS compiler options
  DECISIONS.md                     # architectural decision log
  crates/
    rotiv-core/
      Cargo.toml
      src/
        lib.rs                     # pub mod declarations, re-exports
        error.rs                   # RotivError, structured error type
    rotiv-cli/
      Cargo.toml
      src/
        main.rs                    # entry point, parse CLI, call run()
        cli.rs                     # Cli struct (clap derive), subcommand enum
        error.rs                   # CliError, Display, structured JSON error
        commands/
          mod.rs
          new.rs                   # rotiv new <name> implementation
          info.rs                  # rotiv info (prints spec.json summary)
        output/
          mod.rs
          json.rs                  # OutputMode::Json, serialize to stdout
          human.rs                 # OutputMode::Human, colored terminal output
    rotiv-orm/
      Cargo.toml
      src/
        lib.rs                     # stub — phase 4
        error.rs
    rotiv-compiler/
      Cargo.toml
      src/
        lib.rs                     # stub — phase 3
        error.rs
  packages/
    @rotiv/
      types/
        package.json               # name: @rotiv/types
        tsconfig.json
        src/
          index.ts                 # barrel export
          framework.ts             # RotivConfig, RouteDefinition, etc.
          config.ts                # ProjectConfig type
          spec.ts                  # RotivSpec, SpecVersion types
      sdk/
        package.json               # name: @rotiv/sdk, deps: @rotiv/types
        tsconfig.json
        tsconfig.build.json
        src/
          index.ts
          router.ts                # defineRoute(), loader/action types
          server.ts                # createServer() stub
          context.ts               # RequestContext type
          middleware.ts            # MiddlewareFn type
          errors.ts                # RotivError structured type (mirrors Rust)
      spec/
        package.json               # name: @rotiv/spec
        tsconfig.json
        src/
          index.ts
          schema.ts                # full RotivSpec JSON schema definition
          validator.ts             # validateSpec(spec): ValidationResult
        spec.schema.json           # JSON Schema for .rotiv/spec.json
      create/
        package.json               # name: @rotiv/create, bin: create-rotiv
        tsconfig.json
        src/
          index.ts
          scaffold.ts              # scaffoldProject(name, dest): Promise<void>
          templates.ts             # inline template strings for generated files
  templates/
    default/
      .rotiv/
        spec.json                  # template spec.json for new projects
        context.md                 # template context.md
      app/
        routes/
          index.tsx                # hello world route
        models/
          .gitkeep
      package.json
      tsconfig.json
      README.md
```

---

## Implementation Order (waves)

### Wave 1 — Workspace Scaffolding (no compilation needed)
1. `Cargo.toml` (workspace root listing all 4 crates)
2. `.cargo/config.toml` (sccache, sparse registry)
3. `pnpm-workspace.yaml` + root `package.json`
4. `tsconfig.base.json`
5. `.gitignore`
6. `DECISIONS.md` (initial entries)

### Wave 2 — Rust Stub Crates (fast: just type-check skeletons)
Build all 4 crates as minimal stubs first so the workspace compiles:
- `rotiv-core/`: `lib.rs` + `error.rs` with `RotivError` type
- `rotiv-orm/`: `lib.rs` + `error.rs` stub
- `rotiv-compiler/`: `lib.rs` + `error.rs` stub
- `rotiv-cli/`: full implementation (see Wave 3)

**Key:** `rotiv-orm` and `rotiv-compiler` have **zero** external dependencies in Phase 1 — just `thiserror`. `rotiv-core` only depends on `thiserror` and `serde`. This keeps the workspace compilable in seconds with just `cargo check`.

### Wave 3 — rotiv-cli Implementation
Files: `main.rs`, `cli.rs`, `error.rs`, `commands/mod.rs`, `commands/new.rs`, `commands/info.rs`, `output/mod.rs`, `output/json.rs`, `output/human.rs`

**Dependencies (rotiv-cli/Cargo.toml):**
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
colored = "2"
fs_extra = "1"

[dependencies.rotiv-core]
path = "../rotiv-core"
```

**CLI structure (`cli.rs`):**
```rust
#[derive(Parser)]
#[command(name = "rotiv", version, about)]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    New { name: String },
    Info,
    // Phase 2+: Dev, Build, Deploy, Add, Explain, Validate, Migrate
}
```

**`rotiv new <name>` behavior:**
1. Create `<name>/` directory
2. Copy/generate files from embedded template (use `include_str!` macros)
3. Substitute `{{project_name}}` placeholders
4. Write `.rotiv/spec.json` with initial spec
5. Write `.rotiv/context.md` with initial project description
6. Print success (human) or `{"ok": true, "project": "<name>", "path": "..."}` (--json)

**Structured error format (enforced from day 1):**
```rust
#[derive(Debug, Serialize)]
pub struct RotivError {
    pub code: String,           // "E001", "E002", etc.
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub expected: Option<String>,
    pub got: Option<String>,
    pub suggestion: Option<String>,
    pub corrected_code: Option<String>,
}
```
When `--json`: print `{"error": <RotivError>}` to stderr, exit 1.
When human: print colored, formatted error to stderr, exit 1.

### Wave 4 — TypeScript Packages
Build in dependency order:
1. `@rotiv/types` — zero runtime deps, pure type definitions
2. `@rotiv/spec` — depends on `@rotiv/types`, includes `spec.schema.json`
3. `@rotiv/sdk` — depends on `@rotiv/types`
4. `@rotiv/create` — depends on `@rotiv/types`, `@rotiv/spec`

**`@rotiv/types` key types:**
```typescript
// framework.ts
export interface RouteDefinition { ... }
export interface LoaderContext { ... }
export interface ActionContext { ... }
export type MiddlewareFn = (ctx: RequestContext, next: () => Promise<void>) => Promise<void>;

// spec.ts
export interface RotivSpec {
  version: string;
  framework_version: string;
  project: { name: string; created_at: string };
  routes: RouteEntry[];
  models: ModelEntry[];
  modules: ModuleEntry[];
}
```

**`@rotiv/sdk` key exports:**
```typescript
export function defineRoute(config: RouteConfig): RouteDefinition
export function createServer(config?: Partial<ServerConfig>): RotivServer
export function signal<T>(initial: T): Signal<T>
export function derived<T>(fn: () => T): ReadonlySignal<T>
export function effect(fn: () => void): () => void
```
All implementations are stubs (`throw new Error("Not implemented: Phase 2")`) — types compile cleanly.

### Wave 5 — Templates & Spec Schema
- `templates/default/` with minimal project skeleton
- `spec.schema.json` — JSON Schema draft-07 for `.rotiv/spec.json`
- Embed templates in CLI binary via `include_str!`

---

## Key Design Decisions

### D1: Template Embedding
Templates are embedded in the CLI binary using Rust's `include_str!` macro. No runtime file-system dependency. Simple string substitution for `{{project_name}}`. Alternative (runtime file lookup) rejected — adds install complexity.

### D2: rotiv-core in Phase 1
`rotiv-core` is a stub crate in Phase 1 — it only defines shared error types and the `RotivError` struct. The actual HTTP server lives in Phase 2. This keeps `rotiv-cli` compilable without pulling in `axum`/`tokio` yet.

### D3: No napi-rs in Phase 1
The napi-rs bridge between Rust and TypeScript is Phase 2 work. Phase 1's TypeScript SDK is pure TypeScript stubs — no native module. This means `rotiv dev` (the command that needs the bridge) is not implemented yet.

### D4: sccache Configuration
`.cargo/config.toml` sets `RUSTC_WRAPPER = "sccache"` and uses the sparse registry (`sparse+https://index.crates.io/`) to speed up registry updates. CI also uses sccache with GitHub Actions cache.

### D5: pnpm Workspaces vs npm
pnpm workspaces are used for the TypeScript monorepo. Packages reference each other via `workspace:*` protocol. Root `package.json` has scripts: `typecheck`, `build`, `test`.

### D6: Spec JSON Schema
`.rotiv/spec.json` uses a versioned schema (`"$schema": "https://rotiv.dev/spec/v1"`) but the schema is also bundled locally at `.rotiv/schema.json` for offline use. The `@rotiv/spec` package exports both the TypeScript types and the JSON Schema.

---

## Cargo.toml (workspace root)

```toml
[workspace]
resolver = "2"
members = [
    "crates/rotiv-cli",
    "crates/rotiv-core",
    "crates/rotiv-orm",
    "crates/rotiv-compiler",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Rotiv Contributors"]
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
```

---

## .cargo/config.toml

```toml
[build]
rustc-wrapper = "sccache"

[net]
git-fetch-with-cli = true

[registries.crates-io]
protocol = "sparse"
```

---

## tsconfig.base.json

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "exactOptionalPropertyTypes": true,
    "noUncheckedIndexedAccess": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "skipLibCheck": false
  }
}
```

---

## .rotiv/spec.json Schema (initial version)

```json
{
  "$schema": "https://rotiv.dev/spec/v1",
  "version": "1",
  "framework_version": "0.1.0",
  "project": {
    "name": "{{project_name}}",
    "created_at": "{{created_at}}"
  },
  "routes": [],
  "models": [],
  "modules": [],
  "conventions": {
    "routes_dir": "app/routes",
    "models_dir": "app/models",
    "components_dir": "app/components"
  }
}
```

---

## Acceptance Criteria

Phase 1 is complete when:

- [ ] `cargo check --workspace` passes with zero errors and zero warnings
- [ ] `cargo test --workspace` passes (at least 1 unit test per crate)
- [ ] `cargo build --bin rotiv` produces a working binary
- [ ] `./rotiv new my-app` creates a valid project directory with:
  - `my-app/.rotiv/spec.json` (valid against spec.schema.json)
  - `my-app/.rotiv/context.md`
  - `my-app/app/routes/index.tsx`
  - `my-app/package.json`
- [ ] `./rotiv new my-app --json` outputs valid JSON to stdout
- [ ] `./rotiv info` prints framework version and spec summary
- [ ] `./rotiv new my-app` errors with structured JSON on `--json` flag if directory already exists
- [ ] `pnpm --filter @rotiv/types typecheck` passes
- [ ] `pnpm --filter @rotiv/sdk typecheck` passes
- [ ] `pnpm --filter @rotiv/spec typecheck` passes
- [ ] `pnpm --filter @rotiv/create typecheck` passes
- [ ] `DECISIONS.md` documents D1–D6 above

---

## Open Questions (resolved)

| # | Question | Decision |
|---|----------|----------|
| Q1 | Should `rotiv new` use async Rust? | No — it's a simple file operation. Sync is fine. tokio not needed in rotiv-cli Phase 1. |
| Q2 | Should spec.json be TOML or JSON? | JSON — better agent tooling support, no extra dep. |
| Q3 | TypeScript build tool for packages? | `tsc` only in Phase 1. No esbuild/tsup needed until we have runtime code. |
| Q4 | How to handle Windows path separators? | Use `std::path::PathBuf` throughout — it handles cross-platform paths correctly. |
| Q5 | Should rotiv-cli depend on rotiv-core? | Minimal dep: only for shared `RotivError` type. No HTTP types. |

---

## Verification Steps

1. **Rust check:** `cd rotiv && cargo check --workspace 2>&1`
2. **Rust tests:** `cargo test --workspace 2>&1`
3. **Build CLI:** `cargo build --bin rotiv`
4. **Run new command:** `./target/debug/rotiv new test-project`
5. **Verify output:** `ls test-project/.rotiv/` should show `spec.json` and `context.md`
6. **JSON mode:** `./target/debug/rotiv new test-project --json` (should error: already exists)
7. **TS typecheck:** `pnpm -r typecheck`
