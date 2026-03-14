# Loader

## Explanation
A `loader` function runs on the server for every GET request to its route. It receives a `LoaderContext` and must return a plain serializable value. That value becomes `data` in the route's `component`.

`LoaderContext` provides:
- `ctx.request` — raw Request object
- `ctx.params` — dynamic path parameters (e.g. `{ id: "42" }` for `/users/:id`)
- `ctx.searchParams` — URLSearchParams from the query string
- `ctx.headers` — request headers
- `ctx.db` — database connection (`RotivDb`); use `ctx.db.drizzle` for type-safe Drizzle queries

The loader return type is automatically inferred as the `data` type in `component`. No manual type annotation is needed.

## Code Example
```typescript
import { defineRoute } from "@rotiv/sdk";
import type { BetterSQLite3Database } from "@rotiv/orm";
import { users } from "../models/user.js";

export default defineRoute({
  path: "/users",

  async loader(ctx) {
    const db = ctx.db.drizzle as BetterSQLite3Database;
    const allUsers = await db.select().from(users);
    const page = Number(ctx.searchParams.get("page") ?? "1");
    return { users: allUsers, page };
  },

  component({ data }) {
    return (
      <ul>
        {data.users.map(u => <li key={u.id}>{u.name}</li>)}
      </ul>
    );
  },
});
```

## Related
- routes
- action
- models
