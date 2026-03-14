# Phase 5 Changelog — Agent Tooling

**Date:** 2026-03-14
**Status:** Complete

---

## Summary

Phase 5 adds the agent-facing tooling layer to the Rotiv framework CLI. All features are implemented as pure Rust subcommands in `rotiv-cli`, with no new TypeScript packages and no new Rust crate dependencies. The result is a set of structured CLI commands that AI coding agents can use to query project state, validate edits, scaffold annotated files, and understand the framework.

---

## New CLI Commands

### `rotiv add route <path>`
Scaffolds an annotated route file at `app/routes/<path>.tsx`. The generated file includes `// FRAMEWORK:` comments explaining every convention (path matching, dynamic segments, loader return type, JSX).

- Handles dynamic segments: `users/[id]` → file `app/routes/users/[id].tsx`, path `/users/:id`
- Handles nested paths: `admin/settings` → `app/routes/admin/settings.tsx`
- Refuses to overwrite existing files (error E010)
- Template embedded in binary via `include_str!`

### `rotiv add model <Name>`
Scaffolds an annotated model file at `app/models/<snake>.ts`. The generated file includes the two-export pattern required by both drizzle-kit and Rotiv's runtime registry.

- Validates PascalCase name
- Derives table name via snake_case pluralization
- Prints follow-up instructions: `rotiv migrate --generate-only` + `rotiv migrate`

### `rotiv spec-sync`
Syncs `.rotiv/spec.json` with current filesystem state. Discovers all routes in `app/routes/**/*.tsx` and models in `app/models/*.ts`. Overwrites `routes` and `models` arrays; preserves all other spec fields.

Route entries include `has_loader`, `has_action`, `has_component` flags (detected via non-comment line scanning to avoid matching template comments).

Model entries include `name`, `file`, and `table` (snake_case plural of the model name).

### `rotiv validate [--fix]`
Runs 7 static-analysis checks against all route and model files. Returns structured diagnostics with file, line, code, message, suggestion, and optional auto_fix.

| Code | Check | Severity | Fixable |
|------|-------|----------|---------|
| V001 | Route file missing `export default defineRoute` | Error | Yes |
| V002 | `defineRoute()` missing `component` field | Warning | No |
| V003 | Model file missing `sqliteTable()` or `pgTable()` export | Error | No |
| V004 | Model file missing `defineModel()` call | Error | No |
| V005 | Route has `export default { ... }` (raw object) instead of `defineRoute()` | Error | No |
| V006 | Route loader uses `ctx.db` (on non-comment line) but no `/models/` import | Warning | No |
| V007 | Route path string uses `[param]` bracket notation instead of `:param` | Error | No |

`--fix` applies auto-fixes for V001. Exits 1 if any errors remain after fixing.

JSON output: `{ "ok": bool, "error_count": N, "warning_count": M, "fixed": N, "diagnostics": [...] }`

### `rotiv explain <topic>`
Queries the built-in knowledge base. 8 topics embedded in binary via `include_str!`: `routes`, `models`, `loader`, `action`, `middleware`, `signals`, `migrate`, `context`.

Fuzzy matching: exact → prefix → contains. Human mode prints raw Markdown. JSON mode returns `{ topic, explanation, code_example, related }`.

Unknown topics return error E020 with list of valid topics.

### `rotiv context-regen`
Regenerates `.rotiv/context.md` with a structured project snapshot: Routes table (path, file, loader/action/component flags), Models table (name, table, file), Conventions, Quick Reference.

Header includes auto-generated timestamp and "Do not edit manually" warning.

### `rotiv diff-impact <file>`
Scans all route files for `import` lines referencing the target file's stem. Returns a list of affected routes with their paths and the matching import lines.

JSON output: `{ "target": "...", "affected_routes": [...], "total": N }`

---

## New Files

### `crates/rotiv-cli/src/templates/add/`
- **`route.tsx`** — annotated route template with `{{route_path}}` and `{{route_file_path}}` placeholders
- **`model.ts`** — annotated model template with `{{model_name}}` and `{{table_name}}` placeholders

### `crates/rotiv-cli/src/knowledge/`
8 Markdown knowledge files: `routes.md`, `models.md`, `loader.md`, `action.md`, `middleware.md`, `signals.md`, `migrate.md`, `context.md`

### `crates/rotiv-cli/src/commands/`
- **`add.rs`** — `run_add_route()`, `run_add_model()`, helpers (`derive_route_paths`, `to_snake`, `to_snake_plural`)
- **`spec_sync.rs`** — `run()`, `run_for_project()`, route walker, model discovery via `rotiv_orm::discover_models`
- **`validate.rs`** — calls `rotiv_core::run_diagnostics()` and `apply_fixes()`
- **`explain.rs`** — `TOPICS` const array, `parse_topic()` Markdown section parser
- **`context.rs`** — Markdown table builder, reads project name from spec.json
- **`diff_impact.rs`** — route file import scanner

### `crates/rotiv-core/src/analysis.rs`
New module with `Diagnostic`, `DiagnosticSeverity`, `run_diagnostics()`, `apply_fixes()`. Exported from `rotiv-core` lib root.

### `e2e-test-phase5/`
New workspace member with 2 routes (`index.tsx`, `users/[id].tsx`) and 1 model (`user.ts`).

---

## Modified Files

### `crates/rotiv-cli/src/cli.rs`
Added 6 new command variants: `Add(AddArgs)`, `SpecSync`, `Validate`, `Explain`, `ContextRegen`, `DiffImpact`. Added `AddArgs` struct and `AddSubcommand` enum.

### `crates/rotiv-cli/src/main.rs`
Dispatch for all 6 new commands.

### `crates/rotiv-cli/src/commands/mod.rs`
Added `pub mod` declarations for all 6 new command modules.

### `crates/rotiv-core/src/lib.rs`
Added `pub mod analysis` and exports: `Diagnostic`, `DiagnosticSeverity`, `apply_fixes`, `run_diagnostics`.

### `pnpm-workspace.yaml`
Added `e2e-test-phase5`.

---

## E2E Verification

All checks verified in `e2e-test-phase5/`:

- ✅ `rotiv add route products` → `app/routes/products.tsx` created with FRAMEWORK comments, exit 0
- ✅ `rotiv add route products` (again) → error E010 "file already exists", exit 1
- ✅ `rotiv add model Post` → `app/models/post.ts` created, exit 0
- ✅ `rotiv spec-sync` → spec.json has 3 routes, 2 models, exit 0
- ✅ `rotiv spec-sync --json` → valid JSON
- ✅ `rotiv validate` → 0 diagnostics on valid project, exit 0
- ✅ Bad route (missing `export default defineRoute`) → V001 error, exit 1
- ✅ `rotiv validate --fix` → V001 auto-fixed, exit 0
- ✅ `rotiv explain routes` → Markdown output, exit 0
- ✅ `rotiv explain routes --json` → JSON with `topic`, `explanation`, `code_example`, `related`
- ✅ `rotiv explain xyz` → error E020 with available topics list, exit 1
- ✅ `rotiv context-regen` → `.rotiv/context.md` with routes and models tables, exit 0
- ✅ `rotiv diff-impact app/models/user.ts --json` → lists index.tsx and users/[id].tsx as affected, exit 0
- ✅ `cargo test --workspace` → 53 tests pass (0 failures)
- ✅ `pnpm -r typecheck` → all 14 packages pass
- ✅ `e2e-test-phase4` validate, spec-sync, diff-impact all work unchanged (backward compat)

---

## Key Bugs Fixed

### Comment lines triggering V006 false positives
The route template includes `// ctx.db — database connection` as a comment. The V006 check was triggering on this comment. Fix: added non-comment line filter — only lines where `!l.trim().starts_with("//")` are checked for `ctx.db`.

### Comment lines inflating has_action/has_loader in spec-sync
The route template includes `// action() handles mutations` as a comment. The spec-sync content scan was detecting this as `has_action: true`. Fix: applied the same non-comment line filter when scanning for `loader(`, `action(`, `component(`.

### `rotiv spec sync` (with space) not recognized
Clap auto-hyphenates multi-word enum variants: `SpecSync` → `spec-sync`. The plan used `rotiv spec sync` but the correct form is `rotiv spec-sync`. Updated verification accordingly.

---

## Design Decisions

| ID | Decision |
|----|----------|
| D23 | `rotiv add` uses `Add { subcommand: AddSubcommand }` — mirrors Cargo's `cargo add` pattern |
| D24 | Templates compiled into binary via `include_str!` from `crates/rotiv-cli/src/templates/add/` |
| D25 | spec-sync overwrites only `routes`/`models` arrays, preserves all other spec fields |
| D26 | validate uses line-by-line `contains()` scan on non-comment lines — no AST parsing |
| D27 | explain embeds 8 Markdown topics; fuzzy match: exact → prefix → contains |
| D28 | context-regen is pure Rust; reads project name from spec.json `project.name` or `project_name` |
| D29 | diff-impact scans import lines for target filename stem — pure Rust string matching |
