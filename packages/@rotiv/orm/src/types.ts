import type { BetterSQLite3Database } from "drizzle-orm/better-sqlite3";
import type { NodePgDatabase } from "drizzle-orm/node-postgres";

// ---------------------------------------------------------------------------
// Model definition
// ---------------------------------------------------------------------------

/** A Rotiv model — a Drizzle table with a `_type` brand and a model name. */
export interface ModelDefinition<TTable> {
  readonly _type: "ModelDefinition";
  /** PascalCase model name, e.g. "User" */
  readonly _name: string;
  /** The underlying Drizzle table object — use this in queries */
  readonly table: TTable;
}

// ---------------------------------------------------------------------------
// Database access
// ---------------------------------------------------------------------------

/** Union of the two supported Drizzle instances. */
export type DrizzleInstance =
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  | BetterSQLite3Database<any>
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  | NodePgDatabase<any>;

/**
 * The normalized database access object injected into loader/action context.
 *
 * - `drizzle` — the full Drizzle query builder (type-safe)
 * - `query`   — raw SQL escape hatch, returns rows as plain objects
 */
export interface RotivDb {
  readonly _driver: "sqlite" | "postgres";
  readonly drizzle: DrizzleInstance;
  query<T = unknown>(sql: string, params?: unknown[]): Promise<T[]>;
}

// ---------------------------------------------------------------------------
// Model registry
// ---------------------------------------------------------------------------

export interface ModelRegistry {
  register(model: ModelDefinition<unknown>): void;
  get(name: string): ModelDefinition<unknown> | undefined;
  getAll(): ModelDefinition<unknown>[];
}
