import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

export const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  email: text("email").notNull(),
  passwordHash: text("password_hash").notNull(),
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
});

export const UserModel = defineModel("User", users);
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
