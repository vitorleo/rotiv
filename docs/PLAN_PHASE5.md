# Phase 5 Implementation Plan

See full plan at: C:\Users\Vitor\.claude\plans\hazy-sprouting-quiche.md

## Summary

Phase 5 adds the agent-facing tooling layer:

- `rotiv add route/model` — annotated scaffolding (Wave 1)
- `rotiv spec sync` — live-populates `.rotiv/spec.json` (Wave 2)
- `rotiv validate [--fix]` — 7-code static analysis (Wave 3)
- `rotiv explain <topic>` — embedded knowledge base (Wave 4)
- `rotiv context regen` — regenerates `.rotiv/context.md` (Wave 5)
- `rotiv diff-impact <file>` — import graph scan (Wave 6)
- E2E test + changelog (Wave 7)

## Design Decisions

| ID | Decision |
|----|----------|
| D23 | `rotiv add` uses `Add { subcommand: AddSubcommand }` variant — mirrors Cargo's `cargo add` pattern |
| D24 | Templates compiled into binary via `include_str!` from `crates/rotiv-cli/src/templates/add/` |
| D25 | `rotiv spec sync` reads existing spec.json, overwrites only `routes`/`models` arrays, preserves other fields |
| D26 | `rotiv validate` uses line-by-line `contains()` scan — no AST parsing needed for 7 target invariants |
| D27 | `rotiv explain` embeds 8 Markdown topics via `include_str!`; fuzzy match: exact → prefix → contains |
| D28 | `rotiv context regen` is pure Rust — no Node subprocess; uses existing route/model discovery |
| D29 | `rotiv diff-impact` scans `import` lines for target filename stem — pure Rust string matching |

## New Files

```
crates/rotiv-cli/src/
  templates/add/route.tsx       — annotated route template
  templates/add/model.ts        — annotated model template
  knowledge/routes.md           — routes explanation
  knowledge/models.md
  knowledge/loader.md
  knowledge/action.md
  knowledge/middleware.md
  knowledge/signals.md
  knowledge/migrate.md
  knowledge/context.md
  commands/add.rs               — rotiv add route/model
  commands/validate.rs          — rotiv validate [--fix]
  commands/explain.rs           — rotiv explain <topic>
  commands/context.rs           — rotiv context regen
  commands/diff_impact.rs       — rotiv diff-impact <file>
  commands/spec_sync.rs         — rotiv spec sync

crates/rotiv-core/src/
  analysis.rs                   — Diagnostic struct + run_diagnostics()

e2e-test-phase5/                — new workspace member
```
