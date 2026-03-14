# Models

## Explanation
Models define database tables using Drizzle ORM's TypeScript DSL. Every model file in `app/models/` must have two exports:

1. **Raw table** (`export const users = sqliteTable(...)`) — required by `drizzle-kit` for migration generation
2. **Rotiv wrapper** (`export const UserModel = defineModel("User", users)`) — registers the model in Rotiv's runtime registry

Column helpers (`sqliteTable`, `pgTable`, `text`, `integer`, `varchar`, `serial`, `sql`) are all re-exported from `@rotiv/orm`, so you only need one import.

After changing a model file, run `rotiv migrate --generate-only` to generate a migration, then `rotiv migrate` to apply it.

## Code Example
```typescript
import { defineModel, sqliteTable, text, integer } from "@rotiv/orm";

// 1. Raw table export — drizzle-kit reads this
export const users = sqliteTable("users", {
  id: integer("id").primaryKey({ autoIncrement: true }),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  createdAt: text("created_at")
    .$defaultFn(() => new Date().toISOString())
    .notNull(),
});

// 2. Rotiv wrapper — for runtime registry
export const UserModel = defineModel("User", users);

// Type exports
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
```

## Related
- migrate
- loader
- context
