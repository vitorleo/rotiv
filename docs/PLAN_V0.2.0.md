# Rotiv v0.2.0 Improvement Plan

Based on [v0.1.0 agent feedback](../feedback/v0.1.0/).

## Problem Statement

The v0.1.0 release works inside the monorepo but is **completely non-functional as a standalone install**. An AI agent successfully wrote a complete todo app using the scaffolding and validation tools, but could not run, migrate, or type-check it. Three blockers share a single root cause: the binary assumes it lives inside the Rotiv source tree.

---

## Issues (ordered by priority)

| # | Issue | Severity | Root Cause |
|---|-------|----------|------------|
| 1 | `rotiv dev` fails — route worker not bundled | Blocker | `worker.rs:142-163` resolves worker via monorepo-relative path |
| 2 | `pnpm install` fails — `@rotiv/*` packages not on npm | Blocker | Packages exist only in the monorepo workspace |
| 3 | `rotiv migrate` fails — migration script not bundled | Blocker | `migration.rs:147-178` resolves script via monorepo-relative path |
| 4 | `rotiv dev` banner shows wrong file path for nested routes | Minor | `server.rs:363` calls `.file_name()` instead of showing relative path |
| 5 | `rotiv add model` error missing `suggestion`/`corrected_code` | Enhancement | `add.rs:72-76` doesn't chain `.with_suggestion()` |

---

## Design Decisions

### D43: Bundle TypeScript assets inside the Rust binary

Embed the route worker and migration script as compiled JavaScript using `include_str!()`. At runtime, write them to a temp directory and point `tsx` / `node` at them. This is the same pattern used for first-party modules and explain topics — no new mechanism needed.

**Why not ship them alongside the binary?**
Standalone binaries are easier to distribute (curl | sh, GitHub Releases). A single file with no companion assets eliminates path-resolution bugs entirely.

**Trade-off:** Binary size grows by ~50-100KB (compiled JS). Acceptable.

### D44: Publish `@rotiv/*` packages to npm as v0.2.0-alpha

Packages that scaffolded projects depend on (`@rotiv/types`, `@rotiv/sdk`, `@rotiv/orm`, `@rotiv/signals`, `@rotiv/jsx-runtime`) must be on npm for `pnpm install` to work. Internal packages (`@rotiv/route-worker`, `@rotiv/migrate-script`, `@rotiv/build-script`) stay private.

**Publish checklist per package:**
- `"private": false` (already set for public packages)
- Add `"files": ["dist"]` to limit what goes to npm
- Add `"exports"` pointing to `./dist/index.js` and `./dist/index.d.ts`
- Add `"scripts": { "build": "tsc" }` with proper `tsconfig.json` output config
- Add `"prepublishOnly": "pnpm build"` safety net

### D45: Compute `corrected_code` for name validation errors

When the framework can deterministically compute the correct input (lowercase → PascalCase for models, uppercase → lowercase-hyphen for modules), populate both `suggestion` and `corrected_code` in the error JSON. This enables agents to auto-retry with the corrected command.

---

## Implementation Waves

### Wave 1 — Fix blockers (Issues 1, 3)

**Goal:** `rotiv dev` and `rotiv migrate` work from a standalone binary.

#### 1a. Bundle route worker into CLI binary

**Files to change:**
- `packages/@rotiv/route-worker/` — add a build step that compiles `src/index.ts` → `dist/worker.js`
- `crates/rotiv-cli/src/commands/dev.rs` — embed compiled worker via `include_str!("../../packages/@rotiv/route-worker/dist/worker.js")`, write to temp file at startup, pass path to core
- `crates/rotiv-core/src/worker.rs` — add a 4th resolution path: accept a direct path passed from CLI (avoids monorepo-relative guessing in core)

**Resolution order (updated):**
1. `ROTIV_WORKER_PATH` env var (existing, for advanced users)
2. Path passed from CLI (new — CLI provides the embedded asset path)
3. Dev monorepo layout (existing, for development)
4. Production binary-relative fallback (existing, kept for compatibility)

#### 1b. Bundle migration script into CLI binary

**Files to change:**
- `packages/@rotiv/migrate-script/` — add build step compiling to `dist/index.js`
- `crates/rotiv-cli/src/commands/migrate.rs` — embed compiled script, write to temp, pass to orm
- `crates/rotiv-orm/src/migration.rs` — accept a direct path parameter, same pattern as 1a

#### 1c. Pre-build step for CI

**Files to change:**
- `.github/workflows/release.yml` — add `pnpm install && pnpm build` step before `cargo build` so that `include_str!` can reference the compiled JS output
- Root `package.json` — add `"build"` script that builds all packages

**Acceptance criteria:**
```bash
# From a standalone binary, in a fresh directory:
rotiv new myapp && cd myapp
rotiv dev          # starts server, no error
rotiv migrate      # generates migration files, no error
```

---

### Wave 2 — Publish npm packages (Issue 2)

**Goal:** `pnpm install` works in a scaffolded project.

#### 2a. Prepare packages for publishing

**For each public package** (`types`, `sdk`, `orm`, `signals`, `jsx-runtime`):

| Field | Current | Target |
|-------|---------|--------|
| `main` | `./src/index.ts` | `./dist/index.js` |
| `types` | `./src/index.ts` | `./dist/index.d.ts` |
| `exports` | missing or source | `{ ".": { "import": "./dist/index.js", "types": "./dist/index.d.ts" } }` |
| `files` | missing | `["dist"]` |
| `scripts.build` | missing or no-op | `"tsc"` |
| `scripts.prepublishOnly` | missing | `"pnpm build"` |

**For each tsconfig.json:**
- Set `"outDir": "dist"`, `"declaration": true`, `"declarationDir": "dist"`
- Ensure `"rootDir": "src"`

#### 2b. Add npm publish workflow

**New file:** `.github/workflows/npm-publish.yml`
- Trigger: `push: tags: ['v*']` (same as release)
- Steps: checkout → pnpm install → pnpm build → `pnpm -r publish --no-private --access public`
- Requires `NPM_TOKEN` secret in repo settings

#### 2c. Update scaffolded project template

**File:** `crates/rotiv-cli/src/templates/new/package.json` (the template used by `rotiv new`)
- Change dependency versions from `workspace:*` to `^0.2.0` (or whatever the published version is)
- Ensure the generated project can install from npm without being in a workspace

**Acceptance criteria:**
```bash
rotiv new myapp && cd myapp
pnpm install       # all @rotiv/* packages resolve from npm
pnpm tsc --noEmit  # type-checking works
```

---

### Wave 3 — Fix dev banner path (Issue 4)

**Goal:** `rotiv dev` shows full relative path for nested routes.

**File:** `crates/rotiv-core/src/server.rs` lines 359-370

**Current code (broken):**
```rust
let file = entry.file_path.file_name()  // "index.tsx" — loses directory
```

**Fix:**
```rust
let file = entry.file_path
    .strip_prefix(&routes_dir)
    .unwrap_or(&entry.file_path)
    .display()
    .to_string()
    .replace('\\', "/");
println!("  {}  {}  →  app/routes/{}", label, entry.route_path, file);
```

This mirrors the approach already used in `spec_sync.rs:138` which correctly computes relative paths.

**Acceptance criteria:**
```
rotiv dev
  GET  /           →  app/routes/index.tsx
  GET  /todos/:id  →  app/routes/todos/[id].tsx
```

---

### Wave 4 — Improve error suggestions (Issue 5)

**Goal:** Name validation errors include `suggestion` and `corrected_code`.

#### 4a. Model name (E011)

**File:** `crates/rotiv-cli/src/commands/add.rs` lines 72-76

Add a `to_pascal_case()` helper and chain suggestion:

```rust
let pascal = to_pascal_case(name);
let err = RotivError::new(
    "E011",
    format!("invalid model name '{}': must be PascalCase", name),
)
.with_expected("PascalCase name (e.g. Post, UserProfile)", name)
.with_suggestion(&format!("rotiv add model {}", pascal));
```

The `--json` output should also populate `corrected_code`:
```json
{
  "code": "E011",
  "suggestion": "rotiv add model Todo",
  "corrected_code": "rotiv add model Todo"
}
```

#### 4b. Module name (E012)

Same pattern — compute the hyphenated lowercase version and populate `suggestion`.

#### 4c. Add `corrected_code` to `RotivError`

**File:** `crates/rotiv-core/src/error.rs`

Add a `corrected_code: Option<String>` field and a `.with_corrected_code()` builder method. Update the Serialize impl to include it in JSON output.

**Acceptance criteria:**
```bash
rotiv add model todo --json
# → { "code": "E011", ..., "suggestion": "rotiv add model Todo", "corrected_code": "rotiv add model Todo" }
```

---

### Wave 5 — Documentation and release

#### 5a. Update README

- Remove the assumption that `rotiv dev` and `pnpm install` work immediately (they will work after Wave 1+2, but be honest about current state until then)
- Add a "Requirements" section: Node.js 22+, pnpm 10+
- Link to the releases page for binary downloads

#### 5b. Update `rotiv explain` topics

- `rotiv explain migrate` — mention that the migration runner is bundled in the binary
- `rotiv explain routes` — clarify that `rotiv dev` fully works standalone

#### 5c. Changelog

Create `docs/changelog/v0.2.0.md` documenting all fixes.

#### 5d. Tag and release

```bash
git tag v0.2.0
git push origin v0.2.0   # triggers release.yml (binaries) + npm-publish.yml (packages)
```

---

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| `include_str!` on compiled JS bloats binary | Low | Route worker + migrate script are ~50KB combined |
| npm publish scope (`@rotiv`) requires org setup | Medium | Create npm org `rotiv` first; document in CONTRIBUTING |
| Temp file cleanup on crash | Low | Use `tempfile` crate with RAII; files auto-delete on drop |
| pnpm workspace `workspace:*` protocol in templates | High | Must change to `^0.2.0` in template `package.json` before release |
| CI needs `pnpm build` before `cargo build` | Medium | Add explicit step in release workflow; document build order |

---

## Summary

Five issues, one theme: **the framework must work outside the monorepo**. Waves 1-2 fix the three blockers by bundling assets in the binary and publishing packages to npm. Wave 3-4 fix the two minor issues. Wave 5 documents everything and ships v0.2.0.
