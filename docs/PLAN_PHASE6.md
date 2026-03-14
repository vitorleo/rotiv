# Rotiv Phase 6: Module System

## Context

Phase 5 delivered the full agent tooling layer: `rotiv add`, `rotiv validate`, `rotiv explain`, `rotiv spec-sync`, `rotiv context-regen`, and `rotiv diff-impact`. Phase 6 builds the module system — Rotiv's mechanism for packaging reusable capabilities that auto-wire into a project.

**Goal:** `rotiv add module <name>` installs a module, writes its manifest to `app/modules/<name>/module.json`, updates `spec.json`, injects any middleware into the project, and generates an integration test stub. Three first-party modules ship: `auth`, `sessions`, and `file-uploads`.

The module system is intentionally scoped to what an agent can reliably work with:
- Manifest format + parser (Rust)
- Capability resolution + conflict detection (Rust)
- `rotiv add module` scaffolding (Rust CLI)
- Integration test stubs (generated TypeScript)
- Three bundled first-party modules (TypeScript, embedded in CLI binary)

---

## Key Design Decisions

### D30: Module manifest as `app/modules/<name>/module.json`
Each module lives in `app/modules/<name>/`. The manifest is `module.json` — not TypeScript — so Rust can parse it without spawning a Node subprocess. The `index.ts` file inside the module directory is the TypeScript entry point (middleware, hooks, etc.).

### D31: Three capability tiers
- **Primitive** — core framework primitives, never overridden (e.g., `db`, `router`)
- **Slot** — strict interface that modules implement (e.g., `auth`, `sessions`)
- **EscapeHatch** — raw access to request/response, explicitly marked, auditable

Primitives are defined in a hardcoded list in Rust. Slots and EscapeHatches are declared in module manifests.

### D32: Capability resolution in pure Rust
No subprocess for `provides/requires` resolution. The resolver reads all `app/modules/*/module.json` files, builds a capability map, detects conflicts (two modules providing the same slot), and reports missing requirements. Output: structured JSON diagnostics.

### D33: First-party modules embedded in CLI binary
The three bundled modules (`auth`, `sessions`, `file-uploads`) have their manifest + TypeScript entry embedded via `include_str!` — same pattern as route/model templates. `rotiv add module auth` writes these files to `app/modules/auth/`.

### D34: Integration test stubs generated as `app/modules/<name>/module.test.ts`
Each module installation produces a test file with: import of the module's public API, a `describe` block with one `it` per provided capability. Tests are stubs (`expect(true).toBe(true)`) — the agent fills in real assertions.

### D35: `rotiv validate` gains V008–V010 for module checks
Three new diagnostic codes:
- V008: Module in `spec.json` but no `app/modules/<name>/` directory
- V009: Module declares `requires` capability not provided by any installed module
- V010: Two modules provide the same non-composable slot

### D36: `rotiv explain` gains `modules` topic
Added as a 9th knowledge base topic, covering the manifest format, capability tiers, and how to write a module.

---

## Module Manifest Format

**`app/modules/<name>/module.json`:**
```json
{
  "name": "auth",
  "version": "0.1.0",
  "description": "Session-based authentication middleware",
  "provides": ["auth"],
  "requires": ["sessions"],
  "configures": ["middleware"],
  "tier": "slot",
  "entry": "index.ts",
  "test": "module.test.ts"
}
```

Fields:
- `name` — must match directory name
- `version` — semver string
- `description` — human/agent description
- `provides[]` — capability names this module provides
- `requires[]` — capabilities that must be provided by another installed module
- `configures[]` — framework areas this module hooks into (`middleware`, `routes`, `models`, `db`)
- `tier` — `"primitive"` | `"slot"` | `"escape_hatch"`
- `entry` — TypeScript entry point (default: `"index.ts"`)
- `test` — integration test file (default: `"module.test.ts"`)

---

## First-Party Module Specs

### `auth` module
- **Provides:** `auth`
- **Requires:** `sessions`
- **Configures:** `middleware`
- **Entry:** Exports `authMiddleware: MiddlewareFn` and `requireAuth: MiddlewareFn`
- **Tier:** slot

### `sessions` module
- **Provides:** `sessions`
- **Requires:** *(none)*
- **Configures:** `middleware`, `db`
- **Entry:** Exports `sessionMiddleware: MiddlewareFn`, `getSession()`, `setSession()`, `clearSession()`
- **Tier:** slot

### `file-uploads` module
- **Provides:** `file-uploads`
- **Requires:** *(none)*
- **Configures:** `middleware`, `routes`
- **Entry:** Exports `uploadMiddleware: MiddlewareFn`, `handleUpload()`, `UploadedFile` type
- **Tier:** slot

---

## File Tree (additions/modifications only)

```
packages/@rotiv/types/src/
  spec.ts                                       [MODIFY] Expand ModuleEntry with provides/requires/configures/tier/entry/test

crates/rotiv-core/src/
  modules.rs                                    [NEW] ModuleManifest struct, parse_manifest(), discover_modules(), resolve_capabilities()
  lib.rs                                        [MODIFY] pub mod modules; pub use modules::{...}
  analysis.rs                                   [MODIFY] Add V008, V009, V010 diagnostic codes

crates/rotiv-cli/src/
  cli.rs                                        [MODIFY] Add Module { name } to AddSubcommand
  commands/
    add.rs                                      [MODIFY] Add run_add_module()
    validate.rs                                 [no change — calls run_diagnostics() which now includes V008-V010]
    explain.rs                                  [MODIFY] Add "modules" topic to TOPICS
  templates/add/
    module_manifest.json                        [NEW] module.json template
    module_index.ts                             [NEW] index.ts template
    module_test.ts                              [NEW] module.test.ts template
  knowledge/
    modules.md                                  [NEW] modules knowledge base topic

app/modules/ (in each project)                  [created by rotiv add module]
  <name>/
    module.json
    index.ts
    module.test.ts

e2e-test-phase6/                                [NEW workspace member]
  package.json
  tsconfig.json
  app/
    routes/index.tsx
    models/user.ts
    modules/
      sessions/module.json + index.ts + module.test.ts
      auth/module.json + index.ts + module.test.ts
  .rotiv/spec.json
```

---

## Implementation Waves

### Wave 1 — Expand `ModuleEntry` type + `rotiv-core::modules`

**`packages/@rotiv/types/src/spec.ts`** — expand `ModuleEntry`:
```typescript
export interface ModuleEntry {
  name: string;
  version: string;
  description?: string;
  provides?: string[];
  requires?: string[];
  configures?: string[];
  tier?: "primitive" | "slot" | "escape_hatch";
  entry?: string;
  test?: string;
}
```

**`crates/rotiv-core/src/modules.rs`** (new):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub provides: Vec<String>,
    pub requires: Vec<String>,
    pub configures: Vec<String>,
    pub tier: ModuleTier,
    pub entry: Option<String>,
    pub test: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleTier { Primitive, Slot, EscapeHatch }

/// Parse a single module.json file.
pub fn parse_manifest(path: &Path) -> Result<ModuleManifest, RotivError>

/// Scan app/modules/**/module.json. Returns empty Vec if directory absent.
pub fn discover_modules(project_dir: &Path) -> Result<Vec<ModuleManifest>, RotivError>

#[derive(Debug, Serialize)]
pub struct CapabilityConflict {
    pub capability: String,
    pub provided_by: Vec<String>,  // module names
}

#[derive(Debug, Serialize)]
pub struct MissingRequirement {
    pub module: String,
    pub requires: String,
}

/// Check that all requires are satisfied and no two modules provide the same slot.
pub fn resolve_capabilities(modules: &[ModuleManifest]) -> (Vec<CapabilityConflict>, Vec<MissingRequirement>)
```

Verify: `cargo test --workspace` (unit tests for manifest parsing + resolution)

---

### Wave 2 — `rotiv add module` command

**Templates (embedded via `include_str!`):**

**`crates/rotiv-cli/src/templates/add/module_manifest.json`:**
```json
{
  "name": "{{module_name}}",
  "version": "0.1.0",
  "description": "{{module_name}} module",
  "provides": ["{{module_name}}"],
  "requires": [],
  "configures": ["middleware"],
  "tier": "slot",
  "entry": "index.ts",
  "test": "module.test.ts"
}
```

**`crates/rotiv-cli/src/templates/add/module_index.ts`:**
```typescript
// FRAMEWORK: Module entry point — export your module's public API here.
// provides: ["{{module_name}}"]
// configures: ["middleware"]
// tier: "slot"
//
// A module can export:
//   - MiddlewareFn functions (injected into route pipelines)
//   - Utility functions (imported by route files)
//   - Type definitions (for route file type safety)
import type { MiddlewareFn } from "@rotiv/types";

// FRAMEWORK: Replace this with your module's actual implementation.
// This middleware runs before every route that includes it.
export const {{module_name}}Middleware: MiddlewareFn = async (ctx, next) => {
  // TODO: implement {{module_name}} logic
  await next();
};
```

**`crates/rotiv-cli/src/templates/add/module_test.ts`:**
```typescript
// FRAMEWORK: Integration test stubs for the {{module_name}} module.
// Run with: pnpm test (or your test runner of choice)
// These are stubs — replace expect(true).toBe(true) with real assertions.
import { {{module_name}}Middleware } from "./index.js";

describe("{{module_name}} module", () => {
  it("exports {{module_name}}Middleware", () => {
    expect(typeof {{module_name}}Middleware).toBe("function");
  });

  it("middleware calls next()", async () => {
    let nextCalled = false;
    const mockCtx = {} as Parameters<typeof {{module_name}}Middleware>[0];
    await {{module_name}}Middleware(mockCtx, async () => { nextCalled = true; });
    expect(nextCalled).toBe(true);
  });
});
```

**`crates/rotiv-cli/src/commands/add.rs`** — add `run_add_module()`:
```rust
pub fn run_add_module(name: &str, mode: OutputMode) -> Result<(), CliError> {
    // 1. find_project_root()
    // 2. Validate name: alphanumeric + hyphens, lowercase
    // 3. Dest: app/modules/<name>/ — refuse if exists
    // 4. Write module_manifest.json → module.json (substitute {{module_name}})
    // 5. Write module_index.ts → index.ts
    // 6. Write module_test.ts → module.test.ts
    // 7. Update .rotiv/spec.json: add entry to modules array
    // 8. Print summary
}
```

**`crates/rotiv-cli/src/cli.rs`** — add to `AddSubcommand`:
```rust
/// Install a module (scaffolds app/modules/<name>/)
Module {
    /// Module name, e.g. "auth" or "my-module"
    name: String,
},
```

Verify: `rotiv add module sessions` → creates 3 files + updates spec.json

---

### Wave 3 — First-party modules embedded in CLI

Three bundled first-party modules stored as template sets under `crates/rotiv-cli/src/modules/`:

**`crates/rotiv-cli/src/modules/auth/`:**
- `module.json` — provides `auth`, requires `sessions`, tier `slot`
- `index.ts` — exports `authMiddleware`, `requireAuth`
- `module.test.ts` — stubs for authMiddleware + requireAuth

**`crates/rotiv-cli/src/modules/sessions/`:**
- `module.json` — provides `sessions`, tier `slot`
- `index.ts` — exports `sessionMiddleware`, `getSession`, `setSession`, `clearSession`
- `module.test.ts` — stubs

**`crates/rotiv-cli/src/modules/file-uploads/`:**
- `module.json` — provides `file-uploads`, configures `middleware`, `routes`
- `index.ts` — exports `uploadMiddleware`, `handleUpload`, `UploadedFile` type
- `module.test.ts` — stubs

**`run_add_module()` detects first-party names:**
If `name` is `"auth"`, `"sessions"`, or `"file-uploads"`, use the embedded first-party templates instead of the generic template. This allows `rotiv add module auth` to produce a richer, fully implemented module stub.

Verify: `rotiv add module auth` → creates auth module with requireAuth exported; `rotiv add module sessions` → sessions module with getSession/setSession.

---

### Wave 4 — V008/V009/V010 validate diagnostics

**`crates/rotiv-core/src/analysis.rs`** — add three new checks to `run_diagnostics()`:

```rust
// V008: Module in spec.json but no app/modules/<name>/ directory
// V009: Module declares requires capability not provided by any installed module
// V010: Two modules provide the same non-composable slot
```

Implementation:
1. Read `spec.json` → get `modules` array
2. For each module entry: check `app/modules/<name>/` exists (V008)
3. Call `rotiv_core::modules::discover_modules()` → parse all manifests
4. Call `resolve_capabilities()` → get conflicts + missing requirements
5. Emit V009 for each missing requirement
6. Emit V010 for each capability conflict

All new codes are warnings (not errors) since module setup is optional.

Verify: `rotiv validate` with missing module dir → V008; with unmet requires → V009; with duplicate slot → V010.

---

### Wave 5 — `rotiv explain modules` knowledge topic

**`crates/rotiv-cli/src/knowledge/modules.md`** (new):
```markdown
# Modules

## Explanation
Modules are Rotiv's mechanism for packaging reusable capabilities...
[full explanation of manifest format, tiers, provides/requires, how to use]

## Code Example
[rotiv add module auth, then using authMiddleware in a route]

## Related
- routes, middleware, context
```

**`crates/rotiv-cli/src/commands/explain.rs`** — add to `TOPICS`:
```rust
("modules", include_str!("../knowledge/modules.md")),
```

Verify: `rotiv explain modules` → Markdown; `rotiv explain modules --json` → structured JSON.

---

### Wave 6 — `spec-sync` discovers installed modules

**`crates/rotiv-cli/src/commands/spec_sync.rs`** — add module discovery:
1. Call `rotiv_core::modules::discover_modules(project_dir)`
2. Convert `Vec<ModuleManifest>` to JSON array
3. Overwrite `modules` array in spec.json (alongside routes + models)

Module entry in spec.json:
```json
{
  "name": "auth",
  "version": "0.1.0",
  "provides": ["auth"],
  "requires": ["sessions"],
  "configures": ["middleware"],
  "tier": "slot"
}
```

Verify: `rotiv spec-sync` after installing modules → spec.json has populated modules array.

---

### Wave 7 — E2E test + changelog

Create `e2e-test-phase6/` workspace member.

**Scripted verification:**
1. `rotiv add module sessions` → `app/modules/sessions/` with 3 files, spec.json updated, exit 0
2. `rotiv add module auth` → `app/modules/auth/` (first-party), spec.json updated, exit 0
3. `rotiv add module sessions` (again) → error "already exists", exit 1
4. `rotiv spec-sync` → spec.json has `modules: [2 entries]`, exit 0
5. `rotiv validate` → 0 errors, 0 warnings, exit 0
6. `rotiv validate` with missing module dir → V008 warning, exit 0
7. `rotiv validate` with unmet requires (auth without sessions) → V009 warning
8. `rotiv explain modules` → Markdown output, exit 0
9. `rotiv explain modules --json` → JSON with topic/explanation/code_example/related
10. `pnpm -r typecheck` → all packages pass
11. `cargo test --workspace` → all tests pass
12. e2e-test-phase5 unchanged (backward compat)

Write `changelog/phase6.md`.

---

## Critical Files

| File | Change |
|------|--------|
| [crates/rotiv-core/src/modules.rs](crates/rotiv-core/src/modules.rs) | NEW — ModuleManifest, discover_modules(), resolve_capabilities() |
| [crates/rotiv-core/src/analysis.rs](crates/rotiv-core/src/analysis.rs) | Add V008, V009, V010 diagnostics |
| [crates/rotiv-cli/src/commands/add.rs](crates/rotiv-cli/src/commands/add.rs) | Add run_add_module() |
| [crates/rotiv-cli/src/cli.rs](crates/rotiv-cli/src/cli.rs) | Add Module to AddSubcommand |
| [crates/rotiv-cli/src/commands/spec_sync.rs](crates/rotiv-cli/src/commands/spec_sync.rs) | Add module discovery to sync |
| [crates/rotiv-cli/src/commands/explain.rs](crates/rotiv-cli/src/commands/explain.rs) | Add modules topic |
| [packages/@rotiv/types/src/spec.ts](packages/@rotiv/types/src/spec.ts) | Expand ModuleEntry |

## Reused from Prior Phases

- `find_project_root()` — used by add module + spec-sync
- `rotiv_orm::discover_models()` — reference pattern for `discover_modules()`
- `include_str!` template pattern — used for all 3×3 = 9 module template files
- `run_diagnostics()` in `analysis.rs` — extended with V008-V010
- `OutputMode` + human/JSON output pattern
- `serde_json` already in workspace

## Verification

```bash
cargo build --workspace
rotiv add module sessions
rotiv add module auth
rotiv spec-sync --json
rotiv validate --json
rotiv explain modules --json
cargo test --workspace
pnpm -r typecheck
```

## Acceptance Criteria

- [ ] `rotiv add module <name>` creates `app/modules/<name>/module.json`, `index.ts`, `module.test.ts`
- [ ] `rotiv add module auth` uses first-party template with `requireAuth` export
- [ ] `rotiv add module sessions` uses first-party template with `getSession`/`setSession`
- [ ] `rotiv add module file-uploads` uses first-party template with `uploadMiddleware`
- [ ] `rotiv spec-sync` populates `modules` array in spec.json
- [ ] `rotiv validate` V008 fires when module in spec.json but dir missing
- [ ] `rotiv validate` V009 fires when `requires` capability not provided
- [ ] `rotiv validate` V010 fires when two modules provide same slot
- [ ] `rotiv explain modules` returns Markdown or JSON, exit 0
- [ ] `cargo test --workspace` — all tests pass
- [ ] `pnpm -r typecheck` — all packages pass
- [ ] e2e-test-phase5 unchanged (backward compat)
