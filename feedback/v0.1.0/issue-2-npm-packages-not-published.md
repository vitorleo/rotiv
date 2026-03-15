# [Bug] `@rotiv/*` packages not published to npm — `pnpm install` fails for scaffolded projects

**Labels:** `bug`, `dx`, `release`

## Environment
- OS: Windows 11 Pro (x64)
- CLI: `rotiv-windows-x64.exe` v0.1.0
- Node: v18+, pnpm installed

## Steps to reproduce
```bash
rotiv new todo-app
cd todo-app
pnpm install
```

## Actual output
```
ERR_PNPM_FETCH_404  GET https://registry.npmjs.org/@rotiv%2Ftypes: Not Found - 404
```

All `@rotiv/*` scoped packages (`@rotiv/types`, `@rotiv/sdk`, `@rotiv/orm`, `@rotiv/signals`, `@rotiv/jsx-runtime`) are missing from npm.

## Expected behavior
`pnpm install` succeeds and all framework dependencies are available.

## Impact
**Blocker** — Without the npm packages, TypeScript type checking fails, IDE autocomplete doesn't work, and `rotiv migrate` (which shells out to `drizzle-kit`) cannot run.

## Suggested fix
Publish `@rotiv/*` packages to npm as part of the v0.1.0 release, or:
- Use a GitHub Package Registry and document the `.npmrc` setup needed, or
- Bundle all `@rotiv/*` type declarations into the release (similar to how the CLI binary is distributed), or
- At minimum, add a clear notice in the README that npm packages are not yet published and explain the workaround.
