// FRAMEWORK: Model definition using @rotiv/orm's defineModel().
// defineModel(name, drizzleTable) brands this as a Rotiv model and registers
// it in the global model registry. Column helpers (sqliteTable, text, integer)
// are re-exported from @rotiv/orm — no need to import drizzle-orm directly.
//
// Type inference:
//   User    — shape of a row returned by SELECT
//   NewUser — shape of a row for INSERT
import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

// Raw table export — required by drizzle-kit for schema discovery
export const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
});

// Rotiv model wrapper — use this in loaders/actions
export const UserModel = defineModel("User", users);

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
