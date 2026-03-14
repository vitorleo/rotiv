// FRAMEWORK: Model file — defines a database table and its Rotiv model wrapper.
// Two exports are required:
//   1. Raw Drizzle table export (for drizzle-kit schema discovery and migrations)
//   2. defineModel() wrapper export (for Rotiv runtime registry)
// Column helpers (sqliteTable, text, integer, etc.) are re-exported from @rotiv/orm
// so this file needs only one import.
import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

// FRAMEWORK: Raw table export — drizzle-kit reads this for migration generation.
// Column names use snake_case strings; TypeScript fields are camelCase via .$inferSelect.
// Run `rotiv migrate --generate-only` after adding or changing fields.
export const posts = sqliteTable("posts", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  // Add your fields here:
  // name: text("name").notNull(),
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
});

// FRAMEWORK: Rotiv model wrapper — registers the table in the model registry.
// First arg is the PascalCase model name (shown in `rotiv spec sync` and `rotiv context regen`).
export const PostModel = defineModel("Post", posts);

// FRAMEWORK: Type exports — use these in loaders and actions for type safety.
// Post    — a fully selected row from the database
// NewPost — the insert type (id and createdAt are optional)
export type Post = typeof posts.$inferSelect;
export type NewPost = typeof posts.$inferInsert;
