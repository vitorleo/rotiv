# Phase 7: Polish & Distribution

## Overview

Phase 7 closes the Rotiv implementation spec. It adds `rotiv deploy` (SSH-based VPS deployment), cross-platform release builds via GitHub Actions, a curl-pipe installer, an annotated reference app (todo), an MCP server package for agent platform integrations, and a 10th knowledge topic (`deploy`).

---

## New Command: `rotiv deploy`

Deploys the compiled project to a Linux VPS via SSH and SCP.

### Workflow

```
1. Build         rotiv build          (skip with --skip-build)
2. Upload        scp dist/server <user>@<host>:<remote_path>/server
3. Restart       ssh <user>@<host> "cd <path> && ./server migrate && sudo systemctl restart <service>"
```

### Setup

```bash
# Create .rotiv/deploy.json config template
rotiv deploy --init
# → .rotiv/deploy.json created (edit with your server details)

# Deploy (full pipeline)
rotiv deploy

# Preview commands without executing
rotiv deploy --dry-run --skip-build

# Override config at deploy time
rotiv deploy --host 1.2.3.4 --user deploy --path /srv/app --service myapp

# JSON output (for CI/agent use)
rotiv deploy --skip-build --json
# → { "ok": true, "host": "...", "remote_path": "...", "service": "...", "dry_run": false }
```

### `.rotiv/deploy.json` format

```json
{
  "host": "YOUR_SERVER_IP",
  "user": "root",
  "remote_path": "/opt/rotiv-apps/myapp",
  "service_name": "myapp"
}
```

### Error codes

| Code | Meaning |
|------|---------|
| E020 | Build failed before deploy |
| E021 | SCP upload failed (SSH connectivity) |
| E022 | Remote command failed (migrations or service restart) |
| E023 | Host not configured |
| E024 | Remote path not configured |

---

## GitHub Actions Release Workflow

`.github/workflows/release.yml` — triggers on `v*` tags.

**Build matrix:**

| OS | Target | Artifact |
|----|--------|---------|
| ubuntu-latest | x86_64-unknown-linux-gnu | `rotiv-linux-x86_64` |
| macos-latest | aarch64-apple-darwin | `rotiv-macos-arm64` |
| windows-latest | x86_64-pc-windows-msvc | `rotiv-windows-x64.exe` |

Uses sccache on all platforms. Creates a GitHub Release with `softprops/action-gh-release` and attaches all three binaries.

---

## `install.sh` — Curl-pipe Installer

```bash
curl -fsSL https://github.com/rotiv-dev/rotiv/releases/latest/download/install.sh | bash
```

- Detects platform (Linux/macOS) and architecture (x86_64/arm64/aarch64)
- Downloads the correct binary from the latest GitHub Release
- Installs to `/usr/local/bin/rotiv` (uses sudo if needed)
- Verifies installation by running `rotiv --version`
- Reports an error for unsupported platforms (Windows: points to manual download)

---

## Reference App: `reference-apps/todo`

A fully annotated CRUD todo application demonstrating all core Rotiv patterns.

**Files:**
- `app/models/todo.ts` — `Todo` model with `status` enum field, `$defaultFn` for timestamps
- `app/routes/index.tsx` — list todos + create form; loader + action + component
- `app/routes/todos/[id].tsx` — todo detail + status toggle; dynamic segment; `ctx.params.id`
- `.rotiv/spec.json` — pre-populated with routes and model

Every file has `// EXAMPLE:` comments explaining each pattern:
- How to use `ctx.db.drizzle` in a loader
- Drizzle `select`, `insert`, `update` with `where` clauses
- Post/Redirect/Get pattern after successful actions
- Dynamic segments (`ctx.params.id`)
- Type inference from the Drizzle schema (`typeof todos.$inferSelect`)

---

## `@rotiv/mcp` — Agent Platform Integration

MCP (Model Context Protocol) server enabling AI agents to use the Rotiv CLI as structured tools.

**`packages/@rotiv/mcp/index.json`** — static MCP tool manifest listing all 12 CLI commands:
`rotiv_new`, `rotiv_info`, `rotiv_add_route`, `rotiv_add_model`, `rotiv_add_module`,
`rotiv_spec_sync`, `rotiv_validate`, `rotiv_explain`, `rotiv_context_regen`,
`rotiv_diff_impact`, `rotiv_migrate`, `rotiv_deploy`

Each tool has a typed `inputSchema` matching the CLI flags.

**`packages/@rotiv/mcp/src/server.ts`** — Node.js MCP server:
- Reads newline-delimited JSON-RPC 2.0 from stdin
- Handles `initialize`, `tools/list`, `tools/call` methods
- Dispatches tool calls to the `rotiv` binary via `spawnSync` with `--json`
- Returns structured JSON responses

**Usage in MCP client config:**
```json
{
  "mcpServers": {
    "rotiv": {
      "command": "node",
      "args": ["/path/to/@rotiv/mcp/dist/server.js"],
      "cwd": "/path/to/your/rotiv/project"
    }
  }
}
```

---

## Extended `rotiv explain`

New topic: **`deploy`** (10th topic)

```bash
rotiv explain deploy
rotiv explain deploy --json
```

Covers: deploy config format, step-by-step workflow, dry-run usage, prerequisites, example systemd unit file.

Total topics: **10** (routes, models, loader, action, middleware, signals, migrate, context, modules, deploy)

---

## CI Improvements

Added `cargo clippy --workspace -- -D warnings` step to `.github/workflows/ci.yml`. Clippy runs after `cargo test` on every push/PR to `main`.

---

## File Tree (additions/modifications)

```
.github/workflows/
  ci.yml                              [MODIFIED] added cargo clippy step
  release.yml                         [NEW] cross-platform release builds on v* tags

install.sh                            [NEW] curl-pipe-sh installer

crates/rotiv-cli/
  Cargo.toml                          [MODIFIED] added tempfile dev-dependency
  src/
    cli.rs                            [MODIFIED] Deploy command + updated Explain help text
    main.rs                           [MODIFIED] Deploy dispatch
    commands/
      mod.rs                          [MODIFIED] pub mod deploy
      deploy.rs                       [NEW] SSH deploy via scp/ssh subprocesses
    knowledge/
      deploy.md                       [NEW] deploy explanation
    commands/explain.rs               [MODIFIED] "deploy" 10th topic, count 10

reference-apps/
  todo/
    package.json                      [NEW]
    tsconfig.json                     [NEW]
    .rotiv/spec.json                  [NEW]
    app/models/todo.ts                [NEW] annotated Todo model
    app/routes/index.tsx              [NEW] annotated list + create route
    app/routes/todos/[id].tsx         [NEW] annotated detail + update route

packages/@rotiv/
  mcp/
    package.json                      [NEW]
    tsconfig.json                     [NEW]
    index.json                        [NEW] MCP tool manifest (12 tools)
    src/server.ts                     [NEW] JSON-RPC MCP server

pnpm-workspace.yaml                   [MODIFIED] added reference-apps/todo
```

---

## Verification

```
✓ rotiv deploy --help                      → shows all flags, exit 0
✓ rotiv deploy --init                      → .rotiv/deploy.json created, exit 0
✓ rotiv deploy --init (duplicate)          → E010 error, exit 1
✓ rotiv deploy --dry-run --skip-build      → prints steps, exit 0
✓ rotiv deploy --dry-run --json            → structured JSON, exit 0
✓ rotiv explain deploy                     → Markdown, exit 0
✓ rotiv explain deploy --json              → JSON with explanation + code_example
✓ cat packages/@rotiv/mcp/index.json       → valid JSON, 12 tools
✓ bash -n install.sh                       → syntax valid, exit 0
✓ cargo test --workspace                   → 61 tests pass, 0 failures
✓ cargo build --workspace                  → clean build, no warnings
```

---

## All Phases Complete

| Phase | Feature |
|-------|---------|
| 1 | Foundation — monorepo, `rotiv new`, spec.json |
| 2 | Core Runtime — axum server, file-system routing, `rotiv dev` |
| 3 | Compiler & Bundler — SWC, `rotiv build`, signals |
| 4 | Data Layer — Drizzle ORM, SQLite/PG, `rotiv migrate`, `ctx.db` |
| 5 | Agent Tooling — `rotiv explain/validate/diff-impact/spec-sync/context-regen` |
| 6 | Module System — capabilities, `rotiv add module`, first-party: auth/sessions/file-uploads |
| **7** | **Polish & Distribution — `rotiv deploy`, release CI, install.sh, reference app, MCP** |
