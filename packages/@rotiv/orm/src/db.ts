import { mkdirSync } from "node:fs";
import { join } from "node:path";
import type { RotivDb, DrizzleInstance } from "./types.js";

/**
 * Create a database connection.
 *
 * Driver selection:
 * - `databaseUrl` starts with `postgres://` or `postgresql://` → PostgreSQL via `pg`
 * - Otherwise → SQLite via `better-sqlite3` at `<projectDir>/app/.rotiv/dev.db`
 *
 * The connection is intended to be created once per process and cached by the
 * caller (e.g. the route-worker's `db.ts` module).
 */
export async function createDb(options: {
  databaseUrl?: string;
  projectDir?: string;
}): Promise<RotivDb> {
  const { databaseUrl, projectDir = process.cwd() } = options;

  const isPostgres =
    databaseUrl !== undefined &&
    (databaseUrl.startsWith("postgres://") ||
      databaseUrl.startsWith("postgresql://"));

  if (isPostgres) {
    return createPostgresDb(databaseUrl);
  }

  return createSqliteDb(projectDir);
}

async function createSqliteDb(projectDir: string): Promise<RotivDb> {
  const { default: Database } = await import("better-sqlite3");
  const { drizzle } = await import("drizzle-orm/better-sqlite3");

  const dbDir = join(projectDir, "app", ".rotiv");
  mkdirSync(dbDir, { recursive: true });
  const dbPath = join(dbDir, "dev.db");

  const sqlite = new Database(dbPath);
  // Enable WAL mode for better concurrency
  sqlite.pragma("journal_mode = WAL");

  const instance: DrizzleInstance = drizzle(sqlite);

  return {
    _driver: "sqlite",
    drizzle: instance,
    async query<T = unknown>(sql: string, params: unknown[] = []): Promise<T[]> {
      const stmt = sqlite.prepare(sql);
      const rows = params.length > 0 ? stmt.all(...params) : stmt.all();
      return rows as T[];
    },
  };
}

async function createPostgresDb(databaseUrl: string): Promise<RotivDb> {
  const { Pool } = await import("pg");
  const { drizzle } = await import("drizzle-orm/node-postgres");

  const pool = new Pool({ connectionString: databaseUrl, max: 5 });

  const instance: DrizzleInstance = drizzle(pool);

  return {
    _driver: "postgres",
    drizzle: instance,
    async query<T = unknown>(sql: string, params: unknown[] = []): Promise<T[]> {
      const result = await pool.query(sql, params);
      return result.rows as T[];
    },
  };
}
