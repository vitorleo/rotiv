# Migrate

## Explanation
Rotiv uses Drizzle ORM for database schema management. Migrations are generated from your model files in `app/models/` and applied to the database.

Three modes are available:
- `rotiv migrate --generate-only` — generates SQL migration files in `.rotiv/migrations/` without applying them
- `rotiv migrate` — generates and applies all pending migrations
- `rotiv migrate --check` — reports how many migrations are pending without applying them

Migration files are stored in `.rotiv/migrations/`. The development database lives at `app/.rotiv/dev.db` (SQLite). For production, set `DATABASE_URL=postgres://...` to use PostgreSQL.

`rotiv dev` automatically runs `rotiv migrate --check` at startup and applies pending migrations if any are found. This keeps the database in sync without manual steps.

## Code Example
```bash
# After adding a new field to app/models/user.ts:
rotiv migrate --generate-only
# → .rotiv/migrations/0001_add_user_role.sql created

rotiv migrate
# → 1 migration(s) applied in 42ms

rotiv migrate --check
# → 0 pending migration(s)

# JSON output for automation:
rotiv migrate --json
# → {"ok":true,"migrations_applied":1,"migration_files":[...],"duration_ms":42}
```

## Related
- models
- context
