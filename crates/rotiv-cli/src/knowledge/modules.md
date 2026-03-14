# Modules

## Explanation

Modules are self-contained capability bundles that extend Rotiv routes via middleware. Each module lives in `app/modules/<name>/` and declares its capabilities in a `module.json` manifest.

Modules use a three-tier architecture:

- **Primitive**: foundational building blocks (e.g. `sessions` — provides raw session storage). No dependencies.
- **Slot**: pluggable middleware that fills a capability slot (e.g. `auth` — requires `sessions`, provides `auth`). Follows the Primitive it depends on.
- **EscapeHatch**: advanced overrides that bypass normal conventions. Use sparingly.

A module manifest (`module.json`) declares:

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

- `provides` — capabilities this module makes available
- `requires` — capabilities that must be installed before this module
- `configures` — framework hooks this module participates in (usually `"middleware"`)
- `tier` — `primitive`, `slot`, or `escape_hatch`

### First-party modules

Use `rotiv add module <name>` to install a first-party module:

| Name | Tier | Provides | Requires |
|------|------|----------|---------|
| `sessions` | primitive | sessions | — |
| `auth` | slot | auth | sessions |
| `file-uploads` | slot | file-uploads | — |

## Code Example

```typescript
// app/routes/dashboard.tsx
import { defineRoute } from "@rotiv/sdk";
import { sessionsMiddleware } from "../modules/sessions/index.js";
import { authMiddleware } from "../modules/auth/index.js";

export default defineRoute({
  path: "/dashboard",

  // Modules compose as middleware arrays — executed left to right
  middleware: [
    sessionsMiddleware({ secret: process.env.SESSION_SECRET! }),
    authMiddleware({ redirectTo: "/login" }),
  ],

  async loader(ctx) {
    // ctx.session and ctx.user are now available (injected by middleware)
    const userId = ctx.session.get("userId");
    return { userId };
  },

  component({ data }) {
    return (
      <main>
        <h1>Dashboard</h1>
        <p>User ID: {data.userId}</p>
      </main>
    );
  },
});
```

### Scaffold a custom module

```bash
rotiv add module my-module
```

This creates:
- `app/modules/my-module/module.json` — manifest
- `app/modules/my-module/index.ts` — middleware entry point
- `app/modules/my-module/module.test.ts` — integration test stub

### Validate module health

```bash
rotiv validate
```

Checks V008 (missing module.json), V009 (invalid manifest fields), V010 (missing index.ts).

## Related

- middleware, routes, sessions, auth
