# Rotiv

An AI-native full-stack web framework built for agent-driven development.

Rotiv is designed so that an AI coding agent can build, extend, and maintain a production web app with zero framework confusion. Every convention is explicit, every error is structured, and the framework is its own documentation.

---

## Core Principles

1. **Predictability over flexibility** — one canonical way to do everything
2. **Declarative over imperative** — describe what, not how
3. **Self-describing APIs** — the framework is its own documentation
4. **Structured feedback loops** — errors teach, validation guides
5. **Minimal boilerplate** — every line carries meaningful intent

---

## Tech Stack

- **CLI / Runtime**: Rust (tokio, axum, sqlx, SWC, clap, serde)
- **Application Layer**: TypeScript everywhere (routes, models, components)
- **Database**: SQLite (dev) / PostgreSQL (prod) via Drizzle ORM
- **Frontend**: Signal-based reactivity, compiled to fine-grained DOM updates
- **Package manager**: pnpm workspaces

---

## Installation

```bash
# Linux / macOS
curl -fsSL https://github.com/rotiv-dev/rotiv/releases/latest/download/install.sh | bash

# Or download a binary directly from the releases page
```

---

## Quick Start

```bash
# Create a new project
rotiv new myapp
cd myapp

# Start the dev server
rotiv dev

# Scaffold a route
rotiv add route products/[id]

# Scaffold a model
rotiv add model Product

# Run migrations
rotiv migrate

# Deploy to your VPS
rotiv deploy --init   # create .rotiv/deploy.json
rotiv deploy          # build → upload → restart
```

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `rotiv new <name>` | Scaffold a new project |
| `rotiv dev` | Start dev server with hot-reload |
| `rotiv build` | Production build (tree-shaking, minification) |
| `rotiv deploy` | Deploy to a Linux VPS via SSH |
| `rotiv add route <path>` | Generate an annotated route file |
| `rotiv add model <name>` | Generate a model with schema |
| `rotiv add module <name>` | Install a module (auth, sessions, file-uploads, or custom) |
| `rotiv migrate` | Generate and run database migrations |
| `rotiv validate` | Static analysis with structured diagnostics and auto-fix |
| `rotiv explain <topic>` | Query the built-in knowledge base |
| `rotiv diff-impact <file>` | Show which routes are affected by a file change |
| `rotiv spec-sync` | Sync `.rotiv/spec.json` with current filesystem state |
| `rotiv context-regen` | Regenerate `.rotiv/context.md` for AI context |

All commands accept `--json` for structured output consumable by agents.

---

## Project Structure

```
myapp/
  app/
    routes/          # File-system routing (app/routes/users/[id].tsx → /users/:id)
    models/          # Declarative model definitions with Drizzle
    modules/         # Capability-based middleware modules
  .rotiv/
    spec.json        # Machine-readable project spec (routes, models, modules)
    context.md       # Auto-generated project description for AI context
    deploy.json      # Deploy config (gitignore this if it contains secrets)
```

---

## Routing

Routes are TypeScript files under `app/routes/`. Each file exports a `defineRoute()` call:

```tsx
// app/routes/posts/[id].tsx  →  /posts/:id
import { defineRoute } from "@rotiv/sdk";
import { posts } from "../models/post.js";
import { eq } from "@rotiv/orm";

export default defineRoute({
  path: "/posts/:id",

  async loader(ctx) {
    const [post] = await ctx.db.drizzle
      .select()
      .from(posts)
      .where(eq(posts.id, Number(ctx.params.id)));
    if (!post) throw new Response("Not found", { status: 404 });
    return { post };
  },

  async action(ctx) {
    // Handle POST/PATCH/DELETE
    return Response.redirect(`/posts/${ctx.params.id}`, 303);
  },

  component({ data }) {
    return <article><h1>{data.post.title}</h1></article>;
  },
});
```

---

## Models

```ts
// app/models/post.ts
import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

export const posts = sqliteTable("posts", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  title: text("title").notNull(),
  body: text("body").notNull(),
  createdAt: text("created_at").$defaultFn(() => new Date().toISOString()).notNull(),
});

export const PostModel = defineModel("Post", posts);
export type Post = typeof posts.$inferSelect;
```

---

## Module System

Modules declare capability contracts (`provides`, `requires`, `configures`). Three tiers:

- **Primitive** — locked core capabilities (db, router)
- **Slot** — strict interface (auth, sessions, file-uploads)
- **Escape hatch** — raw access, explicitly opt-in

```bash
# Install a first-party module
rotiv add module auth
rotiv add module sessions
rotiv add module file-uploads

# Install a custom module
rotiv add module my-feature
```

Each module gets `app/modules/<name>/module.json` (manifest), `index.ts` (implementation), and `module.test.ts` (auto-generated integration tests).

---

## Agent Integration

Rotiv is built to be used by AI coding agents:

**`.rotiv/spec.json`** — machine-readable project manifest (routes, models, modules, conventions)

**`.rotiv/context.md`** — auto-updated project description regenerated with `rotiv context-regen`

**`rotiv explain <topic>`** — built-in knowledge base with 10 topics:
`routes`, `models`, `loader`, `action`, `middleware`, `signals`, `migrate`, `context`, `modules`, `deploy`

```bash
rotiv explain loader --json
# → { "topic": "loader", "explanation": "...", "code_example": "...", "related": [...] }
```

**`rotiv validate`** — structured diagnostics:
```json
{ "code": "V001", "message": "missing default export", "file": "app/routes/index.tsx",
  "line": 1, "severity": "error", "auto_fix": "export default function..." }
```

**MCP server** (`@rotiv/mcp`) — all 12 CLI commands exposed as MCP tools for direct agent use.

---

## Repository Structure

```
rotiv/
  crates/
    rotiv-cli/        # CLI binary (the `rotiv` command)
    rotiv-core/       # HTTP server, router, dev server, hot-reload
    rotiv-orm/        # Database engine (SQLite + PostgreSQL via sqlx)
    rotiv-compiler/   # SWC-based TypeScript/JSX transformation
  packages/@rotiv/
    sdk/              # TypeScript API surface
    types/            # Shared type definitions
    orm/              # Drizzle ORM integration
    signals/          # Signal-based reactivity primitives
    jsx-runtime/      # JSX factory
    mcp/              # MCP server for agent platform integrations
    create/           # Project scaffolder
    spec/             # Machine-readable framework spec schema
  reference-apps/
    todo/             # Annotated full CRUD example app
  docs/               # Plans, changelogs, architectural decisions
  e2e-tests/          # End-to-end test projects per phase
```

---

## Deploy

```bash
# Initialize deploy config
rotiv deploy --init
# Edit .rotiv/deploy.json:
# { "host": "...", "user": "root", "remote_path": "/opt/myapp", "service_name": "myapp" }

# Deploy (build → scp binary → ssh migrate + restart)
rotiv deploy

# Dry run
rotiv deploy --dry-run
```

Requires `ssh` and `scp` on PATH. Uses your local SSH key/agent. The remote must run a systemd service.

---

## Development

```bash
# Build the CLI
cargo build -p rotiv-cli

# Run all tests
cargo test --workspace

# TypeScript typecheck
pnpm typecheck
```

Rust clean builds are slow on low-power hardware — the project uses `sccache` and GitHub Actions for release builds.

---

## License

MIT
