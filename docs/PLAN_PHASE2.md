# Rotiv Phase 2: Core Runtime

## Context
Phase 1 delivered the monorepo scaffold, `rotiv new`, `rotiv info`, and TypeScript SDK type stubs. Phase 2 delivers a working `rotiv dev` command: an axum HTTP server that discovers routes from `app/routes/*.tsx`, proxies requests to a Node.js route-worker process, and watches for file changes.

**Goal:** `rotiv dev` in a new project → `curl http://localhost:3000/` → HTML response with the route's content.

---

## Key Design Decisions

### D7: Subprocess architecture instead of napi-rs
napi-rs requires node-gyp, a C++ toolchain, and cross-compilation config. On Windows 11 + WSL2 this is a multi-hour debugging risk with no user-visible benefit over a subprocess approach.

**Architecture:** Rust axum server (port 3000) + Node.js route-worker (port 3001, internal only). On each HTTP request, axum resolves the route file, POSTs to `localhost:3001/_rotiv/invoke` with `{ routeFile, method, params, headers, body }`, gets back `{ status, headers, body }`, and forwards the response. Phase 3 replaces this with the SWC compiler.

### D8: tsx as the Phase 2 TypeScript executor
The route-worker uses `tsx` (npm package) to `import()` `.tsx` route files without a compile step. Single dependency, handles TypeScript/JSX, standard in 2025. Replaced by Rotiv's own SWC compiler in Phase 3.

### D9: notify with polling fallback
WSL2 inotify only fires for writes done from inside WSL2. VS Code writes from the Windows side → inotify never fires. Use `notify`'s `RecommendedWatcher` as primary; fall back to `PollWatcher` (500ms interval) when on Windows or when `ROTIV_FORCE_POLL=1` is set.

### D10: Phase 2 routes return HTML strings, not JSX
No JSX compiler exists until Phase 3. Update `component()` signature to return `string`. The route-worker's `renderToString` shim simply returns the string directly. JSX syntax (`<h1>`) will be supported in Phase 3.

---

## File Tree (additions only)

```
Cargo.toml                                    [MODIFY] add axum, tower-http, notify, reqwest to workspace deps
crates/
  rotiv-core/
    Cargo.toml                                [MODIFY] add axum, tower-http, tokio, notify, reqwest, anyhow
    src/
      lib.rs                                  [MODIFY] add pub mod server, router, proxy, watcher, worker
      server.rs                               [NEW] DevServer, DevServerConfig, AppState, axum setup
      proxy.rs                                [NEW] InvokeRequest/Response, invoke_route() via reqwest
      watcher.rs                              [NEW] FileWatcher, WatchEvent, polling fallback
      worker.rs                               [NEW] RouteWorker — spawn/kill/wait_ready Node.js child
      router/
        mod.rs                                [NEW]
        discovery.rs                          [NEW] discover_routes() — file path → HTTP path mapping
        matcher.rs                            [NEW] route_to_axum_path(), sort key for registration order
        registry.rs                           [NEW] RouteRegistry with Arc<RwLock> for concurrent access
  rotiv-cli/
    Cargo.toml                                [MODIFY] add tokio = { workspace = true }
    src/
      cli.rs                                  [MODIFY] add Dev { port, host } variant to Commands enum
      commands/
        mod.rs                                [MODIFY] add pub mod dev
        dev.rs                                [NEW] run() — find project root, build config, block_on DevServer

packages/@rotiv/
  route-worker/                               [NEW package — private, never published]
    package.json                              express, @types/express, tsx, @types/node, typescript
    tsconfig.json
    src/
      index.ts                                [NEW] Express server on ROTIV_WORKER_PORT; /health + /invoke
      invoke.ts                               [NEW] dynamic import route file, call loader/action
      render.ts                               [NEW] renderToString shim (string passthrough for Phase 2)
      errors.ts                               [NEW] wrap JS errors → RotivError JSON shape with file+line

  sdk/src/server.ts                           [MODIFY] update stub error message only

templates/default/app/routes/index.tsx        [MODIFY] component() returns HTML string, not JSX
templates/default/package.json                [MODIFY] add tsx as devDependency

tests/phase2/
  hello_world.test.sh                         [NEW] end-to-end curl test
```

---

## Implementation Waves

### Wave 1 — Workspace dependencies (no compilation)
Modify root `Cargo.toml` workspace deps:
```toml
axum = { version = "0.8", features = ["json"] }
tower-http = { version = "0.6", features = ["trace", "cors"] }
notify = { version = "7" }
notify-debouncer-mini = { version = "0.5" }
reqwest = { version = "0.12", features = ["json"] }
```
Run `cargo fetch` to populate sccache. Run `cargo check --workspace` to confirm no conflicts.

**Version compatibility check:** axum 0.8 + tower 0.5 + reqwest 0.12 are compatible. Verify with `cargo tree -p rotiv-core` after adding deps.

### Wave 2 — Router module (pure data, no HTTP)
Build `crates/rotiv-core/src/router/` — file path → HTTP route mapping. No server, no Node.js. Unit-testable in isolation.

**File-to-route mapping rules:**
| File | HTTP path |
|------|-----------|
| `index.tsx` | `/` |
| `about.tsx` | `/about` |
| `users/index.tsx` | `/users` |
| `users/[id].tsx` | `/users/:id` |
| `api/users.ts` | `/api/users` |

`RouteRegistry` uses `Arc<RwLock<Vec<RouteEntry>>>` so axum handlers can read while the watcher reloads.

**Unit tests to write:**
- `index.tsx` → `/`
- `[id].tsx` → `/:id`
- empty directory → empty vec, no error
- `find_by_path("/users/42")` matches `/users/:id` entry

### Wave 3 — Worker and Proxy
`worker.rs`: `RouteWorker` struct — spawns `node --import tsx <worker_path>` with `ROTIV_WORKER_PORT` and `ROTIV_PROJECT_DIR` env vars. `wait_ready()` polls `GET /_rotiv/health` every 100ms up to 10s timeout.

**Worker path resolution order:**
1. `ROTIV_WORKER_PATH` env var
2. `<binary_dir>/../../packages/@rotiv/route-worker/src/index.ts` (dev layout)
3. `<binary_dir>/route-worker/index.ts` (production layout)
4. Error: `E_WORKER_NOT_FOUND` with suggestion to set `ROTIV_WORKER_PATH`

`proxy.rs`: `invoke_route(worker_port, InvokeRequest) -> Result<InvokeResponse, RotivError>`. Single shared `Arc<reqwest::Client>` for connection pooling. HTTP 500 from worker → deserialize `RotivError` and return as `Err`.

**Unit test:** `InvokeRequest` serializes to expected JSON shape.

### Wave 4 — File Watcher
`watcher.rs`: `FileWatcher::new(dir)` returns a struct with `recv_timeout()`. Uses `notify_debouncer_mini` with 200ms debounce. Filters to `.tsx`/`.ts` only. Ignores `.rotiv/` subdirectory.

On Windows or `ROTIV_FORCE_POLL=1`: use `notify::PollWatcher` with 500ms interval.

**Unit test:** Watcher constructs without error (can test with a temp dir).

### Wave 5 — HTTP Server (critical wave)
`server.rs`: Single catch-all axum handler (`/*path` + `/` exact match). Handler reads `RouteRegistry` (read lock), calls `invoke_route()`, returns axum `Response`.

`AppState`:
```rust
#[derive(Clone)]
struct AppState {
    registry: Arc<RwLock<RouteRegistry>>,
    worker_port: u16,
    client: Arc<reqwest::Client>,
}
```

**Start sequence:**
1. `registry.load()` — discover routes, print table
2. `worker.start()` + `worker.wait_ready(10s)` — start Node.js
3. Build axum router with `AppState`
4. Spawn watcher task: on file change → kill + restart worker; on new/deleted file → reload registry + restart worker
5. `axum::serve(TcpListener::bind(...), router).await`

**Startup output (human mode):**
```
  Rotiv dev server starting...
  GET /  →  app/routes/index.tsx
  worker ready on :3001
  listening on http://localhost:3000
  watching app/routes/ (Ctrl+C to stop)
```

**JSON mode events** (one JSON object per line to stdout):
```json
{"event":"routes_discovered","routes":[{"path":"/","file":"app/routes/index.tsx"}]}
{"event":"worker_ready","worker_port":3001}
{"event":"listening","url":"http://localhost:3000"}
{"event":"file_changed","file":"app/routes/index.tsx"}
```

Ctrl+C: register handler to kill child process before exit (use `tokio::signal::ctrl_c()`).

### Wave 6 — CLI dev command
`crates/rotiv-cli/src/cli.rs` — add to `Commands`:
```rust
Dev {
    #[arg(short, long, default_value = "3000")]
    port: u16,
    #[arg(long, default_value = "localhost")]
    host: String,
},
```

`commands/dev.rs`:
- Walk up from cwd to find `.rotiv/spec.json` → project root
- Build `DevServerConfig { port, host, project_dir, worker_port: port + 1 }`
- Create single-threaded tokio runtime (`Builder::new_current_thread()`)
- `rt.block_on(DevServer::new(config).start())`
- `find_project_root()` returns `CliError::Rotiv(E_NOT_A_PROJECT)` if not found

**CLI tests:**
- `rotiv dev --help` exits 0
- `rotiv dev` outside project → `E_NOT_A_PROJECT` error
- `rotiv dev --json` outside project → structured JSON error to stderr

### Wave 7 — route-worker TypeScript package
New internal package `packages/@rotiv/route-worker` (`private: true`).

`src/index.ts`: Express server on `process.env.ROTIV_WORKER_PORT ?? 3001`.
- `GET /_rotiv/health` → `{ ok: true }`
- `POST /_rotiv/invoke` → calls `invokeRoute(req.body)`, returns result or error

`src/invoke.ts`:
```typescript
const module = await import(req.routeFile + `?t=${Date.now()}`);
// cache-bust each request so edits are picked up within a session
const route = module.default; // must be RouteDefinition with _type brand
// call loader() or action() based on method
// call renderToString(route.component, { data })
```

`src/render.ts`: Phase 2 shim — if result is string, return as-is; otherwise `JSON.stringify`.

`src/errors.ts`: `toRotivError(err, routeFile)` — parse stack trace to extract file+line, return `RotivError`-shaped object.

### Wave 8 — Template and SDK updates
`templates/default/app/routes/index.tsx`: `component()` returns template literal string (no JSX). Add `// FRAMEWORK:` comment explaining Phase 2 limitation.

`templates/default/package.json`: add `"tsx": "^4.0.0"` to devDependencies.

`packages/@rotiv/sdk/src/server.ts`: update stub message to `"Use rotiv dev to start the development server."`.

### Wave 9 — End-to-end test + verification
`tests/phase2/hello_world.test.sh`:
1. `rotiv new test-phase2-$$`
2. `cd` + `pnpm install`
3. Start `rotiv dev &`, capture PID
4. Poll `curl -s http://localhost:3000/` for up to 15s
5. Assert HTTP 200 + body contains `Hello from test-phase2`
6. Kill PID, verify no orphan `node` processes
7. `rm -rf` test project

---

## Critical Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add axum, tower-http, notify, reqwest to workspace deps |
| `crates/rotiv-core/Cargo.toml` | Reference new workspace deps |
| `crates/rotiv-core/src/lib.rs` | Add pub mod for all new modules |
| `crates/rotiv-cli/src/cli.rs` | Add `Dev` variant to `Commands` |
| `crates/rotiv-cli/src/commands/mod.rs` | Add `pub mod dev` |
| `templates/default/app/routes/index.tsx` | String return, no JSX |

---

## Acceptance Criteria

- [ ] `cargo check --workspace` — 0 errors, 0 warnings
- [ ] `cargo test --workspace` — all Phase 1 tests still pass + new Phase 2 unit tests
- [ ] `rotiv dev --help` exits 0
- [ ] `rotiv dev` outside a project → `E_NOT_A_PROJECT` structured error
- [ ] `rotiv dev --json` outside a project → JSON error to stderr
- [ ] `rotiv new hello && cd hello && pnpm install && rotiv dev` → server starts, prints route table
- [ ] `curl http://localhost:3000/` → HTTP 200, body contains `Hello from hello`
- [ ] Edit `app/routes/index.tsx` → within 2s, curl returns updated response
- [ ] Ctrl+C → clean shutdown, no orphan node processes
- [ ] `pnpm --filter @rotiv/route-worker typecheck` passes
- [ ] `pnpm --filter @rotiv/sdk typecheck` passes (still)
- [ ] `DECISIONS.md` updated with D7–D10

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| WSL2 file watching silent | Polling fallback on by default on Windows; `ROTIV_FORCE_POLL=1` override |
| axum/tower version conflicts | Pin axum 0.8 + tower 0.5 + reqwest 0.12; check with `cargo tree` before coding |
| Node.js process leak on Ctrl+C | `tokio::signal::ctrl_c()` handler explicitly kills child PID |
| tsx module cache on edits | Cache-bust dynamic import with `?t=Date.now()`; watcher also restarts worker |
| Port 3001 already in use | Try `port+1`, `port+2`, `port+3` in sequence; log chosen port |
| First build time (~8-12 min for axum+reqwest) | Expected; use `cargo check` during development; sccache handles subsequent builds |
