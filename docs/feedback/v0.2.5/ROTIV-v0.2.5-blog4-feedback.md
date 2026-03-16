# Rotiv v0.2.5 — AI Agent Experience Report (blog4)

**Date:** 2026-03-16
**Agent:** GitHub Copilot (GPT-5.3-Codex)
**Task:** Build a blog webapp (blog4)

---

## Summary

Project scaffolding works and dependency installation works after pnpm is available, but `rotiv build` fails in a standalone generated project with:

`Build script not found. Set ROTIV_BUILD_SCRIPT_PATH or run from the monorepo`

This suggests the CLI build flow currently expects a build script path that is not configured in a fresh app created by `rotiv new`.

---

## Step-by-Step Results

### ✓ Download rotiv binary (macOS arm64) — works

Downloaded release asset from v0.2.5 and verified:

```bash
./bin/rotiv-macos-arm64 --version
# rotiv 0.2.5
```

### ✓ `rotiv new blog4` — works

Project scaffolded successfully at `blog4/` with:

- `.rotiv/spec.json`
- `.rotiv/context.md`
- `app/routes/`
- `app/models/`
- `package.json`
- `README.md`

### ✓ `pnpm install` — works (after pnpm installation)

Initially, pnpm was missing in the environment (`command not found: pnpm`).
After installing pnpm globally, dependency install succeeded and generated:

- `node_modules/`
- `pnpm-lock.yaml`

Notable install output:

- optional `@rotiv/*` packages were skipped for compatibility
- pnpm warned about ignored build scripts (`better-sqlite3`, `esbuild`)

### ✗ `rotiv build` fails in blog4

**Actual error:**

```text
error [E_UNKNOWN] build failed: Build script not found.
Set ROTIV_BUILD_SCRIPT_PATH or run from the monorepo
```

No `dist/` directory was produced.

---

## Root Cause Hypothesis

The generated app includes a `build` script in `package.json` (`rotiv build`), but the framework CLI appears to require additional build-script context (`ROTIV_BUILD_SCRIPT_PATH`) unless executed inside the framework monorepo.

In other words, `rotiv new` produced a project that cannot be built out-of-the-box with `rotiv build` in this environment.

---

## Suggested Fix Options for Framework

1. Ensure `rotiv new` writes any required build script file/path metadata so `rotiv build` works immediately.
2. Make `rotiv build` auto-resolve its build script when run inside a generated app.
3. Improve error guidance to include exact setup steps for standalone projects (if monorepo execution is intentional).
4. Add a CI smoke test: `rotiv new <name> && pnpm install && rotiv build` on macOS/Linux/Windows.

---

## Blog4 Files Created

All app code is in `blog4/`.

Key generated files/folders:

- `README.md`
- `package.json`
- `.rotiv/spec.json`
- `.rotiv/context.md`
- `app/models/`
- `app/routes/`

---

## How To Try Locally

```bash
cd blog4
pnpm install
pnpm build
pnpm dev
# open http://localhost:3000
```

If `pnpm build` fails with the same message, this confirms the current framework build-path issue for standalone generated apps.
