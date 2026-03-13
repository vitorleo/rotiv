# Phase 2 — Core Runtime & `rotiv dev`

## Summary

Delivered a working `rotiv dev` command: axum HTTP server that discovers routes from `app/routes/`, proxies requests to a Node.js route-worker process, serves HTML responses, and watches for file changes.

**End-to-end result:** `rotiv dev` in a new project → `curl http://127.0.0.1:3000/` → HTTP 200 with rendered HTML.

## What was built

### `rotiv-core` — new modules

- **`router/discovery.rs`** — `discover_routes(dir)` walks `app/routes/` and maps file paths to HTTP routes:
  - `index.tsx` → `/`
  - `about.tsx` → `/about`
  - `users/[id].tsx` → `/users/:id`
  - `_`-prefixed files skipped (private)
- **`router/matcher.rs`** — `matches(pattern, path)` extracts dynamic params from a matched route.
- **`router/registry.rs`** — `RouteRegistry` with `load()`, `reload()`, `find_by_path()`, `extract_params()`. Shared as `Arc<RwLock<RouteRegistry>>`.
- **`proxy.rs`** — `invoke_route(client, worker_port, InvokeRequest) -> InvokeResponse`. POSTs to the route-worker. HTTP 500 from worker is deserialized as a structured `RotivError`.
- **`worker.rs`** — `RouteWorker` spawns `node --import tsx <worker_path>`. `wait_ready()` polls `/_rotiv/health` every 100ms up to 15s. `resolve_worker_path()` checks env var → dev layout → production layout.
- **`watcher.rs`** — `FileWatcher` wraps `notify_debouncer_mini`. Uses `PollWatcher` (500ms) on Windows or when `ROTIV_FORCE_POLL=1`. Exposes `try_recv()` (non-blocking) for use with async code.
- **`server.rs`** — `DevServer::start()` orchestrates the full startup sequence:
  1. Discover routes and print table
  2. Spawn Node.js route-worker, wait for health check
  3. Build axum router with single catch-all handler
  4. Spawn watcher task (reload registry + restart worker on file change)
  5. Register Ctrl+C handler (kills worker, exits cleanly)
  6. Bind axum and serve

### `rotiv-cli` — new command

- **`commands/dev.rs`** — Walks up from cwd to find `.rotiv/spec.json` (project root). Builds `DevServerConfig`. Runs `DevServer` on a single-threaded tokio runtime.
- **`cli.rs`** — Added `Dev { port, host }` variant. Default host changed to `127.0.0.1` (avoids Windows IPv6 `::1` bind issue).

### `@rotiv/route-worker` — new TypeScript package

Internal Express server (`private: true`, never published) that executes TypeScript route files on demand:

- **`src/index.ts`** — Express on `ROTIV_WORKER_PORT`. `GET /_rotiv/health` and `POST /_rotiv/invoke`.
- **`src/invoke.ts`** — Cache-busting dynamic `import()` via `pathToFileURL()` (required for Windows absolute paths in Node ESM). Calls loader or action, renders component.
- **`src/render.ts`** — Phase 2 shim: `renderToString` returns the string directly; `wrapHtml` wraps in a full HTML5 document.
- **`src/errors.ts`** — `toRotivError(err, routeFile)` parses stack traces to extract file + line number.

### Template & SDK updates

- `templates/default/app/routes/index.tsx` — `component()` now returns an HTML template literal string (no JSX until Phase 3).
- `templates/default/package.json` — Added `tsx` as devDependency.
- `packages/@rotiv/sdk/src/server.ts` — Updated stub message to `"Use rotiv dev to start the development server."`.

## Bugs fixed during verification

| Bug | Root cause | Fix |
|-----|-----------|-----|
| Server binds but curl connection refused | `localhost` → IPv6 `::1` on Windows; axum binds `::1` but curl hits `127.0.0.1` | Changed default host to `127.0.0.1` |
| Axum accepts connection but never responds | `recv_timeout()` in watcher loop blocks the single tokio thread | Replaced with `try_recv()` + `tokio::time::sleep()` |
| Worker invoke fails on Windows paths | Node ESM requires `file://` URLs; raw `C:/...` paths are rejected | Added `pathToFileURL()` in `invoke.ts` |
| Proxy fails to parse worker response | Proxy expected JSON-wrapped `InvokeResponse`; worker sends raw HTTP | Proxy now reads status/headers/body directly from the HTTP response |

## Key decisions

- **D7** — Subprocess architecture (axum + Node.js worker) instead of napi-rs. Avoids node-gyp/C++ toolchain on Windows.
- **D8** — `tsx` for on-the-fly TypeScript execution in the route-worker.
- **D9** — `notify` with `PollWatcher` fallback; watcher loop uses async sleep to avoid blocking the tokio runtime.
- **D10** — `component()` returns HTML strings in Phase 2; JSX deferred to Phase 3 (SWC compiler).

## Test results

```
cargo test --workspace   →  34 passed, 0 failed
pnpm --filter @rotiv/route-worker typecheck  →  pass
pnpm --filter @rotiv/sdk typecheck           →  pass
curl http://127.0.0.1:3000/                  →  HTTP 200, "Hello from <project>!"
```
