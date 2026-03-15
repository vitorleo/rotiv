# [Bug] `rotiv migrate` fails outside monorepo — `ROTIV_MIGRATE_SCRIPT_PATH` not documented

**Labels:** `bug`, `dx`

## Environment
- OS: Windows 11 Pro (x64)
- CLI: `rotiv-windows-x64.exe` v0.1.0

## Steps to reproduce
```bash
rotiv new todo-app
cd todo-app
rotiv migrate --generate-only
```

## Actual output
```json
{"error":{"code":"E_UNKNOWN","message":"migration error: Script not found: Set ROTIV_MIGRATE_SCRIPT_PATH or run from the Rotiv monorepo","file":null,"line":null,"expected":null,"got":null,"suggestion":null,"corrected_code":null}}
```

## Expected behavior
`rotiv migrate` generates migration files using the bundled drizzle-kit integration, without requiring the user to clone the Rotiv monorepo or set internal environment variables.

## Root cause
The migration command shells out to a TypeScript script (`drizzle-kit generate`) that must be resolved via `ROTIV_MIGRATE_SCRIPT_PATH` or a hardcoded monorepo path. The standalone binary doesn't bundle this script.

## Note
`rotiv explain migrate` correctly documents the feature but gives no hint about this limitation. The error message exposes an internal implementation detail (`ROTIV_MIGRATE_SCRIPT_PATH`) that end users have no way to satisfy.

## Suggested fix
Bundle the migration runner script as a compiled asset inside the binary, similar to the worker issue (#1). Alternatively, generate a `drizzle.config.ts` in the project root during `rotiv new` so users can run `pnpm drizzle-kit generate` directly as a fallback.
