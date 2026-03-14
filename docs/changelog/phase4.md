# Phase 4 Changelog — Data Layer

**Date:** 2026-03-13
**Status:** Complete

---

## Summary

Phase 4 adds a full data layer to the Rotiv framework: a TypeScript ORM package (`@rotiv/orm`) built on Drizzle ORM, SQLite (dev) and PostgreSQL (prod) drivers, schema migration via `drizzle-kit`, and `db` injection into every loader and action context. The architecture follows the same Rust → Node subprocess delegation pattern established in Phase 3.

---

## Packages Added

### `packages/@rotiv/orm` (new, public)
- **`src/types.ts`** — `ModelDefinition<T>`, `DrizzleInstance` (SQLite | PG union), `RotivDb`, `ModelRegistry` interfaces
- **`src/define-model.ts`** — `defineModel(name, table)` function that brands a Drizzle table as a Rotiv model; re-exports all Drizzle column helpers (`sqliteTable`, `pgTable`, `text`, `integer`, `varchar`, `serial`, `sql`, `eq`, `and`, `or`, etc.) so route files need only one import
- **`src/db.ts`** — `createDb({ databaseUrl?, projectDir? })` async factory: SQLite path (`better-sqlite3` + WAL mode, auto-creates `app/.rotiv/`) or PG path (`pg.Pool(max:5)`) depending on `DATABASE_URL`
- **`src/registry.ts`** — `globalModelRegistry` singleton (`InMemoryModelRegistry`)
- **`src/index.ts`** — re-exports all public API including `BetterSQLite3Database` and `NodePgDatabase` type re-exports for route-file type narrowing
- `skipLibCheck: true` added to workspace `tsconfig.base.json` to suppress drizzle-orm's internal declaration errors for unused dialects (MySQL, SingleStore, Gel)

### `packages/@rotiv/migrate-script` (new, private)
- Mirrors `@rotiv/build-script` pattern: spawned by Rust as `node --import tsx <script_path> --project <dir> [--generate|--migrate|--check|--introspect]`
- **`src/drizzle-config.ts`** — writes `drizzle.config.ts` to project root at runtime with correct schema/out/dialect/credentials
- **`src/runner.ts`** — `generateMigrations()`, `applyMigrations()`, `checkPending()`, `introspectModels()`; drizzle-kit is invoked with `shell: true` and `NODE_OPTIONS=--import tsx` so TypeScript schema files (with `.js` imports) load correctly
- **`src/index.ts`** — CLI entry, parses `--generate | --migrate | --check | --introspect`, prints JSON to stdout

---

## Packages Modified

### `packages/@rotiv/types`
- **`src/db.ts`** (new) — `RotivDb` interface with `_driver`, `drizzle: unknown`, `query<T>()` — no drizzle-orm import, keeps `@rotiv/types` dependency-free
- **`src/framework.ts`** — `LoaderContext` and `ActionContext` gain `readonly db: RotivDb`
- **`src/index.ts`** — exports `RotivDb`

### `packages/@rotiv/route-worker`
- **`src/db.ts`** (new) — module-scoped `initDb(projectDir)` / `getDb()`; calls `createDb()` from `@rotiv/orm`; handles `DATABASE_URL` env for PG
- **`src/index.ts`** — calls `await initDb(projectDir)` before `app.listen()`; reads `ROTIV_PROJECT_DIR` env (set by Rust worker spawner)
- **`src/invoke.ts`** — `buildContext()` injects `db: getDb()` into context
- **`src/transform.ts`** — extended `rewriteImports()` to also handle **relative imports** (`.` prefix): resolves to the original route file's directory, then recursively calls `transformAndCache()` on `.ts` sources so all transitive TypeScript files are compiled before Node imports them from the cache dir
- **`package.json`** — added `@rotiv/orm: workspace:*` and `@rotiv/types: workspace:*` deps

---

## Crates Added/Modified

### `crates/rotiv-orm` (significantly extended from stub)
- **`src/error.rs`** — full `OrmError` enum: `NotImplemented`, `ScriptNotFound`, `SpawnFailed`, `MigrationFailed`, `ParseFailed`, `PendingMigrations(u32)`, `Io(#[from] io::Error)`
- **`src/discovery.rs`** (new) — `discover_models(project_dir)` → `Vec<ModelFileEntry>`; globs `app/models/*.ts`; converts snake_case filenames to PascalCase names; returns empty Vec if directory absent
- **`src/migration.rs`** (new) — `MigrationOptions`, `MigrationResult`, `run_migrations()`, `auto_migrate()`, `resolve_migrate_script_path()`; spawns `node --import tsx <migrate_script> --project <dir> [mode]`; parses JSON stdout; maps `pending` field from check output; resolves script via env `ROTIV_MIGRATE_SCRIPT_PATH` → dev monorepo path → production path
- **`src/lib.rs`** — exposes all new modules and public API
- **`Cargo.toml`** — added `serde` and `serde_json` workspace deps

### `crates/rotiv-core`
- **`src/models.rs`** (new) — `discover_models(project_dir)` → `Vec<ModelEntry>` using `RotivError`
- **`src/lib.rs`** — exposes `discover_models`, `ModelEntry`
- **`src/server.rs`** — imports `rotiv_orm::auto_migrate`; after worker ready: runs auto-migrate if `app/models/` exists (non-fatal warning on error); prints model count
- **`Cargo.toml`** — added `rotiv-orm` path dep

### `crates/rotiv-cli`
- **`src/cli.rs`** — added `Migrate { generate_only: bool, check: bool }` command variant
- **`src/commands/migrate.rs`** (new) — calls `find_project_root()`, checks for `app/models/` dir, calls `run_migrations()`, prints human/JSON output
- **`src/commands/mod.rs`** — added `pub mod migrate`
- **`src/main.rs`** — dispatches `Commands::Migrate`
- **`src/commands/new.rs`** — added `MODEL_USER_TS` constant via `include_str!`; writes `app/models/user.ts` instead of `.gitkeep`
- **`Cargo.toml`** — added `rotiv-orm` path dep

---

## Template Updates

### `templates/default/app/models/user.ts` (new)
Canonical example model with:
- Raw `users` table export (required by drizzle-kit for schema discovery)
- `UserModel = defineModel("User", users)` wrapper for Rotiv runtime
- `User` and `NewUser` type exports
- Detailed FRAMEWORK comments

### `templates/default/package.json`
- Added `@rotiv/orm: "^0.1.0"` to `dependencies`
- Added `drizzle-kit: "^0.30.0"` to `devDependencies`

---

## E2E Test: `e2e-test-phase4`

New workspace member. Route `app/routes/index.tsx` uses:
```typescript
const ping = await ctx.db.query<{ n: number }>("SELECT 1 as n");
const db = ctx.db.drizzle as BetterSQLite3Database;
const userRows = await db.select().from(users);
```

**Verified checks:**
- ✅ `rotiv migrate --generate-only` → `.rotiv/migrations/0000_*.sql` created, exit 0
- ✅ `rotiv migrate` → `app/.rotiv/dev.db` created, 1 migration applied, exit 0
- ✅ `rotiv migrate --check` → "0 pending migration(s)", exit 0
- ✅ `rotiv migrate --check --json` → valid JSON with `ok: true`
- ✅ `rotiv dev` → worker starts, auto-migrate "up to date", "1 model(s) found", HTTP 200
- ✅ `curl http://127.0.0.1:3005/` → HTML with `DB ping: 1` and `Users: []`
- ✅ `ctx.db.query("SELECT 1 as n")` → `[{ n: 1 }]`
- ✅ `ctx.db.drizzle as BetterSQLite3Database` → type-safe Drizzle queries
- ✅ Missing `db` on context → TypeScript compile error (required by `LoaderContext`)
- ✅ `e2e-test-phase3` still serves JSX HTML unchanged (backward compat)
- ✅ `cargo test --workspace` → 44 tests pass (0 failures)
- ✅ `pnpm -r typecheck` → all packages pass

---

## Key Bugs Fixed

### `better-sqlite3` native build on Windows
`better-sqlite3` has no prebuilt binary for Node 22 on Windows. Required adding `"better-sqlite3"` to `pnpm.onlyBuiltDependencies` in root `package.json` (done in Wave 1 setup) and compiling from source via MSBuild + Python.

### drizzle-kit schema loading with TypeScript models
`drizzle-kit` uses CJS `require()` internally to load schema files. Rotiv model files import from `@rotiv/orm` (ESM with `.js` extensions), causing `MODULE_NOT_FOUND`. Fix: set `NODE_OPTIONS=--import tsx` in the environment when spawning drizzle-kit, allowing `tsx` to intercept TypeScript imports.

### Relative imports in transform cache
Route files importing `../models/user.js` fail when the cached `.mjs` is in the OS temp dir (wrong relative base). Fix: extended `rewriteImports()` in `transform.ts` to recursively `transformAndCache()` all relative `.ts` imports, pointing the cache entry to the compiled absolute `file://` URL.

### `checkPending` missing `duration_ms`
The Rust parser expected `MigrateScriptOutput` with `duration_ms` but the check mode response only had `{ pending, ok }`. Fix: added `duration_ms` to `PendingResult` interface and tracked elapsed time in `checkPending()`.

### `e2e-test-phase3` tsconfig broken extends
`tsconfig.json` extended `@rotiv/types/tsconfig.base.json` which doesn't exist. Fixed to `../tsconfig.base.json`.

---

## Design Decisions

| ID | Decision |
|----|----------|
| D16 | **Drizzle ORM** — TypeScript-native DSL for both SQLite and PG; `drizzle-kit` for migrations |
| D17 | **`defineModel()` wrapper** — brands Drizzle table as `ModelDefinition`; raw table also exported for drizzle-kit schema discovery |
| D18 | **Single DB connection per worker** — `initDb()` once at startup; SQLite WAL mode; `app/.rotiv/dev.db` |
| D19 | **`@rotiv/migrate-script` subprocess** — mirrors `@rotiv/build-script`; drizzle-kit spawned with `NODE_OPTIONS=--import tsx` |
| D20 | **Auto-migrate on `rotiv dev`** — `--check` (journal read only, no subprocess) then `--migrate` if pending > 0 |
| D21 | **Model discovery in Rust** — filesystem glob in `rotiv-orm::discovery` and `rotiv-core::models`; no Node subprocess |
| D22 | **`RotivDb.drizzle: unknown`** — `@rotiv/types` stays dep-free; route files cast `ctx.db.drizzle as BetterSQLite3Database` imported from `@rotiv/orm` |
