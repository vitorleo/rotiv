# Rotiv v0.2.3 — Patch Plan

**Source:** v0.2.2 AI agent experience report (blog2, 2026-03-15)

---

## Root Cause

The only remaining blocker is that `tsx` cannot be resolved when the user runs `rotiv dev` or
`rotiv migrate` from a newly scaffolded project. This happens because:

1. `pnpm install` fails (all `@rotiv/*` packages return 404 from npm)
2. Without `pnpm install`, the project's `node_modules` is empty
3. `rotiv dev` and `rotiv migrate` invoke `node --import tsx` which resolves `tsx` from
   `node_modules` — which is empty — so Node crashes with `Cannot find package 'tsx'`

The framework's CLI tooling, scaffolding, validation, and spec tools are all confirmed working.

---

## Fix

### D46: `resolve_tsx_loader()` — find tsx by absolute path

Instead of passing the bare string `"tsx"` to `node --import`, search for tsx at known locations
and pass the absolute path to its ESM loader (`tsx/dist/esm/index.cjs`). Resolution order:

1. `<project_dir>/node_modules/tsx/dist/esm/index.cjs` — user has tsx installed (normal case)
2. `<binary_dir>/node_modules/tsx/dist/esm/index.cjs` — tsx bundled alongside rotiv binary
3. `<binary_dir>/../node_modules/tsx/dist/esm/index.cjs` — one level up from binary
4. Bare `"tsx"` — fallback for globally installed tsx

This function is implemented in both `rotiv-core/src/worker.rs` (for `rotiv dev`) and
`rotiv-orm/src/migration.rs` (for `rotiv migrate`).

### D47: install.sh checks for tsx and auto-installs it

The installer now:
- Checks that `node` is in PATH (hard requirement, exits 1 if missing)
- Checks that `tsx` is in PATH; if not, runs `npm install -g tsx`

### D48: Better error message when tsx/drizzle-kit output is empty

Error message now explicitly says "tsx or drizzle-kit may not be installed" with the fix command.

---

## Files Changed

| File | Change |
|------|--------|
| `crates/rotiv-core/src/worker.rs` | Add `resolve_tsx_loader()`, use it in `start()` |
| `crates/rotiv-orm/src/migration.rs` | Add `resolve_tsx_loader()`, use it in `run_migrations()` |
| `install.sh` | Check for node + auto-install tsx |
