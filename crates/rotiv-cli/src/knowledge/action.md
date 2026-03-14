# Action

## Explanation
An `action` function runs on the server for POST, PUT, PATCH, and DELETE requests to its route. It receives an `ActionContext` and should return a plain serializable value or void.

`ActionContext` extends `LoaderContext` with two helpers:
- `ctx.json()` — parse the request body as JSON
- `ctx.formData()` — parse the request body as FormData

Actions are typically used for form submissions, API mutations, and CRUD operations.

## Code Example
```typescript
import { defineRoute } from "@rotiv/sdk";
import type { BetterSQLite3Database } from "@rotiv/orm";
import { users } from "../models/user.js";

export default defineRoute({
  path: "/users",

  async loader(ctx) {
    const db = ctx.db.drizzle as BetterSQLite3Database;
    return { users: await db.select().from(users) };
  },

  async action(ctx) {
    const body = await ctx.json<{ name: string; email: string }>();
    const db = ctx.db.drizzle as BetterSQLite3Database;
    await db.insert(users).values({ name: body.name, email: body.email });
    return { ok: true };
  },

  component({ data }) {
    return (
      <main>
        <form method="post">
          <input name="name" placeholder="Name" />
          <input name="email" placeholder="Email" />
          <button type="submit">Create</button>
        </form>
        <ul>{data.users.map(u => <li>{u.name}</li>)}</ul>
      </main>
    );
  },
});
```

## Related
- routes
- loader
- models
