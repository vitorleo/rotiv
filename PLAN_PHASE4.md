# Rotiv Phase 4: Data Layer

## Context

Phase 3 delivered JSX compilation via `@swc/core`, `@rotiv/jsx-runtime`, `@rotiv/signals` (SSR-only), and `rotiv build`. The full request pipeline is now: Rust axum server → POST to Node route-worker → SWC transform → dynamic import → `loader(ctx)` → `renderToString()` → HTML.

Phase 4 adds the data layer: a TypeScript model DSL, SQLite (dev) / PostgreSQL (prod) drivers, schema migration, and `db` injection into loader/action context. The primary constraint is **no new heavy Rust crates** — all database work runs in Node.js subprocesses, following the same pattern established by `rotiv-compiler` → `@rotiv/build-script`.

**Goal:** A loader with `ctx.db.drizzle.select().from(UserModel.table)` → typed results → rendered HTML.

---

## Key Design Decisions

### D16: Drizzle ORM as the TypeScript database layer
Drizzle provides a single TypeScript-native schema DSL for both SQLite and PostgreSQL, `drizzle-kit` for migrations, and first-class ESM support. Chosen over raw `better-sqlite3`/`pg` because it eliminates a parallel query-builder DSL.

### D17: `defineModel()` as a thin branded wrapper over Drizzle tables
`defineModel(name, drizzleTable)` attaches `_type: "ModelDefinition"` and `_name` to a Drizzle table object. The result is simultaneously a valid Drizzle table (for queries) and a Rotiv-typed model (for the registry and spec). Column helpers (`text`, `integer`, etc.) are re-exported from `@rotiv/orm` so route files need only one import.

### D18: Single DB connection per route-worker process lifetime
`initDb(projectDir)` is called once at worker startup before `app.listen()`. Connection is stored in module scope. `DATABASE_URL` controls driver: starts with `postgres://` → `pg.Pool`; otherwise → `better-sqlite3` at `<projectDir>/app/.rotiv/dev.db`. The `RotivDb.query()` method wraps both drivers in a unified async interface.

### D19: `@rotiv/migrate-script` subprocess pattern
A new private package mirrors `@rotiv/build-script` exactly: Rust spawns `node --import tsx <script_path>` with flags, reads JSON from stdout. Modes: `--generate` (drizzle-kit generate), `--migrate` (drizzle-kit migrate), `--check` (read journal, no subprocess spawn), `--introspect` (import model files, emit field metadata).

### D20: Auto-migration on `rotiv dev` with fast journal check
On startup, after the worker is healthy, Rust runs a `--check` mode call (reads journal JSON — no drizzle-kit subprocess). If pending migrations exist, it spawns the full `--migrate` call. This keeps startup fast when no schema changes occurred.

### D21: Model file discovery in Rust, type extraction in Node
`rotiv-core::models::discover_models()` does filesystem glob (fast, no subprocess). Full field-level introspection (`--introspect` mode in `@rotiv/migrate-script`) is deferred to Phase 5's `rotiv info --verbose`.

### D22: `RotivDb.drizzle: unknown` at the `@rotiv/types` layer
Keeps `@rotiv/types` dependency-free. Route files that need type-safe queries import `DrizzleInstance` from `@rotiv/orm`, which narrows `drizzle` to the concrete Drizzle type. The two-layer approach: `@rotiv/types` defines the interface; `@rotiv/orm` provides the implementation.

---

## File Tree (additions/modifications only)

```
packages/@rotiv/
  orm/                                            [NEW package]
    package.json                                  type: module, deps: drizzle-orm, better-sqlite3, pg
    tsconfig.json
    src/
      types.ts                                    [NEW] ModelDefinition, RotivDb, DrizzleInstance, ModelRegistry
      define-model.ts                             [NEW] defineModel() + re-exported Drizzle column helpers
      db.ts                                       [NEW] createDb(options) → RotivDb (SQLite or PG)
      registry.ts                                 [NEW] globalModelRegistry singleton
      index.ts                                    [NEW] re-exports

  migrate-script/                                 [NEW internal package, private]
    package.json                                  private: true, deps: @rotiv/orm, drizzle-kit, drizzle-orm
    tsconfig.json
    src/
      drizzle-config.ts                           [NEW] write <projectDir>/.rotiv/drizzle.config.ts
      runner.ts                                   [NEW] generateMigrations(), applyMigrations(), checkPending(), introspect()
      index.ts                                    [NEW] CLI: --generate | --migrate | --check | --introspect, prints JSON to stdout

  route-worker/
    package.json                                  [MODIFY] add @rotiv/orm: workspace:*, better-sqlite3, pg
    src/
      db.ts                                       [NEW] module-scoped connection: initDb(projectDir), getDb()
      index.ts                                    [MODIFY] await initDb() before app.listen()
      invoke.ts                                   [MODIFY] buildContext() adds db: getDb()

  types/
    src/
      db.ts                                       [NEW] RotivDb interface (no Drizzle import — keeps @rotiv/types dep-free)
      framework.ts                                [MODIFY] LoaderContext + ActionContext gain db: RotivDb
      index.ts                                    [MODIFY] export RotivDb from db.ts

crates/
  rotiv-orm/
    Cargo.toml                                    [MODIFY] add serde, serde_json (workspace)
    src/
      lib.rs                                      [MODIFY] expose pub mod discovery, pub mod migration
      error.rs                                    [MODIFY] add SpawnFailed, MigrationFailed, ScriptNotFound, ParseFailed, PendingMigrations, Io
      discovery.rs                                [NEW] discover_models(project_dir) → Vec<ModelFileEntry>
      migration.rs                                [NEW] MigrationOptions, MigrationResult, run_migrations(), resolve_migrate_script_path()

  rotiv-core/
    src/
      models.rs                                   [NEW] discover_models(dir) → Vec<ModelEntry> (for spec + startup print)
      lib.rs                                      [MODIFY] pub mod models; pub use models::{discover_models, ModelEntry}
      server.rs                                   [MODIFY] print model count at startup; call auto_migrate after worker ready

  rotiv-cli/
    Cargo.toml                                    [MODIFY] add rotiv-orm path dep
    src/
      cli.rs                                      [MODIFY] add Migrate { generate_only: bool, check: bool }
      main.rs                                     [MODIFY] dispatch Commands::Migrate
      commands/
        mod.rs                                    [MODIFY] add pub mod migrate
        migrate.rs                                [NEW] run(generate_only, check, mode) → find_project_root + run_migrations()
        dev.rs                                    [MODIFY] call auto_migrate() after worker.wait_ready()

templates/default/
  app/models/
    user.ts                                       [NEW] example model with defineModel() + FRAMEWORK: comment
  package.json                                    [MODIFY] add @rotiv/orm dep, drizzle-kit devDep
```

---

## Implementation Waves

### Wave 1 — `@rotiv/orm`

New package. Pure TypeScript, no Rust changes.

**`src/types.ts`:**
```typescript
export interface ModelDefinition<TTable> {
  readonly _type: "ModelDefinition";
  readonly _name: string;
  readonly table: TTable;
}
export type DrizzleInstance =
  | BetterSQLite3Database<Record<string, never>>
  | NodePgDatabase<Record<string, never>>;
export interface RotivDb {
  readonly _driver: "sqlite" | "postgres";
  readonly drizzle: DrizzleInstance;
  query<T = unknown>(sql: string, params?: unknown[]): Promise<T[]>;
}
```

**`src/define-model.ts`:**
```typescript
export function defineModel<TTable>(name: string, table: TTable): ModelDefinition<TTable>
// Re-exports: sqliteTable, pgTable, text, integer, varchar, serial, sql, eq, and, or, ...
```

**`src/db.ts`:**
```typescript
export async function createDb(options: {
  databaseUrl?: string;   // if starts with postgres:// → PG; else SQLite
  projectDir?: string;    // SQLite file: <projectDir>/app/.rotiv/dev.db
}): Promise<RotivDb>
```

SQLite: `better-sqlite3` sync driver wrapped in async interface; `fs.mkdirSync(dir, { recursive: true })` ensures `app/.rotiv/` exists.
PG: `pg.Pool` with `max: 5`.

**`package.json`:**
```json
{
  "name": "@rotiv/orm",
  "version": "0.1.0",
  "type": "module",
  "exports": { ".": "./src/index.ts" },
  "dependencies": {
    "drizzle-orm": "^0.44.0",
    "better-sqlite3": "^9.0.0",
    "pg": "^8.0.0"
  },
  "devDependencies": {
    "@types/better-sqlite3": "^7.0.0",
    "@types/pg": "^8.0.0",
    "drizzle-kit": "^0.30.0",
    "tsx": "^4.0.0",
    "typescript": "^5.0.0"
  }
}
```

Verify: `pnpm --filter @rotiv/orm typecheck`

---

### Wave 2 — DB connection in route-worker + context types

**New `packages/@rotiv/types/src/db.ts`** (no Drizzle import — keeps `@rotiv/types` dep-free):
```typescript
export interface RotivDb {
  readonly _driver: "sqlite" | "postgres";
  readonly drizzle: unknown;  // narrowed to DrizzleInstance in @rotiv/orm
  query<T = unknown>(sql: string, params?: unknown[]): Promise<T[]>;
}
```

**Modify `packages/@rotiv/types/src/framework.ts`:**
```typescript
import type { RotivDb } from "./db.js";
// Add to LoaderContext and ActionContext:
readonly db: RotivDb;
```

**New `packages/@rotiv/route-worker/src/db.ts`:**
```typescript
import { createDb } from "@rotiv/orm";
let _db: RotivDb | null = null;
export async function initDb(projectDir: string): Promise<void>
export function getDb(): RotivDb  // throws if not initialized
```

**Modify `packages/@rotiv/route-worker/src/index.ts`:**
```typescript
await initDb(process.env["ROTIV_PROJECT_DIR"] ?? process.cwd());
// then: app.listen(...)
```

**Modify `packages/@rotiv/route-worker/src/invoke.ts`** — `buildContext()`:
```typescript
import { getDb } from "./db.js";
// Add to returned object:
db: getDb(),
```

Verify: `pnpm --filter @rotiv/types typecheck && pnpm --filter @rotiv/route-worker typecheck`

---

### Wave 3 — `@rotiv/migrate-script`

Internal package. Follows `@rotiv/build-script` pattern exactly.

**`src/runner.ts`:**
```typescript
export function generateMigrations(projectDir: string): MigrateResult
  // 1. writeDrizzleConfig(projectDir)
  // 2. spawnSync(drizzleKitBin, ["generate"], { cwd: projectDir })
  // 3. return { ok, migrationFiles, duration_ms }

export function applyMigrations(projectDir: string): MigrateResult
  // spawnSync(drizzleKitBin, ["migrate"], { cwd: projectDir })

export function checkPending(projectDir: string): { pending: number; ok: boolean }
  // Reads <projectDir>/.rotiv/migrations/_journal.json
  // Compares to applied migrations in dev.db — pure JSON read, no subprocess

export async function introspectModels(projectDir: string): Promise<ModelIntrospection[]>
  // dynamic import() of each app/models/*.ts file via tsx loader
```

`drizzleKitBin` = local `node_modules/.bin/drizzle-kit` (not `npx` — avoids overhead).

**`src/drizzle-config.ts`** — writes `.rotiv/drizzle.config.ts` at runtime:
```typescript
export default {
  schema: "./app/models/*.ts",
  out: "./.rotiv/migrations",
  dialect: "sqlite",  // or "postgresql" from DATABASE_URL
  dbCredentials: { url: "./app/.rotiv/dev.db" },
};
```

**`src/index.ts`** — CLI entry, prints JSON to stdout:
```
{ "ok": true, "migrations_applied": 1, "migration_files": ["..."], "duration_ms": 312 }
```

Verify: `pnpm --filter @rotiv/migrate-script typecheck`

---

### Wave 4 — `rotiv-orm` Rust crate

No new Rust crate dependencies. Only `serde_json` (already in workspace).

**`src/error.rs`:**
```rust
pub enum OrmError {
    NotImplemented(String),
    ScriptNotFound(String),
    SpawnFailed(String),
    MigrationFailed(String),
    ParseFailed(String),
    PendingMigrations(u32),
    #[from] Io(std::io::Error),
}
```

**`src/discovery.rs`:**
```rust
pub struct ModelFileEntry { pub name: String, pub file: PathBuf }
/// Scans <project_dir>/app/models/ for *.ts files.
/// Converts snake_case filename → PascalCase (user.ts → User).
/// Returns empty Vec if directory does not exist.
pub fn discover_models(project_dir: &PathBuf) -> Result<Vec<ModelFileEntry>, OrmError>
```

**`src/migration.rs`:**
```rust
pub struct MigrationOptions {
    pub project_dir: PathBuf,
    pub generate_only: bool,
    pub check_only: bool,
    pub json_output: bool,
}
pub struct MigrationResult {
    pub migrations_applied: u32,
    pub migration_files: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}
pub fn run_migrations(options: MigrationOptions) -> Result<MigrationResult, OrmError>
pub fn resolve_migrate_script_path() -> Result<PathBuf, OrmError>
  // 1. ROTIV_MIGRATE_SCRIPT_PATH env
  // 2. <binary>/../../packages/@rotiv/migrate-script/src/index.ts (dev)
  // 3. <binary>/migrate-script/index.ts (prod)
```

Spawns: `node --import tsx <script_path> --project <dir> [--generate | --migrate | --check]`
Parses JSON stdout → `MigrationResult`. Non-zero exit → `OrmError::MigrationFailed(stderr)`.

**`crates/rotiv-core/src/models.rs`** (startup print + spec):
```rust
pub struct ModelEntry { pub name: String, pub file: PathBuf }
pub fn discover_models(models_dir: &Path) -> Result<Vec<ModelEntry>, RotivError>
```

Verify: `cargo test --workspace`

---

### Wave 5 — `rotiv migrate` CLI command

**`crates/rotiv-cli/src/cli.rs`:**
```rust
Migrate {
    #[arg(long)] generate_only: bool,
    #[arg(long)] check: bool,
},
```

**`crates/rotiv-cli/src/commands/migrate.rs`:**
```rust
pub fn run(generate_only: bool, check: bool, mode: OutputMode) -> Result<(), CliError>
  // find_project_root() → MigrationOptions → run_migrations() → print human/JSON
```

**`crates/rotiv-cli/src/commands/dev.rs`** — add after `worker.wait_ready()`:
```rust
let models_dir = project_dir.join("app").join("models");
if models_dir.exists() {
    if let Err(e) = auto_migrate(&project_dir) {
        eprintln!("  [migrate]  warning: {e}");
        // Non-fatal — dev server continues
    }
}
```

Add `rotiv-orm` path dep to `crates/rotiv-cli/Cargo.toml`.

Verify: `cargo build --workspace` + `rotiv migrate --help`

---

### Wave 6 — Template + spec updates

**`templates/default/app/models/user.ts`** (replaces `.gitkeep`):
```typescript
// FRAMEWORK: Model definition using @rotiv/orm's defineModel().
// defineModel(name, drizzleTable) registers this model in the model registry.
// Column helpers (sqliteTable, text, integer) are re-exported from @rotiv/orm.
import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

export const UserModel = defineModel(
  "User",
  sqliteTable("users", {
    id: integer("id").primaryKey({ autoIncrement: true }),
    name: text("name").notNull(),
    email: text("email").notNull().unique(),
    createdAt: text("created_at")
      .$defaultFn(() => new Date().toISOString())
      .notNull(),
  })
);

export type User = typeof UserModel.table.$inferSelect;
export type NewUser = typeof UserModel.table.$inferInsert;
```

**`templates/default/package.json`** — add `@rotiv/orm: "^0.1.0"` dep, `drizzle-kit: "^0.30.0"` devDep.

**`crates/rotiv-cli/src/commands/new.rs`** — embed + write `app/models/user.ts`.

**`crates/rotiv-core/src/server.rs`** — discover models at startup, print count.

---

### Wave 7 — E2E verification

`e2e-test-phase4/` workspace member. Route uses `ctx.db.drizzle.select().from(UserModel.table)`.

1. `rotiv migrate --generate-only` → `.rotiv/migrations/0000_initial.sql`, exit 0
2. `rotiv migrate` → `app/.rotiv/dev.db` created, exit 0
3. `rotiv migrate --check` → exit 0 (no pending)
4. `rotiv dev` → auto-migrate (no pending), HTTP 200 on `/`
5. `curl /` → HTML with rendered user list (empty array)
6. Add field → `rotiv migrate --generate-only` → new migration file
7. `rotiv migrate` → new migration applied
8. `rotiv build` → `dist/server/routes/index.mjs`, exit 0
9. `rotiv migrate --json` → valid JSON
10. `e2e-test-phase3` unchanged (backward compat)

---

## Acceptance Criteria

- [ ] `cargo test --workspace` — all tests pass (including new `rotiv-orm` tests)
- [ ] `cargo build --workspace` — 0 errors, no new crates added
- [ ] `pnpm --filter @rotiv/orm typecheck` — pass
- [ ] `pnpm --filter @rotiv/migrate-script typecheck` — pass
- [ ] `pnpm --filter @rotiv/route-worker typecheck` — pass
- [ ] `pnpm --filter @rotiv/types typecheck` — pass
- [ ] `rotiv migrate --help` — exits 0, shows `--generate-only` and `--check`
- [ ] `rotiv migrate --check` with no `app/models/` → exits 0
- [ ] `rotiv migrate` in `e2e-test-phase4/` → `.rotiv/migrations/` and `app/.rotiv/dev.db` created
- [ ] Loader with `ctx.db.drizzle.select().from(UserModel.table)` → rendered HTML
- [ ] `ctx.db.query("SELECT 1 as n")` → `[{ n: 1 }]`
- [ ] Missing `db` on context → TypeScript compile error
- [ ] Auto-migrate check adds < 500ms when no pending migrations
- [ ] `e2e-test-phase3` still serves JSX HTML (backward compat)

---

## Critical Files

| File | Change |
|------|--------|
| [packages/@rotiv/types/src/framework.ts](packages/@rotiv/types/src/framework.ts) | Add `db: RotivDb` to `LoaderContext` and `ActionContext` |
| [packages/@rotiv/route-worker/src/invoke.ts](packages/@rotiv/route-worker/src/invoke.ts) | `buildContext()` includes `db: getDb()` |
| [packages/@rotiv/route-worker/src/index.ts](packages/@rotiv/route-worker/src/index.ts) | `await initDb(projectDir)` before `app.listen()` |
| [crates/rotiv-orm/src/migration.rs](crates/rotiv-orm/src/migration.rs) | `run_migrations()` — mirror `rotiv-compiler/src/lib.rs` pattern |
| [crates/rotiv-cli/src/cli.rs](crates/rotiv-cli/src/cli.rs) | Add `Migrate` command variant |

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| `better-sqlite3` native binary fails on Windows | Ships prebuilt binaries for Node 22; document Python fallback |
| `drizzle-kit` slow via npx | Use `node_modules/.bin/drizzle-kit` absolute path; pin versions |
| drizzle-kit needs importable TS schema | migrate-script uses `--import tsx`, same as route-worker |
| `RotivDb.drizzle: unknown` breaks type safety | Route files import `DrizzleInstance` from `@rotiv/orm`; documented in FRAMEWORK comment |
| Auto-migrate blocks startup | `--check` reads journal JSON only; full migrate only when pending > 0 |
| `app/.rotiv/` dir missing | `initDb()` calls `mkdirSync({ recursive: true })` before connecting |
| Phase 3 backward compat broken | No `app/models/` → auto-migrate skipped; `@rotiv/orm` not imported in old routes |
