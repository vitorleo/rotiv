# Rotiv Phase 7: Polish & Distribution

## Context

Phase 6 delivered the module system (capability-based middleware, three-tier architecture, first-party modules: auth, sessions, file-uploads). Phase 7 closes out the implementation spec with distribution infrastructure, the deploy command, reference apps, and agent platform integrations.

---

## Deliverables

1. **`rotiv deploy`** — SSH deploy to Linux VPS (copy binary + run migrations + restart systemd service)
2. **GitHub Actions release workflow** — builds cross-platform binaries (Linux x86_64, macOS arm64, Windows x64) and publishes them as GitHub Releases
3. **`install.sh`** — curl-pipe-sh installer for Linux/macOS
4. **Reference app: todo** — fully annotated CRUD app using routes, models, modules
5. **Agent platform integrations** — MCP server definition + tool manifest JSON
6. **`rotiv explain deploy`** — 10th knowledge topic
7. **Extended CI** — add release workflow on version tags

---

## Key Design Decisions

### D37: `rotiv deploy` — pure SSH via std subprocess
No SSH crates (libssh2, russh). Spawns `ssh` and `scp` as subprocesses. This keeps the binary small, avoids complex async I/O, and means the user's local SSH key/agent is used automatically. The deploy config lives in `.rotiv/deploy.json` (gitignored host/user/path). If the file doesn't exist, deploy reads flags from CLI (`--host`, `--user`, `--path`).

### D38: Release binary names
`rotiv-linux-x86_64`, `rotiv-macos-arm64`, `rotiv-windows-x64.exe`. Published as GitHub Release assets. The install script detects platform and downloads the correct one.

### D39: GitHub Actions release workflow — separate from CI
`release.yml` triggers on `v*` tags. Builds Linux in the GA runner (ubuntu-latest), macOS on macos-latest, Windows on windows-latest. Uses `cargo build --release`. Uploads artifacts then creates a GitHub Release with `gh release create`.

### D40: Reference app "todo"
Lives at `reference-apps/todo/`. A complete working Rotiv app: index route (list todos), new todo form (action), todo detail. Uses a `Todo` model. Annotated with `// EXAMPLE:` comments explaining every pattern. Has `.rotiv/spec.json` pre-populated.

### D41: MCP server — static JSON tool manifest
`packages/@rotiv/mcp/index.json` — a static MCP tool manifest listing all CLI commands as tools. No executable needed for the manifest itself. Also `packages/@rotiv/mcp/server.ts` — a thin Node.js MCP server that spawns `rotiv` subprocesses and returns structured JSON.

### D42: `rotiv deploy` deploy.json config format
```json
{
  "host": "YOUR_SERVER_IP",
  "user": "root",
  "remote_path": "/opt/rotiv-apps/myapp",
  "service_name": "myapp",
  "binary_path": "dist/server"
}
```

---

## File Tree (additions/modifications only)

```
.github/workflows/
  ci.yml                              [MODIFY] add clippy lint step
  release.yml                         [NEW] cross-platform release builds on v* tags

install.sh                            [NEW] curl-pipe-sh installer

crates/rotiv-cli/src/
  cli.rs                              [MODIFY] Add Deploy command
  main.rs                             [MODIFY] Dispatch Deploy
  commands/
    mod.rs                            [MODIFY] pub mod deploy
    deploy.rs                         [NEW] SSH deploy via subprocess

  knowledge/
    deploy.md                         [NEW] deploy explanation

  commands/explain.rs                 [MODIFY] add "deploy" topic (10th), update count

reference-apps/
  todo/
    package.json
    tsconfig.json
    .rotiv/spec.json
    app/
      routes/
        index.tsx          # list todos + form
        todos/[id].tsx     # todo detail
      models/
        todo.ts            # Todo model with status field

packages/@rotiv/
  mcp/
    package.json
    index.json             # MCP tool manifest
    server.ts              # thin MCP server (spawns rotiv CLI)
    tsconfig.json
```

---

## Implementation Waves

### Wave 1 — `rotiv deploy` command

**`.rotiv/deploy.json`** (created by `rotiv deploy --init`):
```json
{
  "host": "YOUR_SERVER_IP",
  "user": "root",
  "remote_path": "/opt/rotiv-apps/myapp",
  "service_name": "myapp"
}
```

**`crates/rotiv-cli/src/commands/deploy.rs`**:
```rust
pub fn run(host: Option<&str>, user: Option<&str>, path: Option<&str>,
           service: Option<&str>, init: bool, dry_run: bool,
           mode: OutputMode) -> Result<(), CliError>
```

Steps:
1. `--init`: write `.rotiv/deploy.json` template, exit
2. Read config from `.rotiv/deploy.json` (merge with CLI flags)
3. `rotiv build` (unless `--skip-build`)
4. `scp dist/server <user>@<host>:<remote_path>/server`
5. `ssh <user>@<host> "cd <remote_path> && ./server migrate && sudo systemctl restart <service_name>"`
6. Report success/failure with structured output

**`crates/rotiv-cli/src/cli.rs`** addition:
```rust
Deploy {
    #[arg(long)] host: Option<String>,
    #[arg(long)] user: Option<String>,
    #[arg(long, name = "path")] remote_path: Option<String>,
    #[arg(long)] service: Option<String>,
    #[arg(long)] init: bool,
    #[arg(long)] dry_run: bool,
    #[arg(long)] skip_build: bool,
}
```

### Wave 2 — Release workflow + install.sh

**`.github/workflows/release.yml`**:
- Trigger: `push: tags: ['v*']`
- Matrix: `[ubuntu-latest, macos-latest, windows-latest]`
- Steps: checkout → sccache → cargo build --release → rename binary → upload artifact → create release

**`install.sh`**:
- Detect OS (uname -s) and arch (uname -m)
- Map to binary name (rotiv-linux-x86_64, rotiv-macos-arm64, etc.)
- Download from GitHub releases latest
- chmod +x, move to /usr/local/bin
- Print version

### Wave 3 — Reference app: todo

Fully annotated, working Rotiv todo app at `reference-apps/todo/`. Every file has `// EXAMPLE:` comments. Pre-seeded spec.json.

### Wave 4 — MCP server package

**`packages/@rotiv/mcp/index.json`** — static MCP tool manifest with all 12 CLI commands.

**`packages/@rotiv/mcp/server.ts`** — reads stdin for JSON-RPC requests, dispatches to `rotiv` subprocess with `--json`, returns response.

### Wave 5 — Knowledge + docs + changelog

- `rotiv explain deploy` — 10th topic
- Update explain.rs test count to 10
- `changelog/phase7.md`

---

## Acceptance Criteria

```bash
# Deploy command
rotiv deploy --init                  # creates .rotiv/deploy.json, exit 0
rotiv deploy --dry-run               # prints steps without executing, exit 0
rotiv deploy --help                  # shows all flags, exit 0

# Release workflow
# .github/workflows/release.yml exists and is valid YAML

# install.sh
cat install.sh | bash --norc         # (dry run — just parse check)
bash -n install.sh                   # syntax check, exit 0

# Reference app
ls reference-apps/todo/app/routes/   # index.tsx, todos/[id].tsx
ls reference-apps/todo/app/models/   # todo.ts

# MCP
cat packages/@rotiv/mcp/index.json   # valid JSON with tools array
node --input-type=module < packages/@rotiv/mcp/server.ts  # loads without error

# Knowledge
rotiv explain deploy                 # Markdown output, exit 0
rotiv explain deploy --json          # JSON with explanation field

# All tests
cargo test --workspace               # pass
```
