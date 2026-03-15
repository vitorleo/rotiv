# Rotiv v0.2.0 — Implementation Prompt

You are the lead developer for Rotiv, an AI-native full-stack web framework. Phases 1–7 are complete and a v0.1.0 binary has been released. An AI coding agent tested v0.1.0 by building a todo app from scratch and filed detailed feedback. Your task is to implement v0.2.0 which fixes every issue found.

---

## BACKGROUND

Rotiv is a Rust CLI + TypeScript packages monorepo. The CLI (`rotiv`) is a single binary built with Clap. It scaffolds projects, runs a dev server, validates code, and deploys via SSH. TypeScript packages under `packages/@rotiv/*` provide the SDK, types, ORM, signals, and JSX runtime that scaffolded projects depend on.

### Tech stack
- **Rust crates** (in `crates/`): `rotiv-cli` (binary), `rotiv-core` (server, router, worker), `rotiv-orm` (migrations, DB), `rotiv-compiler` (SWC bundler)
- **TypeScript packages** (in `packages/@rotiv/`): `sdk`, `types`, `orm`, `signals`, `jsx-runtime`, `route-worker`, `migrate-script`, `build-script`, `mcp`, `create`, `spec`
- **Patterns already established**: `include_str!()` for embedding templates/knowledge/module files in the binary; structured `RotivError` with `code`, `message`, `suggestion`, `expected`, `got` fields; `--json` flag on all commands

### Repository layout
```
crates/
  rotiv-cli/src/
    cli.rs              — Clap command definitions
    main.rs             — dispatch
    commands/            — one file per command (dev.rs, migrate.rs, add.rs, ...)
    templates/           — embedded templates for `rotiv add`
    knowledge/           — embedded .md files for `rotiv explain`
    modules/             — embedded first-party module files
  rotiv-core/src/
    server.rs            — dev server, route printing
    worker.rs            — route worker resolution
    router/discovery.rs  — filesystem route scanning
  rotiv-orm/src/
    migration.rs         — migration script resolution
packages/@rotiv/
  route-worker/          — TypeScript route handler (private, internal)
  migrate-script/        — TypeScript migration runner (private, internal)
  sdk/                   — public: defineRoute, defineModel API
  types/                 — public: shared type definitions
  orm/                   — public: Drizzle re-exports, defineModel
  signals/               — public: signal(), derived(), effect()
  jsx-runtime/           — public: JSX factory
  mcp/                   — public: MCP server for agent integrations
```

---

## THE PROBLEM

v0.1.0 works inside the monorepo but is **completely non-functional as a standalone install**. Three blockers and two minor issues were found:

### Blocker 1: `rotiv dev` fails — route worker not bundled
`crates/rotiv-core/src/worker.rs` resolves the route worker via monorepo-relative path traversal (`../../packages/@rotiv/route-worker/src/index.ts`). When running from a standalone binary, this path doesn't exist. The production fallback (`<binary_dir>/route-worker/index.ts`) is also never populated.

### Blocker 2: `pnpm install` fails — `@rotiv/*` packages not on npm
Scaffolded projects (`rotiv new`) have `package.json` dependencies on `@rotiv/sdk`, `@rotiv/types`, etc. with `workspace:*` versions. These packages are not published to npm, so `pnpm install` returns 404 for every `@rotiv/*` package.

### Blocker 3: `rotiv migrate` fails — migration script not bundled
`crates/rotiv-orm/src/migration.rs` resolves the migration runner the same way as the route worker — monorepo path traversal. Same failure mode as Blocker 1.

### Minor 4: `rotiv dev` banner shows wrong file path
`crates/rotiv-core/src/server.rs` calls `.file_name()` on the route entry, which strips the directory. Shows `app/routes/[id].tsx` instead of `app/routes/todos/[id].tsx`.

### Minor 5: `rotiv add model` error missing suggestion
`crates/rotiv-cli/src/commands/add.rs` E011 error doesn't chain `.with_suggestion()`. The `corrected_code` field in JSON output is always `null` for name validation errors.

---

## YOUR TASK

Implement all five fixes in order. Follow the wave structure below. After each wave, run `cargo test --workspace` and `cargo build --workspace` to verify. Register progress in `docs/changelog/v0.2.0.md` when complete.

---

## WAVE 1 — Bundle route worker and migration script in the binary (Blockers 1, 3)

### Strategy
Use the same `include_str!()` pattern already used for templates, knowledge topics, and first-party modules. Embed the compiled JavaScript (not TypeScript source) of the route worker and migration script. At runtime, write the embedded JS to a temp file and pass its path to the resolver.

### Steps

#### 1a. Build the route worker and migration script to JS

Each of these packages needs a build step that compiles TypeScript → JavaScript:

- `packages/@rotiv/route-worker/` — add `tsconfig.json` with `"outDir": "dist"` and a `"build": "tsc"` script. The compiled output should be `dist/index.js`.
- `packages/@rotiv/migrate-script/` — same treatment.

Make sure the existing source code compiles. If it doesn't (e.g., imports from `@rotiv/*` that don't resolve), add the necessary dependencies or adjust imports to be self-contained enough to compile.

#### 1b. Embed compiled JS in the CLI binary

In `crates/rotiv-cli/src/commands/dev.rs`:
```rust
const EMBEDDED_WORKER: &str = include_str!("../../../../packages/@rotiv/route-worker/dist/index.js");
```

At the start of `run()`, write `EMBEDDED_WORKER` to a temp file using the `tempfile` crate (already a dev-dependency), then pass its path to `rotiv-core` via environment variable or a new function parameter.

Same pattern in `crates/rotiv-cli/src/commands/migrate.rs` for the migration script.

#### 1c. Update the resolvers to accept an explicit path

In `crates/rotiv-core/src/worker.rs`, modify `resolve_worker_path()` to accept an `Option<&Path>` parameter. If `Some`, use it directly (highest priority after env var). This avoids the monorepo-relative guessing entirely when the CLI provides the embedded asset.

Same in `crates/rotiv-orm/src/migration.rs`.

#### 1d. Update CI release workflow

In `.github/workflows/release.yml`, add steps before `cargo build`:
```yaml
- name: Install pnpm
  uses: pnpm/action-setup@v4
  with:
    version: 10

- name: Install Node.js
  uses: actions/setup-node@v4
  with:
    node-version: 22

- name: Install dependencies and build TS packages
  run: pnpm install && pnpm -r run build
```

This ensures the `dist/` directories exist when `include_str!()` runs during `cargo build`.

Also add a root-level `"build"` script to `package.json` if one doesn't exist:
```json
"scripts": { "build": "pnpm -r run build" }
```

### Acceptance criteria
```bash
# From a standalone binary, outside the monorepo:
rotiv new myapp && cd myapp
rotiv dev          # starts server, prints routes, no error
rotiv migrate      # generates migration files, no error
```

---

## WAVE 2 — Prepare npm packages for publishing (Blocker 2)

### Strategy
Make public `@rotiv/*` packages publishable to npm. Internal packages (`route-worker`, `migrate-script`, `build-script`) stay `"private": true`.

### Steps

#### 2a. Update each public package

For each of: `types`, `sdk`, `orm`, `signals`, `jsx-runtime`

Update `package.json`:
```json
{
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "files": ["dist"],
  "scripts": {
    "build": "tsc",
    "prepublishOnly": "pnpm build"
  }
}
```

Update `tsconfig.json`:
```json
{
  "compilerOptions": {
    "outDir": "dist",
    "declaration": true,
    "declarationDir": "dist",
    "rootDir": "src"
  }
}
```

Verify each package compiles with `pnpm build` from its directory.

#### 2b. Create npm publish workflow

Create `.github/workflows/npm-publish.yml`:
```yaml
name: Publish npm packages
on:
  push:
    tags: ['v*']
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: 10 }
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          registry-url: https://registry.npmjs.org
      - run: pnpm install
      - run: pnpm -r run build
      - run: pnpm -r publish --no-private --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

#### 2c. Update scaffolded project template

Find the template `package.json` used by `rotiv new` (in `crates/rotiv-cli/src/templates/new/`). Change all `workspace:*` versions to `^0.2.0`.

### Acceptance criteria
```bash
rotiv new myapp && cd myapp
pnpm install       # resolves all @rotiv/* from npm (after publish)
pnpm tsc --noEmit  # type-checking passes
```

---

## WAVE 3 — Fix dev banner file path (Minor 4)

### The bug
In `crates/rotiv-core/src/server.rs`, the `print_routes()` function (or equivalent) uses:
```rust
entry.file_path.file_name()
```
This returns just the filename (`[id].tsx`), losing the parent directory (`todos/`).

### The fix
Compute the path relative to the routes directory, matching how `spec_sync.rs` already does it:
```rust
let file = entry.file_path
    .strip_prefix(&routes_dir)
    .unwrap_or(&entry.file_path)
    .display()
    .to_string()
    .replace('\\', "/");
println!("  {}  {}  →  app/routes/{}", label, entry.route_path, file);
```

You'll need to pass the `routes_dir` (the `app/routes/` absolute path) into the function, or compute it from the project root.

### Acceptance criteria
```
rotiv dev
  GET  /           →  app/routes/index.tsx
  GET  /todos/:id  →  app/routes/todos/[id].tsx
```

---

## WAVE 4 — Improve error suggestions (Minor 5)

### 4a. Add `corrected_code` field to `RotivError`

In `crates/rotiv-core/src/error.rs`, add:
```rust
pub corrected_code: Option<String>,
```

Add a builder method:
```rust
pub fn with_corrected_code(mut self, code: impl Into<String>) -> Self {
    self.corrected_code = Some(code.into());
    self
}
```

Make sure it serializes in the JSON output.

### 4b. Fix E011 (model name validation)

In `crates/rotiv-cli/src/commands/add.rs`, where E011 is raised:

1. Add a `to_pascal_case()` helper that converts `"todo"` → `"Todo"`, `"user-profile"` → `"UserProfile"`
2. Chain `.with_suggestion()` and `.with_corrected_code()`:

```rust
let pascal = to_pascal_case(name);
RotivError::new("E011", format!("invalid model name '{}': must be PascalCase", name))
    .with_expected("PascalCase name (e.g. Post, UserProfile)", name)
    .with_suggestion(&format!("rotiv add model {}", pascal))
    .with_corrected_code(&format!("rotiv add model {}", pascal))
```

### 4c. Fix E012 (module name validation)

Same pattern — compute the lowercase-hyphen version and populate suggestion.

### Acceptance criteria
```bash
rotiv add model todo --json
# → { "code": "E011", "suggestion": "rotiv add model Todo", "corrected_code": "rotiv add model Todo", ... }

rotiv add module MyAuth --json
# → { "code": "E012", "suggestion": "rotiv add module my-auth", "corrected_code": "rotiv add module my-auth", ... }
```

---

## WAVE 5 — Documentation and changelog

### 5a. Update README.md
- Add a **Requirements** section: Node.js 22+, pnpm 10+
- Ensure the Quick Start flow (`rotiv new` → `pnpm install` → `rotiv dev`) is accurate post-fixes

### 5b. Update `rotiv explain` knowledge files
- `crates/rotiv-cli/src/knowledge/migrate.md` — mention the migration runner is bundled in the binary; no monorepo required
- `crates/rotiv-cli/src/knowledge/routes.md` — mention `rotiv dev` works standalone

### 5c. Write `docs/changelog/v0.2.0.md`
Document all five fixes with before/after examples.

---

## CONSTRAINTS

- Do NOT refactor code beyond what is needed for these five fixes. Keep changes minimal and focused.
- Every Rust crate must still compile independently and pass `cargo test --workspace`.
- Use the existing `include_str!()` pattern for asset embedding. Do not introduce new embedding mechanisms.
- The `tempfile` crate is already a dev-dependency of `rotiv-cli`. Use it for runtime temp files.
- All CLI commands must continue to return structured JSON with `--json`.
- Run `cargo test --workspace` and `cargo build --workspace` after each wave to catch regressions.
- Do not push to git or create tags. Just implement the code changes and write the changelog.

## FEEDBACK FILES (for reference)

The full feedback from the agent test is at:
- `feedback/v0.1.0/experience-report.md` — overall experience summary
- `feedback/v0.1.0/issue-1-route-worker-missing-from-release.md`
- `feedback/v0.1.0/issue-2-npm-packages-not-published.md`
- `feedback/v0.1.0/issue-3-migrate-requires-monorepo.md`
- `feedback/v0.1.0/issue-4-spec-sync-shows-wrong-route-file.md`
- `feedback/v0.1.0/issue-5-model-name-validation-error-missing-suggestion.md`

Read these files before starting implementation to fully understand the user experience that produced each issue.
