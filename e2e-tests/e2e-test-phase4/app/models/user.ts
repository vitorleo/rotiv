import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

// Export the raw table so drizzle-kit can discover the schema for migrations
export const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
});

// defineModel wraps the table for Rotiv's model registry and runtime query API
export const UserModel = defineModel("User", users);

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
