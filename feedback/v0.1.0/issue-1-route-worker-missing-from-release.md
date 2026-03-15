# [Bug] `rotiv dev` fails with `E_WORKER_NOT_FOUND` — `@rotiv/route-worker` not bundled in release binary

**Labels:** `bug`, `dx`

## Environment
- OS: Windows 11 Pro (x64)
- CLI: `rotiv-windows-x64.exe` v0.1.0 (from GitHub Releases)
- Project: scaffolded with `rotiv new todo-app`

## Steps to reproduce
```bash
rotiv new todo-app
cd todo-app
rotiv dev
```

## Actual output
```
  Rotiv dev server
  GET  /  →  app/routes/index.tsx
  GET  /todos/:id  →  app/routes/[id].tsx
error [E_WORKER_NOT_FOUND] cannot locate @rotiv/route-worker entry point
  hint: Set ROTIV_WORKER_PATH=/path/to/packages/@rotiv/route-worker/src/index.ts
```

## Expected behavior
`rotiv dev` starts a working development server without requiring the user to point at internal monorepo source files.

## Root cause (hypothesis)
The `@rotiv/route-worker` TypeScript entry point is expected either at a hardcoded monorepo path **or** via `ROTIV_WORKER_PATH`. The standalone release binary doesn't bundle this worker or ship it alongside the executable.

## Suggested fix
Either:
1. Embed the compiled route worker as a static asset inside the binary (e.g. via `include_bytes!` or similar in Rust), or
2. Ship a `rotiv-worker.js` file alongside the binary in the release tarball and resolve it relative to the executable path, or
3. Document the `ROTIV_WORKER_PATH` workaround prominently in the README and in the `rotiv dev --help` output.

## Impact
**Blocker** — `rotiv dev` (the primary development workflow) is completely non-functional from a standalone release install.
