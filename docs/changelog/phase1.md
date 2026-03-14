# Phase 1 — Monorepo Scaffold & CLI Foundation

## Summary

Established the full monorepo structure, Rust CLI with `rotiv new` / `rotiv info`, `.rotiv/spec.json` schema, and TypeScript SDK type stubs.

## What was built

### Rust workspace (`crates/`)
- **`rotiv-core`** — Shared `RotivError` struct (code, message, file, line, suggestion, corrected_code). Serializable to JSON. Stub crate only in Phase 1.
- **`rotiv-cli`** — Binary crate with two commands:
  - `rotiv new <name>` — Scaffolds a new project directory from embedded templates. Writes `app/routes/index.tsx`, `package.json`, `tsconfig.json`, `.rotiv/spec.json`, `README.md`.
  - `rotiv info` — Reads `.rotiv/spec.json` from the nearest project root and prints framework version, project name, and route count.
- **`rotiv-orm`**, **`rotiv-compiler`** — Zero-dependency stub crates (Phase 3+).
- **`.cargo/config.toml`** — sccache wrapper + sparse registry for fast incremental builds.

### TypeScript packages (`packages/@rotiv/`)
- **`@rotiv/types`** — Core type stubs: `RouteDefinition`, `RouteConfig`, `RotivServer`, `ServerConfig`, `LoaderContext`, `ActionContext`.
- **`@rotiv/spec`** — `.rotiv/spec.json` schema types + `validateSpec()` function + JSON Schema file.
- **`@rotiv/sdk`** — Developer-facing API: `defineRoute()`, `createServer()` (stub), `RotivRuntimeError`.
- **`@rotiv/create`** — `create-rotiv` scaffolding helper (stub for now).

### Templates (`templates/default/`)
- `app/routes/index.tsx` — Starter route using `defineRoute()` with loader + JSX component.
- `package.json`, `tsconfig.json` — Project defaults.
- `.rotiv/spec.json` — Project spec with `{{project_name}}` placeholder.

## Key decisions

- **D1** — Templates embedded in binary via `include_str!` (single-binary distribution).
- **D2** — `rotiv-core` is a stub in Phase 1; HTTP server arrives in Phase 2.
- **D3** — No napi-rs in Phase 1; TypeScript SDK is pure stubs.
- **D4** — sccache for fast Rust builds on low-power hardware (Intel N150).
- **D5** — pnpm workspaces with `workspace:*` protocol.
- **D6** — `.rotiv/spec.json` uses versioned `$schema` URL; JSON Schema bundled locally for offline use.
