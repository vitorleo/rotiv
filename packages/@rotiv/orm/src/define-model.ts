import type { ModelDefinition } from "./types.js";
import { globalModelRegistry } from "./registry.js";

/**
 * Define a Rotiv model.
 *
 * Wraps a Drizzle table with a `_type` brand and registers it in the global
 * model registry. The returned object is a valid Drizzle table (for queries)
 * and a Rotiv-typed model (for spec discovery).
 *
 * @example
 * ```typescript
 * import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";
 *
 * export const UserModel = defineModel(
 *   "User",
 *   sqliteTable("users", {
 *     id: integer("id").primaryKey({ autoIncrement: true }),
 *     name: text("name").notNull(),
 *   })
 * );
 * ```
 */
export function defineModel<TTable>(
  name: string,
  table: TTable
): ModelDefinition<TTable> {
  const model: ModelDefinition<TTable> = {
    _type: "ModelDefinition",
    _name: name,
    table,
  };
  globalModelRegistry.register(model as ModelDefinition<unknown>);
  return model;
}

// ---------------------------------------------------------------------------
// Re-export Drizzle column helpers so route files need only one import.
// ---------------------------------------------------------------------------

// SQLite column builders
export {
  sqliteTable,
  text,
  integer,
  real,
  blob,
  numeric,
} from "drizzle-orm/sqlite-core";

// PostgreSQL column builders
export {
  pgTable,
  varchar,
  serial,
  bigserial,
  boolean as pgBoolean,
  timestamp,
  jsonb,
  uuid,
} from "drizzle-orm/pg-core";

// Query helpers (driver-agnostic)
export { sql, eq, ne, and, or, gt, lt, gte, lte, like, inArray, isNull, isNotNull, desc, asc } from "drizzle-orm";
