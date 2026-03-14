# Rotiv Phase 3: Compiler & Bundler

## Context

Phase 2 delivered `rotiv dev`: axum server + Node.js route-worker (tsx) + file watcher. Components return HTML strings (Phase 2 limitation — no JSX compiler). Phase 3 delivers JSX support in `rotiv dev`, a signal primitive library, and the `rotiv build` command.

**Goal:** Route files with JSX syntax → `rotiv dev` → `curl http://127.0.0.1:3000/` → rendered HTML. Plus `rotiv build` compiles the project to `dist/`.

---

## Key Design Decisions

### D11: `@rotiv/jsx-runtime` — custom Rotiv JSX factory (not React)
JSX transform target is `@rotiv/jsx-runtime`, not React. TypeScript compiles `<h1>` → `jsx("h1", ...)` via `jsxImportSource: "@rotiv"`. The `jsx()` function builds a VNode tree; `renderToString()` serializes it to HTML on the server. This is the foundation for Phase 4 fine-grained DOM updates.

### D12: `@swc/core` npm in the route-worker — no Rust SWC crates
Phase 3 avoids adding `swc_core` (30+ Rust crates) to the workspace to protect compile times on the Intel N150. Instead, `@swc/core` npm runs inside the route-worker process. `invoke.ts` calls it to transform `.tsx` files before dynamic import. This replaces relying on `tsx`'s esbuild JSX defaults, which do not support `jsxImportSource`.

### D13: Transform-cache pattern for JSX in `rotiv dev`
The route-worker transforms route source → writes compiled `.mjs` to `os.tmpdir()/rotiv-cache/<sha1(filepath+mtime)>.mjs` → imports that. Cache key includes mtime so stale transforms are never served. The `?t=Date.now()` cache-bust on the import URL forces Node ESM re-evaluation. Phase 2 backward compat is preserved: string-returning components still work unchanged.

### D14: `@rotiv/signals` — SSR-only in Phase 3
`signal()`, `derived()`, `effect()` are pure TypeScript with zero Rust dependency. SSR behavior: compute once, no subscriptions. Phase 4 adds client-side DOM subscriptions via a separate `@rotiv/signals/client` export path.

### D15: `rotiv build` delegates to a Node.js build script
The `rotiv-compiler` Rust crate spawns a Node.js child process (`packages/@rotiv/build-script`) that uses `@swc/core` to transform route files to `dist/`. No Rust SWC crates. The crate adds only `serde_json` (already in workspace) — zero new compile overhead.

---

## File Tree (additions/modifications only)

```
packages/@rotiv/
  jsx-runtime/                          [NEW package]
    package.json
    tsconfig.json
    src/
      types.ts                          [NEW] VNode, Props, Children, JSX namespace
      jsx-runtime.ts                    [NEW] jsx, jsxs, Fragment
      jsx-dev-runtime.ts                [NEW] jsxDEV
      render.ts                         [NEW] renderToString(VNode) → string
      index.ts                          [NEW] re-exports

  signals/                              [NEW package]
    package.json
    tsconfig.json
    src/
      types.ts                          [NEW] Getter, Setter, Disposer, SignalPair
      signal.ts                         [NEW] signal<T>(initial) → [get, set]
      derived.ts                        [NEW] derived<T>(fn) → getter (memoized)
      effect.ts                         [NEW] effect(fn) → disposer (runs once SSR)
      index.ts                          [NEW] re-exports

  build-script/                         [NEW internal package, private]
    package.json
    tsconfig.json
    src/
      compiler.ts                       [NEW] SWC transform loop over app/routes/
      manifest.ts                       [NEW] write dist/manifest.json
      index.ts                          [NEW] CLI: parse --project --out --minify

  route-worker/
    package.json                        [MODIFY] add @swc/core, @rotiv/jsx-runtime
    src/
      transform.ts                      [NEW] transformAndCache(filePath) → .mjs path
      invoke.ts                         [MODIFY] call transformAndCache before import()
      render.ts                         [MODIFY] dispatch to jsx-runtime renderToString

crates/
  rotiv-compiler/
    Cargo.toml                          [MODIFY] add serde, serde_json
    src/
      lib.rs                            [MODIFY] CompileOptions, CompileResult, compile_project()
      error.rs                          [MODIFY] add SpawnFailed, BuildFailed, ScriptNotFound

  rotiv-cli/
    Cargo.toml                          [MODIFY] add rotiv-compiler dependency
    src/
      cli.rs                            [MODIFY] add Build { out, minify } variant
      commands/
        mod.rs                          [MODIFY] add pub mod build
        build.rs                        [NEW] run() → find_project_root + compile_project

templates/default/
  tsconfig.json                         [MODIFY] add jsxImportSource: "@rotiv"
  package.json                          [MODIFY] add @rotiv/signals, @rotiv/jsx-runtime
  app/routes/index.tsx                  [MODIFY] use JSX syntax (not HTML string)
```

---

## Implementation Waves

### Wave 1 — `@rotiv/jsx-runtime`
Pure TypeScript, no external runtime deps.

**`src/types.ts`:**
- `VNode = { type: string | typeof Fragment, props: Props, key: string | null }`
- `Props = Record<string, unknown> & { children?: Children }`
- `Children = VNode | VNode[] | string | number | boolean | null | undefined`
- `export namespace JSX` (not `declare global`) — avoids conflicts with `@types/react`

**`src/jsx-runtime.ts`:**
```typescript
export const Fragment = Symbol.for("rotiv.Fragment");
export function jsx(type, props, key?): VNode { return { type, props, key: key ?? null }; }
export const jsxs = jsx;
```

**`src/render.ts` — `renderToString(node: unknown): string`:**
- `null | undefined | false` → `""`
- `string | number` → HTML-escape (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`)
- `array` → join each item recursively
- `Fragment` → render children only
- Function type → call with `node.props`, recurse on result
- HTML tag string → `<tag attr="val">children</tag>` or self-closing for void elements
- Props rules: `children` → body; `className` → `class`; `htmlFor` → `for`; boolean attrs; `style` object → CSS string; event handlers (`on*`) → omit; `dangerouslySetInnerHTML` → raw inner HTML

**Void elements:** `area br col embed hr img input link meta param source track wbr`

**`tsconfig.json`:** `"jsx": "react-jsx", "jsxImportSource": "@rotiv"`, `rootDir: "src"`

**package.json exports map:**
```json
{
  ".": "./src/index.ts",
  "./jsx-runtime": "./src/jsx-runtime.ts",
  "./jsx-dev-runtime": "./src/jsx-dev-runtime.ts"
}
```

Typecheck: `pnpm --filter @rotiv/jsx-runtime typecheck`

---

### Wave 2 — Route-worker JSX support

**`package.json` additions:**
```json
{ "dependencies": { "@swc/core": "^1.7.0", "@rotiv/jsx-runtime": "workspace:*" } }
```

**New `src/transform.ts`:**
```typescript
import { transform } from "@swc/core";
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const CACHE_DIR = join(tmpdir(), "rotiv-transform-cache");

export async function transformAndCache(filePath: string): Promise<string> {
  const mtime = (await stat(filePath)).mtimeMs;
  const key = createHash("sha1")
    .update(filePath + String(mtime))
    .digest("hex")
    .slice(0, 16);
  const outPath = join(CACHE_DIR, `${key}.mjs`);

  // Re-use cached output if mtime hasn't changed
  try {
    await stat(outPath);
    return outPath;  // cache hit
  } catch { /* cache miss — transform */ }

  const source = await readFile(filePath, "utf8");
  const result = await transform(source, {
    filename: filePath,
    jsc: {
      parser: { syntax: "typescript", tsx: true },
      transform: { react: { runtime: "automatic", importSource: "@rotiv" } },
      target: "es2022",
    },
    module: { type: "es6" },
    sourceMaps: false,
  });

  await mkdir(CACHE_DIR, { recursive: true });
  await writeFile(outPath, result.code);
  return outPath;
}
```

**`src/invoke.ts` change** — replace the `pathToFileURL` import block:
```typescript
// before: const fileUrl = pathToFileURL(req.route_file); ...
// after:
const cachedPath = await transformAndCache(req.route_file);
const fileUrl = pathToFileURL(cachedPath);
fileUrl.searchParams.set("t", String(Date.now()));
routeModule = await import(fileUrl.href);
```

**`src/render.ts` replacement:**
```typescript
import { renderToString as jsxRenderToString } from "@rotiv/jsx-runtime";

export function renderToString(component, props): string {
  if (!component) return "";
  const result = component(props);
  // VNode from jsx-runtime
  if (result !== null && typeof result === "object" && "type" in result && "props" in result) {
    return jsxRenderToString(result);
  }
  // Phase 2 backward compat: plain string
  if (typeof result === "string") return result;
  return JSON.stringify(result);
}

export function wrapHtml(body: string, title = "Rotiv"): string { /* unchanged */ }
```

Typecheck: `pnpm --filter @rotiv/route-worker typecheck`

---

### Wave 3 — `@rotiv/signals`

SSR-only Phase 3 implementation. All three primitives execute synchronously, no side effects.

```typescript
// signal.ts
export function signal<T>(initial: T): [() => T, (v: T | ((p: T) => T)) => void] {
  let v = initial;
  return [() => v, (next) => { v = typeof next === "function" ? next(v) : next; }];
}

// derived.ts — memoize once
export function derived<T>(fn: () => T): () => T {
  const v = fn();
  return () => v;
}

// effect.ts — run once, return disposer
export function effect(fn: () => void | (() => void)): () => void {
  const cleanup = fn();
  return typeof cleanup === "function" ? cleanup : () => {};
}
```

Typecheck: `pnpm --filter @rotiv/signals typecheck`

---

### Wave 4 — Template updates

**`templates/default/tsconfig.json`** — add:
```json
"jsxImportSource": "@rotiv"
```

**`templates/default/package.json`** — add:
```json
{
  "dependencies": { "@rotiv/signals": "^0.1.0" },
  "devDependencies": { "@rotiv/jsx-runtime": "^0.1.0" }
}
```

**`templates/default/app/routes/index.tsx`:**
```tsx
import { defineRoute } from "@rotiv/sdk";
// FRAMEWORK: Phase 3 — JSX syntax supported. Component returns JSX compiled
// by @swc/core to @rotiv/jsx-runtime calls during rotiv dev.
export default defineRoute({
  path: "/",
  async loader() {
    return { message: "Hello from {{project_name}}!" };
  },
  component({ data }) {
    return (
      <main>
        <h1>{data.message}</h1>
        <p>Edit <code>app/routes/index.tsx</code> to get started.</p>
      </main>
    );
  },
});
```

Also check `packages/@rotiv/create/` to see if it hardcodes `package.json` or `tsconfig.json` content (read before modifying).

---

### Wave 5 — `@rotiv/build-script` + `rotiv-compiler` + `rotiv build`

**`packages/@rotiv/build-script/src/compiler.ts`** — discover `app/routes/**/*.{ts,tsx}`, transform each with `@swc/core` (minify when `--minify` flag set), write to `dist/server/routes/`.

**`packages/@rotiv/build-script/src/index.ts`** — parse `--project`, `--out`, `--minify` from `process.argv`, call `compileRoutes()`, print JSON result to stdout:
```json
{"files": ["dist/server/routes/index.mjs"], "warnings": [], "duration_ms": 423}
```

**`crates/rotiv-compiler/src/error.rs`** — add variants:
```rust
SpawnFailed(String), BuildFailed(String), ScriptNotFound(String), ParseFailed(String)
```

**`crates/rotiv-compiler/src/lib.rs`** — implement:
```rust
pub struct CompileOptions { pub project_dir: PathBuf, pub out_dir: PathBuf, pub minify: bool, pub source_maps: bool }
pub struct CompileResult { pub files_written: Vec<PathBuf>, pub warnings: Vec<String>, pub duration_ms: u64 }
pub fn compile_project(options: CompileOptions) -> Result<CompileResult, CompilerError>
```
`compile_project` spawns `node --import tsx <build_script_path>` with `--project`/`--out`/`--minify` args. Resolves build script path: (1) `ROTIV_BUILD_SCRIPT_PATH` env, (2) `<binary_dir>/../../packages/@rotiv/build-script/src/index.ts` (dev), (3) `<binary_dir>/build-script/index.ts` (prod).

**`crates/rotiv-cli/src/commands/build.rs`** — `run(out: Option<PathBuf>, minify: bool, mode: OutputMode)`. Calls `find_project_root()` (extract to `rotiv_core::project` so `dev.rs` and `build.rs` share it), builds `CompileOptions`, calls `compile_project()`, prints human or JSON output.

**`crates/rotiv-cli/src/cli.rs`** — add:
```rust
Build {
  #[arg(short, long)] out: Option<PathBuf>,
  #[arg(long)] minify: bool,
},
```

---

## Acceptance Criteria

- [ ] `cargo test --workspace` — 0 errors (all Phase 1+2 tests pass)
- [ ] `pnpm --filter @rotiv/jsx-runtime typecheck` — pass
- [ ] `pnpm --filter @rotiv/signals typecheck` — pass
- [ ] `pnpm --filter @rotiv/route-worker typecheck` — pass
- [ ] `pnpm --filter @rotiv/build-script typecheck` — pass
- [ ] `rotiv build --help` exits 0
- [ ] `rotiv dev` with a JSX component route → `curl http://127.0.0.1:3000/` → HTTP 200, HTML contains rendered JSX tags
- [ ] Phase 2 string-returning component still renders correctly (backward compat)
- [ ] `rotiv build` in a test project → exits 0, `dist/server/routes/index.mjs` exists
- [ ] `renderToString(<h1 class="x">&amp;</h1>)` → `<h1 class="x">&amp;</h1>` (no double-escape)
- [ ] `renderToString(<br />)` → `<br>` (void element, no closing tag)

---

## Critical Files

| File | Change |
|------|--------|
| [packages/@rotiv/route-worker/src/invoke.ts](packages/@rotiv/route-worker/src/invoke.ts) | Add `transformAndCache` call before `import()` |
| [packages/@rotiv/route-worker/src/render.ts](packages/@rotiv/route-worker/src/render.ts) | Replace with VNode-aware dispatcher |
| [crates/rotiv-compiler/src/lib.rs](crates/rotiv-compiler/src/lib.rs) | Implement `compile_project()` |
| [crates/rotiv-cli/src/cli.rs](crates/rotiv-cli/src/cli.rs) | Add `Build` command variant |
| [templates/default/app/routes/index.tsx](templates/default/app/routes/index.tsx) | Use JSX syntax |
| [templates/default/tsconfig.json](templates/default/tsconfig.json) | Add `jsxImportSource` |

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| `@swc/core` native binary on Windows | Pin to `^1.7.0`; verify with `node -e "require('@swc/core').version"` after install |
| `tsx` loader conflicts with `.mjs` cache files | Cache files always written with `.mjs` extension — `tsx` ignores them |
| JSX namespace conflict with `@types/react` | Use `export namespace JSX` (module-scoped), not `declare global { namespace JSX }` |
| Stale transform cache on file edit | Cache key = `sha1(path + mtime)` — new content always produces a new key |
| `find_project_root` duplicated in dev.rs and build.rs | Extract to `rotiv_core::project::find_root()` in Wave 5 |
