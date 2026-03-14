# Phase 6: Module System

## Overview

Phase 6 adds the module system to Rotiv — a capability-based middleware composition layer that lets AI agents install, configure, and validate self-contained feature bundles. Three first-party modules (sessions, auth, file-uploads) are bundled into the CLI binary and can be installed with a single command.

---

## New Commands

### `rotiv add module <name>`

Scaffolds a module directory at `app/modules/<name>/`:

```bash
rotiv add module sessions      # first-party: cookie sessions
rotiv add module auth          # first-party: authentication (requires sessions)
rotiv add module file-uploads  # first-party: multipart upload handling
rotiv add module my-module     # custom: generic scaffold
```

**Output (human):**
```
✓ created app/modules/sessions/
  files: module.json, index.ts, module.test.ts

  Next steps:
    Import and use in a route:
    import { sessionsMiddleware } from "../modules/sessions/index.js";
```

**Output (JSON):**
```json
{ "ok": true, "kind": "module", "file": "app/modules/sessions/" }
```

Name validation: must be lowercase alphanumeric with hyphens (e.g. `auth`, `file-uploads`). Error code `E012` on invalid names, `E010` if the module already exists.

---

## Module Manifest Format

Each module declares capabilities in `app/modules/<name>/module.json`:

```json
{
  "name": "auth",
  "version": "0.1.0",
  "description": "Authentication middleware",
  "provides": ["auth"],
  "requires": ["sessions"],
  "configures": ["middleware"],
  "tier": "slot",
  "entry": "index.ts",
  "test": "module.test.ts"
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Module identifier (matches directory name) |
| `version` | yes | Semver string |
| `provides` | yes | Capabilities this module exports |
| `requires` | no | Capabilities this module depends on |
| `configures` | no | Framework hooks (usually `["middleware"]`) |
| `tier` | no | `primitive` / `slot` / `escape_hatch` |
| `entry` | no | Entry file (default `index.ts`) |
| `test` | no | Test file (default `module.test.ts`) |
| `description` | no | Human-readable description |

### Module Tiers

- **Primitive**: Foundational. No requirements (e.g. `sessions`).
- **Slot**: Fills a capability slot, depends on a Primitive (e.g. `auth` → `sessions`).
- **EscapeHatch**: Advanced override, bypasses normal conventions.

---

## First-Party Modules

### `sessions`

Cookie-based session management.

```typescript
import { sessionsMiddleware } from "../modules/sessions/index.js";
// Injects ctx.session.get/set/destroy into every request
```

Tier: **primitive** | Provides: `sessions` | Requires: —

### `auth`

Authentication middleware with login/logout helpers.

```typescript
import { authMiddleware, login, logout, getCurrentUser } from "../modules/auth/index.js";
// Redirects unauthenticated requests, exposes login()/logout() helpers
```

Tier: **slot** | Provides: `auth` | Requires: `sessions`

### `file-uploads`

Multipart form data handling with size/type validation.

```typescript
import { fileUploadsMiddleware } from "../modules/file-uploads/index.js";
// Parses multipart/form-data, injects ctx.files["fieldName"]
```

Tier: **slot** | Provides: `file-uploads` | Requires: —

---

## Extended `rotiv spec-sync`

Now discovers and populates the `modules` array in `.rotiv/spec.json`:

```json
{
  "modules": [
    { "name": "auth", "version": "0.1.0" },
    { "name": "sessions", "version": "0.1.0" }
  ]
}
```

Human output updated: `synced N route(s), M model(s), K module(s) → .rotiv/spec.json`

JSON output updated:
```json
{ "ok": true, "routes": 2, "models": 1, "modules": 2, "spec": ".rotiv/spec.json" }
```

---

## New Diagnostic Codes (`rotiv validate`)

| Code | Severity | Check | Fixable |
|------|----------|-------|---------|
| V008 | error | Module directory missing `module.json` | No |
| V009 | error | `module.json` is invalid JSON or missing required fields (`name`, `version`, `provides`) | No |
| V010 | error | Module missing `index.ts` entry file | No |

---

## Extended `rotiv explain`

New topic: **`modules`**

```bash
rotiv explain modules
rotiv explain modules --json
```

Returns the three-tier architecture explanation, first-party module table, code example showing middleware composition, and scaffold/validate instructions.

Total topics: 9 (routes, models, loader, action, middleware, signals, migrate, context, modules).

---

## Capability Resolution in `rotiv-core`

New module: `crates/rotiv-core/src/modules.rs`

```rust
pub struct ModuleManifest { name, version, provides, requires, configures, tier, entry, test, description }
pub fn parse_manifest(path: &Path) -> Result<ModuleManifest, RotivError>
pub fn discover_modules(project_dir: &Path) -> Result<Vec<ModuleManifest>, RotivError>
pub fn resolve_capabilities(modules: &[ModuleManifest]) -> (Vec<CapabilityConflict>, Vec<MissingRequirement>)
```

`resolve_capabilities` detects:
- **Conflicts**: two or more modules provide the same capability
- **Missing requirements**: a module `requires` a capability that no installed module `provides`

---

## File Tree (additions/modifications)

```
crates/rotiv-cli/src/
  cli.rs                                         [MODIFIED] Added Module subcommand to AddSubcommand
  main.rs                                        [MODIFIED] Dispatch rotiv add module
  commands/
    add.rs                                       [MODIFIED] run_add_module(), add_module_to_spec(), to_camel(), first_party_module()
    explain.rs                                   [MODIFIED] Added "modules" topic, updated topic count to 9
    spec_sync.rs                                 [MODIFIED] discover_modules_entries(), modules in spec.json
  modules/
    sessions/module.json                         [NEW]
    sessions/index.ts                            [NEW]
    sessions/module.test.ts                      [NEW]
    auth/module.json                             [NEW]
    auth/index.ts                                [NEW]
    auth/module.test.ts                          [NEW]
    file-uploads/module.json                     [NEW]
    file-uploads/index.ts                        [NEW]
    file-uploads/module.test.ts                  [NEW]
  templates/add/
    module_manifest.json                         [NEW]
    module_index.ts                              [NEW]
    module_test.ts                               [NEW]
  knowledge/
    modules.md                                   [NEW]

crates/rotiv-core/src/
  modules.rs                                     [NEW] ModuleManifest, discover_modules, resolve_capabilities
  analysis.rs                                    [MODIFIED] V008, V009, V010 checks
  lib.rs                                         [MODIFIED] pub mod modules, re-exports

packages/@rotiv/types/src/
  spec.ts                                        [MODIFIED] ModuleTier, expanded ModuleEntry

e2e-test-phase6/                                 [NEW workspace member]
  package.json, tsconfig.json
  app/routes/index.tsx, dashboard.tsx
  app/models/user.ts
  .rotiv/spec.json
```

---

## Verification

All checks passed:

```
✓ rotiv add module sessions         → app/modules/sessions/ created, exit 0
✓ rotiv add module sessions (dup)   → E010 error, exit 1
✓ rotiv add module auth             → first-party content, exit 0
✓ rotiv add module file-uploads     → first-party content, exit 0
✓ rotiv add module INVALID          → E012 error, exit 1
✓ rotiv add module my-custom-module → generic scaffold, exit 0
✓ rotiv spec-sync                   → 2 routes, 1 model, 3 modules, exit 0
✓ rotiv spec-sync --json            → valid JSON with modules count
✓ rotiv validate (clean)            → 0 diagnostics, exit 0
✓ rotiv validate (V008 trigger)     → 1 error, exit 1
✓ rotiv explain modules             → Markdown output, exit 0
✓ rotiv explain modules --json      → full JSON with explanation/code/related
✓ cargo test --workspace            → 58 tests pass, 0 failures
```
