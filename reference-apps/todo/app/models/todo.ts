// EXAMPLE: Model file — defines the database schema for a single entity.
// Two exports are always required:
//   1. The raw Drizzle table (for drizzle-kit schema/migration generation)
//   2. The defineModel() wrapper (for Rotiv's runtime model registry)
import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

// EXAMPLE: sqliteTable("table_name", columns)
// Column helpers: integer, text, blob, real (SQLite) or serial, varchar, etc. (PG)
export const todos = sqliteTable("todos", {
  id: integer("id").primaryKey({ autoIncrement: true }),

  // EXAMPLE: text fields are nullable by default; add .notNull() to require them
  title: text("title").notNull(),

  // EXAMPLE: status as a text enum — store as string, validate in app code
  // Possible values: "pending" | "done"
  status: text("status", { enum: ["pending", "done"] })
    .notNull()
    .default("pending"),

  // EXAMPLE: createdAt as ISO string — $defaultFn() runs on INSERT
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
});

// EXAMPLE: defineModel("ModelName", tableRef) registers this model with Rotiv.
// The first argument becomes the model name in spec.json and context.md.
export const TodoModel = defineModel("Todo", todos);

// EXAMPLE: Type helpers — infer TypeScript types from the Drizzle schema.
// Use these in loader return types, action parameters, and component props.
export type Todo = typeof todos.$inferSelect;   // full row from DB
export type NewTodo = typeof todos.$inferInsert; // row for INSERT
