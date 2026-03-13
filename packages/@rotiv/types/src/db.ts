/**
 * Database interface injected into loader/action context.
 *
 * `drizzle` is typed as `unknown` here to keep `@rotiv/types` free of
 * drizzle-orm as a dependency. Route files that need type-safe Drizzle
 * queries should import `DrizzleInstance` from `@rotiv/orm` and cast:
 *
 *   import type { DrizzleInstance } from "@rotiv/orm";
 *   const db = ctx.db.drizzle as DrizzleInstance;
 */
export interface RotivDb {
  readonly _driver: "sqlite" | "postgres";
  /** Drizzle ORM instance — cast to DrizzleInstance from @rotiv/orm for type-safe queries. */
  readonly drizzle: unknown;
  /** Execute a raw SQL query and return typed results. */
  query<T = unknown>(sql: string, params?: unknown[]): Promise<T[]>;
}
