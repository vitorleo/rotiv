# Rotiv Architecture

> Reference document for AI coding agents. Describes the technical structure, key abstractions, data flow, and conventions of the Rotiv framework codebase.

---

## Overview

Rotiv is an AI-native full-stack TypeScript web framework. Its primary goal is to be fully operable by AI coding agents: every project state can be queried, every file follows strict conventions, and the CLI provides structured JSON output for all commands.

The system has two layers:

1. **Rust CLI + runtime** — a single binary (`rotiv`) that runs all framework operations: dev server, file watcher, route discovery, migration runner, scaffolding, static analysis, and knowledge lookup.
2. **TypeScript packages** — published to npm; used by user project code at author-time (types) and at runtime (JSX rendering, ORM queries, signal primitives).

---

## Repository Layout

```
rotiv/
  crates/
    rotiv-cli/          # CLI binary — commands, templates, knowledge, modules
    rotiv-core/         # Core runtime — dev server, router, worker, watcher, analysis
    rotiv-orm/          # Database layer — migration runner, model discovery
    rotiv-compiler/     # Build pipeline (esbuild-based, used by `rotiv build`)
  packages/@rotiv/
    types/              # Core TypeScript type definitions
    sdk/                # Runtime APIs (defineRoute, ctx types)
    jsx-runtime/        # Server-side JSX renderToString
    signals/            # Signal primitives (SSR-safe)
    orm/                # Drizzle ORM wrapper (defineModel, typed queries)
    route-worker/       # Node.js worker that evaluates route modules at runtime
    migrate-script/     # Node.js script that runs Drizzle migrations
    mcp/                # MCP server exposing all CLI commands as AI tools
    build-script/       # esbuild config for `rotiv build`
    create/             # `create-rotiv` npm initializer
    spec/               # Spec.json schema types
  templates/
    default/            # `rotiv new` scaffold (package.json, tsconfig, etc.)
  e2e-test-phase*/      # End-to-end test projects (one per phase)
  reference-apps/
    todo/               # Reference todo app demonstrating all conventions
  docs/
    DECISIONS.md        # Architectural decision log (D1–D45+)
    PLAN_V0.2.0.md      # Implementation plan for v0.2.0
    changelog/          # Per-version changelogs
  .github/workflows/
    release.yml         # Binary release pipeline (Linux, macOS, Windows)
    npm-publish.yml     # npm package publish pipeline
```

---

## Rust Crates

### `rotiv-cli` — CLI Binary

Entry point: `crates/rotiv-cli/src/main.rs`

Uses [clap](https://docs.rs/clap) for argument parsing. The root `Commands` enum in `cli.rs` covers:

| Command | Description |
|---------|-------------|
| `new` | Scaffold a new project from template |
| `dev` | Start dev server with file watching |
| `build` | Production build via esbuild |
| `migrate` | Run or generate Drizzle migrations |
| `add route/model/module` | Scaffold annotated files |
| `spec sync` | Populate `.rotiv/spec.json` from filesystem |
| `validate [--fix]` | Static analysis, 7 diagnostic codes |
| `explain <topic>` | Embedded knowledge base lookup |
| `context regen` | Regenerate `.rotiv/context.md` |
| `diff-impact <file>` | Import graph scan, affected routes |
| `deploy` | SSH-based server deployment |

All commands support `--json` for machine-readable output. Errors always serialize as `RotivError` JSON.

**Templates** (`src/templates/`) — embedded via `include_str!()` at compile time:
- `add/route.tsx` — annotated route template
- `add/model.ts` — annotated model template
- `add/module_manifest.json`, `module_index.ts`, `module_test.ts` — module scaffold
- `default/` — full project scaffold (see also top-level `templates/`)

**Knowledge base** (`src/knowledge/`) — 10 Markdown files embedded via `include_str!()`:
`routes`, `models`, `loader`, `action`, `middleware`, `signals`, `migrate`, `context`, `modules`, `deploy`

**First-party modules** (`src/modules/`) — embedded TS source for `sessions`, `auth`, `file-uploads`:
```
src/modules/
  sessions/   module.json, index.ts, module.test.ts
  auth/       module.json, index.ts, module.test.ts
  file-uploads/ module.json, index.ts, module.test.ts
```
Installed via `rotiv add module <name>`. Custom modules use the generic template.

### `rotiv-core` — Runtime Engine

Key modules:

| Module | Responsibility |
|--------|---------------|
| `error.rs` | `RotivError` — structured error with code, message, file, line, expected, got, suggestion, corrected_code |
| `router/discovery.rs` | Glob `app/routes/**/*.{tsx,ts}`, derive HTTP paths, return `Vec<RouteEntry>` |
| `server.rs` | `DevServer` — orchestrates router registry, route worker, file watcher, Axum HTTP server |
| `worker.rs` | `RouteWorker` — spawns `tsx` subprocess running the embedded route-worker, communicates via HTTP |
| `proxy.rs` | `invoke_route()` — sends `InvokeRequest` to worker, returns `InvokeResponse` |
| `watcher.rs` | `FileWatcher` — uses `notify` to watch `app/` for changes, reloads registry |
| `analysis.rs` | `run_diagnostics()` — 7 diagnostic codes (V001–V007), `apply_fixes()` for auto-fixable ones |
| `models.rs` | `discover_models()` — glob `app/models/**/*.ts`, extract model name and table name |
| `modules.rs` | `discover_modules()`, `resolve_capabilities()`, `ModuleManifest` |
| `project.rs` | `find_project_root()` — walks up from CWD looking for `.rotiv/spec.json` |

**Dev server request flow:**
```
HTTP request
  → Axum router (server.rs)
  → RouteRegistry lookup (router/discovery.rs)
  → invoke_route() (proxy.rs) → HTTP POST to tsx worker
  → InvokeResponse (rendered HTML or JSON)
  → HTTP response
```

**Route worker resolution order** (`worker.rs::resolve_worker_path`):
1. `ROTIV_WORKER_PATH` env var (testing/override)
2. `embedded_path` passed from CLI (production — written to tempdir)
3. Dev monorepo layout: `../../packages/@rotiv/route-worker/src/index.ts` relative to binary
4. Production layout: sibling `route-worker/` directory

### `rotiv-orm` — Database Layer

| Module | Responsibility |
|--------|---------------|
| `migration.rs` | `run_migrations()` — writes embedded migrate-script to tempdir, runs via `tsx` |
| `discovery.rs` | `discover_models()` — scans `app/models/` for model files |
| `error.rs` | `OrmError` enum |

**Migrate script resolution order** (`migration.rs::resolve_migrate_script_path`):
Same 4-level pattern as route worker.

### `rotiv-compiler` — Build Pipeline

Wraps esbuild for `rotiv build`. Produces optimized JS bundles with SSR pre-rendering.

---

## TypeScript Packages

All packages use `"type": "module"`, target `NodeNext`, and publish from `dist/` after `tsc` build.

| Package | Purpose | Key exports |
|---------|---------|-------------|
| `@rotiv/types` | Core TypeScript types | `RouteContext`, `Loader`, `Action`, `Component`, `RouteDefinition` |
| `@rotiv/sdk` | Author-time + runtime APIs | `defineRoute()`, `ctx` |
| `@rotiv/jsx-runtime` | JSX transform + renderToString | `jsx`, `jsxs`, `renderToString` |
| `@rotiv/signals` | Signal primitives | `signal()`, `computed()`, `effect()` |
| `@rotiv/orm` | Drizzle wrapper | `defineModel()`, `sqliteTable`, `pgTable`, typed query builder |

### Route Worker (`@rotiv/route-worker`)

A Node.js HTTP server (listens on a random port). For each incoming `InvokeRequest`:
1. Loads the route file via dynamic `import()`
2. Calls `loader()` or `action()` as appropriate
3. Calls `component()` and renders JSX to HTML string via `renderToString`
4. Returns `InvokeResponse`

**Embedded into binary:** all 6 source files are embedded via `include_str!()` in `dev.rs` and written to a `tempfile::TempDir` at runtime.

### Migrate Script (`@rotiv/migrate-script`)

A Node.js script that:
1. Discovers `app/models/*.ts` to build the Drizzle schema
2. Uses `drizzle-kit` to generate SQL migration files in `migrations/`
3. Optionally runs the migrations against the configured database

**Embedded into binary:** 3 source files embedded via `include_str!()` in `migrate.rs`.

---

## Key Conventions

### Route Files

Location: `app/routes/**/*.tsx` or `app/routes/**/*.ts`

Required export:
```typescript
export default defineRoute({
  path: "/users/:id",       // must match file location
  async loader(ctx) { ... }, // optional, server-side data fetch
  async action(ctx) { ... }, // optional, form/mutation handler
  component({ data }) { ... } // optional, renders HTML
});
```

Dynamic segments: `[param]` in filename → `:param` in path. `index.tsx` → `/`.

API-only routes: omit `component`, return JSON from `loader`.

### Model Files

Location: `app/models/<snake_case>.ts`

Required exports:
```typescript
export const users = sqliteTable("users", { ... });     // raw Drizzle table (schema discovery)
export const userModel = defineModel("User", users);    // Rotiv registry
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
```

### Module Files

Location: `app/modules/<kebab-name>/`

Required files:
- `module.json` — manifest with `name`, `version`, `capabilities[]`, `requires[]`
- `index.ts` — middleware factory export
- `module.test.ts` — unit tests

### Error Codes

| Range | Crate | Domain |
|-------|-------|--------|
| E001–E009 | rotiv-core | Project structure, file system |
| E010–E019 | rotiv-cli | Scaffolding, add command |
| E011 | rotiv-cli | Invalid model name (must be PascalCase) |
| E012 | rotiv-cli | Invalid module name (must be kebab-case) |
| E015–E019 | rotiv-cli | Migration errors |
| E020–E024 | rotiv-cli | Deploy errors |
| V001–V007 | rotiv-core/analysis | Static analysis diagnostics |

### Diagnostic Codes (validate)

| Code | Check | Auto-fixable |
|------|-------|-------------|
| V001 | Route missing `export default defineRoute` | Yes |
| V002 | `defineRoute(` missing `component` field | No |
| V003 | Model missing `sqliteTable(` or `pgTable(` | No |
| V004 | Model missing `defineModel(` | No |
| V005 | Route uses raw `export default {` | No |
| V006 | Route uses `ctx.db` but no model import | No |
| V007 | Filename has `[param]` but path lacks `:param` | No |

---

## Data Flow: `rotiv dev`

```
rotiv dev
  1. find_project_root() — locate .rotiv/spec.json
  2. write_embedded_worker() — write route-worker TS to tempdir
  3. DevServerConfig { project_dir, port, worker_path: Some(tempdir_entry) }
  4. DevServer::start()
     a. RouteRegistry::load() — glob app/routes/**/*.{tsx,ts}
     b. print_routes() — show routes table with relative paths
     c. find_available_port()
     d. RouteWorker::new(project_dir, worker_port, embedded_path)
        → resolve_worker_path() → embedded_path wins
        → spawn `tsx <tempdir>/index.ts` subprocess
     e. FileWatcher::start() — watch app/ for changes
     f. Axum::serve() on configured port
  5. Per request: invoke_route() → HTTP POST to worker → response
  6. On file change: registry.load() + worker.reload()
```

## Data Flow: `rotiv migrate`

```
rotiv migrate
  1. find_project_root()
  2. write_embedded_migrate_script() — write migrate-script TS to tempdir
  3. run_migrations(MigrationOptions { db_url, generate_only, script_path: Some(tempdir_entry) })
     → resolve_migrate_script_path() → embedded_path wins
     → spawn `tsx <tempdir>/index.ts` with env vars
  4. Apply SQL from migrations/ to database
```

---

## Output Mode Pattern

All CLI commands accept `--json` and produce consistent output:

**Human mode** (default):
```
  ✓  created app/routes/users.tsx
     path    /users
```

**JSON mode** (`--json`):
```json
{"ok": true, "kind": "route", "file": "app/routes/users.tsx"}
```

**Error (human)**:
```
  ✗  E011: invalid model name 'user': must be PascalCase
     expected  PascalCase name (e.g. Post)
     got       user
     hint      Did you mean 'User'?
```

**Error (JSON)**:
```json
{"ok": false, "error": {"code": "E011", "message": "...", "suggestion": "...", "corrected_code": "User"}}
```

---

## Spec File (`.rotiv/spec.json`)

Created by `rotiv new`, updated by `rotiv spec sync` and `rotiv add module`.

```json
{
  "version": "1",
  "name": "my-app",
  "framework": "rotiv",
  "framework_version": "0.2.0",
  "routes": [
    { "path": "/", "file": "app/routes/index.tsx", "has_loader": true, "has_action": true, "has_component": true }
  ],
  "models": [
    { "name": "User", "file": "app/models/user.ts", "table": "users" }
  ],
  "modules": [
    { "name": "sessions", "version": "0.1.0" }
  ]
}
```

Used by `rotiv validate`, `rotiv context regen`, `rotiv explain`, and AI agents for project state queries.

---

## MCP Server (`@rotiv/mcp`)

JSON-RPC 2.0 server that exposes all 12 Rotiv CLI commands as typed tools. AI agent platforms (Claude Desktop, Cursor, etc.) connect to it and can call tools like `rotiv_validate`, `rotiv_explain`, `rotiv_spec_sync`, etc.

Each tool spawns `rotiv --json <command>` as a subprocess and returns the JSON output as the tool result.

Start with: `node packages/@rotiv/mcp/src/server.js`

---

## Testing

- **Unit tests**: in each crate's `src/` (via `#[test]`)
- **E2E projects**: `e2e-test-phase*/` — one per phase, each a real Rotiv project
- **Reference app**: `reference-apps/todo/` — full todo app showing all patterns

Run all tests: `cargo test --workspace`
Run TS typechecks: `pnpm -r typecheck`
