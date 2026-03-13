# Phase 3 — JSX Compiler, Signals & `rotiv build`

## Summary

Delivered JSX support in `rotiv dev`, the `@rotiv/signals` primitive library, and the `rotiv build` command. Route files with JSX syntax are now compiled by `@swc/core` at request time and served as server-rendered HTML.

**End-to-end result:** `rotiv dev` with a JSX component route → `curl http://127.0.0.1:3000/` → HTTP 200 with rendered JSX tags. `rotiv build` compiles all routes to `dist/server/routes/*.mjs`.

---

## What was built

### `@rotiv/jsx-runtime` — new TypeScript package

Custom JSX factory and SSR renderer (not React):

- **`src/types.ts`** — `VNode`, `Props`, `Children`, `Fragment` symbol, module-scoped `JSX` namespace (avoids conflicts with `@types/react`).
- **`src/jsx-runtime.ts`** — `jsx(type, props, key?)`, `jsxs` alias. Builds a VNode tree.
- **`src/jsx-dev-runtime.ts`** — `jsxDEV` with extra source-map params for dev builds.
- **`src/render.ts`** — `renderToString(node)`:
  - `null | undefined | false` → `""`
  - `string | number` → HTML-escaped (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`)
  - Arrays → joined recursively
  - `Fragment` → renders children without wrapper
  - Function components → called with props, result recursed
  - HTML elements → `<tag attrs>children</tag>`; void elements self-close (`<br>`, `<img>`, etc.)
  - Prop remapping: `className` → `class`, `htmlFor` → `for`, boolean attrs, `style` object → CSS string, event handlers omitted, `dangerouslySetInnerHTML` → raw body
- **`package.json`** exports map: `.` → `src/index.ts`, `./jsx-runtime` → `src/jsx-runtime.ts`, `./jsx-dev-runtime` → `src/jsx-dev-runtime.ts`

### `@rotiv/signals` — new TypeScript package

SSR-only Phase 3 implementation (synchronous, no subscriptions):

- **`signal<T>(initial)`** — returns `[get, set]` tuple. `set` accepts value or updater function.
- **`derived<T>(fn)`** — computes once at call time, returns memoized getter.
- **`effect(fn)`** — runs `fn` once, returns cleanup disposer (no-op on SSR).

Phase 4 will add client-side DOM subscriptions via a separate `@rotiv/signals/client` export.

### `@rotiv/route-worker` — JSX support (Wave 2)

- **`src/transform.ts`** (new) — `transformAndCache(filePath)`:
  - Hashes `filePath + mtime` → `sha1[:16].mjs` in `os.tmpdir()/rotiv-transform-cache/`
  - Cache hit: returns existing path (mtime-stable = content-stable)
  - Cache miss: calls `@swc/core` with `{ jsx: { runtime: "automatic", importSource: "@rotiv" } }`, writes compiled `.mjs`
  - Rewrites all bare package specifiers in the output to absolute `file://` URLs (see bugs section)
- **`src/invoke.ts`** (modified) — calls `transformAndCache` before `import()`. Cache-busts import URL with `?t=Date.now()`.
- **`src/render.ts`** (replaced) — VNode-aware dispatcher: detects VNode via `type`/`props` fields → `jsxRenderToString`; falls back to plain string (Phase 2 backward compat).

### `@rotiv/build-script` — new internal package (private)

Node.js build runner invoked by `rotiv-compiler` as a child process:

- **`src/compiler.ts`** — `compileRoutes(options)`: glob `app/routes/**/*.{ts,tsx}`, SWC-transform each to `dist/server/routes/*.mjs`, optional minification.
- **`src/manifest.ts`** — `writeManifest()`: writes `dist/manifest.json` with file list, route paths, build timestamp.
- **`src/index.ts`** — CLI entry: parses `--project`, `--out`, `--minify`. Prints JSON to stdout: `{ files, warnings, duration_ms }`.

### `rotiv-compiler` — implemented (was stub)

- **`src/error.rs`** — Added `SpawnFailed`, `BuildFailed`, `ScriptNotFound`, `ParseFailed` variants.
- **`src/lib.rs`** — `CompileOptions`, `CompileResult`, `compile_project()`:
  - Spawns `node --import tsx <build_script_path>` with `--project`/`--out`/`--minify` args
  - Resolves build script: (1) `ROTIV_BUILD_SCRIPT_PATH` env, (2) `<binary>/../../packages/@rotiv/build-script/src/index.ts` (dev), (3) `<binary>/build-script/index.ts` (prod)
  - Parses JSON stdout into `CompileResult`
- **`Cargo.toml`** — added `serde`, `serde_json`

### `rotiv-core` — shared project root discovery

- **`src/project.rs`** (new) — `find_project_root()`: walks up from cwd until `.rotiv/spec.json` is found. Returns `E_NOT_A_PROJECT` with suggestion if not found.
- **`src/lib.rs`** — added `pub mod project; pub use project::find_project_root;`

### `rotiv-cli` — `rotiv build` command

- **`src/commands/build.rs`** (new) — `run(out, minify, mode)`: calls `find_project_root()`, builds `CompileOptions`, calls `compile_project()`, prints human or JSON output.
- **`src/commands/dev.rs`** — removed local `find_project_root`; now uses `rotiv_core::find_project_root`.
- **`src/cli.rs`** — added `Build { out: Option<PathBuf>, minify: bool }` variant.
- **`src/main.rs`** — dispatches `Commands::Build`.
- **`Cargo.toml`** — added `rotiv-compiler` dependency.

### Template updates (Wave 4)

- **`templates/default/tsconfig.json`** — added `"jsxImportSource": "@rotiv"`
- **`templates/default/package.json`** — added `@rotiv/jsx-runtime`, `@rotiv/signals` to dependencies
- **`templates/default/app/routes/index.tsx`** — component now returns JSX syntax
- **`packages/@rotiv/create/src/templates.ts`** — updated all hardcoded template strings to match

---

## Bugs fixed during verification

| Bug | Root cause | Fix |
|-----|-----------|-----|
| `tsx` not resolvable in route-worker subprocess | Worker inherited project dir as cwd; project has no `tsx` in `node_modules` | Set worker `current_dir` to route-worker package dir (`src/` → package root) |
| `@rotiv/jsx-runtime` not found in temp cache `.mjs` | Cached `.mjs` lives in `os.tmpdir()` where Node ESM cannot resolve bare scoped packages | `transform.ts` rewrites all bare specifiers to absolute `file://` URLs at transform time |
| `@rotiv/sdk` not in route-worker's `node_modules` | Route files import `@rotiv/sdk` but it's a project dep, not a worker dep | Added `createRequire(routeFilePath)` fallback resolver — resolves from the project dir |
| `@rotiv/sdk` named exports not visible (`defineRoute` missing) | No `"type": "module"` in `@rotiv/sdk/package.json` → tsx compiled as CJS, wrapping named exports in `default` | Added `"type": "module"` to `@rotiv/sdk/package.json` |
| TypeScript `TS2375` in `build-script/compiler.ts` | `minify: undefined` not assignable to `JsMinifyOptions` with `exactOptionalPropertyTypes` | Replaced with conditional spread: `...(options.minify ? { minify: ... } : {})` |
| TypeScript `TS2532` in `render.ts` | `rawKey[2]` is `string \| undefined` under `noUncheckedIndexedAccess` | Changed to `rawKey[2]?.toUpperCase() === rawKey[2]` |

---

## Key decisions

- **D11** — `@rotiv/jsx-runtime` as custom Rotiv JSX factory (not React). Module-scoped `JSX` namespace avoids `@types/react` conflicts.
- **D12** — `@swc/core` npm in the route-worker process (not Rust `swc_core` crates). Protects compile time on Intel N150 (avoids 30+ extra Rust crates).
- **D13** — Transform-cache pattern: `sha1(path + mtime)` key, cached `.mjs` in OS temp dir, import URL cache-busted with `?t=Date.now()`. Specifiers rewritten to absolute `file://` URLs at write time so the cached file is portable.
- **D14** — `@rotiv/signals` SSR-only in Phase 3: compute once, no subscriptions. Phase 4 adds `@rotiv/signals/client` with DOM binding.
- **D15** — `rotiv build` delegates to a Node.js child process (`@rotiv/build-script`). No Rust SWC crates. `rotiv-compiler` adds only `serde_json` (already in workspace).

---

## Test results

```
cargo test --workspace                         →  37 passed, 0 failed
pnpm --filter @rotiv/jsx-runtime typecheck     →  pass
pnpm --filter @rotiv/signals typecheck         →  pass
pnpm --filter @rotiv/route-worker typecheck    →  pass
pnpm --filter @rotiv/build-script typecheck    →  pass
rotiv build --help                             →  exit 0
rotiv dev  (JSX route)  → curl /              →  HTTP 200, <h1>Hello from e2e-test-phase3!</h1>
rotiv dev  (string route, Phase 2 compat)     →  HTTP 200, correct HTML
rotiv build                                    →  exit 0, dist/server/routes/index.mjs exists
```
