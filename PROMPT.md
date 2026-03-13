# ROTIV: AI-Native Web Framework — Implementation Planning Prompt

You are the lead architect for Rotiv, a new full-stack web framework
designed from the ground up for AI coding agents. Your task is to
produce a detailed, phased implementation plan and then begin
building the framework incrementally.

## CONTEXT

Rotiv is built on these core principles:
1. Predictability over flexibility — one canonical way to do everything
2. Declarative over imperative — agents describe what, not how
3. Self-describing APIs — the framework is its own documentation
4. Structured feedback loops — errors teach, validation guides
5. Minimal boilerplate — every line carries meaningful intent

## TECH STACK

- Language: TypeScript everywhere (frontend + backend + tooling)
- Runtime Core: Rust (compiled binary) with TypeScript API surface
  - Rust crates: tokio, axum, sqlx, swc_core, napi-rs, serde,
    tower, notify, clap
- Database: SQLite (dev) / PostgreSQL (prod), declarative schema
- Frontend: Fine-grained reactive signals (Solid.js/Svelte-inspired),
  compiled to precise DOM updates
- Styling: Built-in design token system with generated utility classes
- Build: Zero-config, Rust-powered bundler (SWC-based)
- Package manager: pnpm workspaces (monorepo)

## INFRASTRUCTURE

- Development machine: Intel N150 (4-core low-power), 16 GB RAM,
  Windows 11 + WSL2 (Ubuntu 24.04). Rust clean builds are slow
  (~15-20 min) on this hardware. ALWAYS use sccache. Keep
  incremental builds fast by minimizing cross-crate dependencies.
  Offload release builds to GitHub Actions.
- CI/CD: GitHub Actions free tier with sccache. This is the primary
  compilation environment for release binaries. Build Linux x86_64
  binaries here, then deploy to VPS via scp.
- Deployment target: Hostinger KVM 2 VPS in São Paulo, Brazil
  - Ubuntu 24.04 LTS, 2 cores, 8 GB RAM, 100 GB SSD
  - SSH root@YOUR_SERVER_IP (your-server.example.com)
  - Runs: Nginx reverse proxy, PostgreSQL 16, systemd services
  - Do NOT compile Rust on the VPS. Deploy pre-built binaries only.
- This is a personal/open-source project, not a commercial
  platform. Keep infrastructure simple and lean.
- IMPORTANT: Given the limited local CPU, structure the Rust
  workspace to minimize recompilation. Keep crates small and
  focused. Use trait objects and dynamic dispatch at crate
  boundaries to reduce the dependency graph. The TS SDK should
  be developable independently without triggering Rust rebuilds.

## REPOSITORY STRUCTURE

Rotiv/ (monorepo)
  crates/            → Rust workspace
    Rotiv-core/       → Router, dev server, hot-reload, HTTP layer
    Rotiv-cli/        → CLI binary (the 'Rotiv' command)
    Rotiv-orm/        → Database engine (SQLite + Postgres via sqlx)
    Rotiv-compiler/   → SWC-based TS/JSX transformation + bundling
  packages/          → TypeScript workspace
    @Rotiv/sdk/       → TS API surface agents code against
    @Rotiv/create/    → Project scaffolder
    @Rotiv/types/     → Shared type definitions
    @Rotiv/spec/      → Machine-readable framework specification
  templates/         → Starter project templates
  reference-apps/    → Annotated canonical example applications
  tests/             → Integration and E2E tests

## WHAT ROTIV MUST DO (Feature Requirements)

### CLI (Rotiv command)
- Rotiv new <name> — Scaffold a new project with all conventions
  in place, including .Rotiv/spec.json and .Rotiv/context.md
- Rotiv dev — Start dev server with hot-reload, structured
  error output, and snapshot-based state diffing
- Rotiv build — Production build (tree-shaking, code-splitting,
  minification) with zero configuration
- Rotiv deploy — Deploy to a target server via SSH (default: your
  VPS at your-server.example.com). Copies the compiled binary,
  runs migrations, and restarts the service via systemd.
- Rotiv add route <path> — Generate route handler with annotated
  comments explaining every convention
- Rotiv add model <name> — Generate model with schema, migration,
  and auto-generated CRUD routes
- Rotiv add module <name> — Install a module from the registry,
  wire it up, update context.md
- Rotiv explain <concept> — Query the built-in knowledge base,
  return structured JSON with explanation + code example
- Rotiv validate — Static analysis against framework rules,
  return structured diagnostics with auto-fix suggestions
- Rotiv diff-impact — Analyze pending changes and report all
  affected routes, components, and module contracts
- Rotiv migrate — Generate and run database migrations from
  schema diff

### Routing
- File-system routing: app/routes/[name].tsx
- Each route file exports: default component, loader() for
  server-side data, action() for mutations, middleware array
- Dynamic segments: app/routes/users/[id].tsx
- Nested layouts: app/routes/users/layout.tsx
- API-only routes: app/routes/api/users.ts (no component export)
- All routes automatically typed end-to-end

### Data Layer
- Declarative model definitions in app/models/*.ts
- Models define fields with types, constraints, defaults,
  relationships, and state machines
- Auto-generated migrations from model changes
- Compile-time query validation (via sqlx Rust integration)
- Type-safe database queries that flow through to UI components

### Frontend Reactivity
- Signal-based: const count = signal(0)
- Derived: const doubled = derived(() => count() * 2)
- Effects: effect(() => console.log(count()))
- Components are plain functions returning JSX
- No virtual DOM — compiled to fine-grained DOM updates
- Built-in transition and animation primitives

### Module System
- Capability-based: modules declare provides/requires/configures
- Three tiers: Primitives (locked), Slots (strict interfaces),
  Escape hatches (raw access, explicitly marked)
- Auto-generated integration tests on module installation
- Module registry with agent-success-rate scoring

### Agent-Facing Features
- .Rotiv/spec.json: Complete machine-readable framework spec
- .Rotiv/context.md: Auto-updated project description
  (models, routes, components, relationships)
- Structured error messages (JSON with file, line, expected,
  got, suggestion, corrected_code)
- Rotiv explain returns task-oriented guidance, not API docs
- Rotiv validate returns structured diagnostics with auto-fixes
- Scaffolded files include FRAMEWORK: annotated comments

## IMPLEMENTATION PHASES

Plan the implementation in these phases. For each phase, produce:
- Specific deliverables (files, crates, packages)
- Acceptance criteria (what “done” looks like)
- Dependencies on prior phases
- Estimated complexity (S/M/L/XL)
- Risk factors and mitigation strategies

Phase 1: Foundation
  - Monorepo setup (Cargo workspace + pnpm workspace)
  - Rotiv-cli crate with basic command routing (clap)
  - Rotiv new command that generates a minimal project skeleton
  - .Rotiv/spec.json initial schema and generator
  - Basic TypeScript SDK package with type stubs

Phase 2: Core Runtime
  - Rotiv-core: HTTP server (axum + tokio)
  - File-system router that maps app/routes/*.tsx to handlers
  - Rotiv dev command with file watching (notify crate)
  - Basic request/response cycle working end-to-end
  - napi-rs bridge between Rust core and TypeScript SDK

Phase 3: Compiler & Bundler
  - Rotiv-compiler: SWC-based TypeScript/JSX transformation
  - Signal-based reactivity compilation (transform signals
    into fine-grained DOM operations)
  - Development bundling (fast, no optimization)
  - Production bundling (tree-shaking, code-splitting, minify)
  - Rotiv build command

Phase 4: Data Layer
  - Rotiv-orm: Model definition DSL in TypeScript
  - SQLite driver for development
  - PostgreSQL driver for production
  - Schema diffing and migration generation
  - Rotiv migrate command
  - Compile-time query validation
  - Loader/action integration with typed data flow

Phase 5: Agent Tooling
  - Rotiv explain command with built-in knowledge base
  - Rotiv validate with structured diagnostics and auto-fix
  - Rotiv diff-impact analysis
  - Annotated scaffolding (Rotiv add route/model/module)
  - .Rotiv/context.md auto-generation and updating
  - Structured error output across all commands

Phase 6: Module System
  - Module manifest format and parser
  - Capability resolution and conflict detection
  - Rotiv add module command with auto-wiring
  - Integration test auto-generation
  - Build first-party modules: auth, sessions, file-uploads

Phase 7: Polish & Distribution
  - Cross-platform binary builds (macOS, Linux, Windows)
  - Installation script (curl | sh)
  - Rotiv deploy targeting a Linux VPS via SSH + systemd
  - GitHub Actions workflow for building release binaries
  - Reference apps (todo, saas, ecommerce) fully annotated
  - Agent platform integrations (MCP server, tool definitions)
  - Benchmark suite: agent success rate measurement

## YOUR TASK

1. Read this specification carefully.
2. Produce a detailed implementation plan for Phase 4, breaking it
   into specific tasks with file paths and code structure decisions.
3. Identify any design decisions that need resolution before coding
   (list them as open questions with your recommended answers).
4. Begin implementing Phase 4, starting with the monorepo setup.
5. After each significant milestone, run tests and validate that
   the foundation is solid before proceeding.
6. Document every architectural decision in a DECISIONS.md file
   at the repo root.

Important constraints:
- Prioritize working code over perfect architecture. Ship Phase 1
  as a functional scaffold before optimizing.
- Every Rust crate must compile independently and have at least
  basic unit tests from day one.
- The TypeScript SDK must have type definitions that compile
  cleanly, even if implementations are stubs initially.
- All CLI commands must return structured JSON when called with
  --json flag, for agent consumption.
- Error messages must follow the structured format from the start
  (file, line, expected, got, suggestion) — not added later.
