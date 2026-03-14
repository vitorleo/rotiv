# Routes

## Explanation
Routes are the core building block of a Rotiv application. Each file in `app/routes/` maps to a URL endpoint. The filename determines the URL: `users.tsx` → `/users`, `users/[id].tsx` → `/users/:id`, `index.tsx` → `/`.

Every route file must export a `defineRoute()` call as the default export. `defineRoute()` accepts a configuration object with four optional fields: `path`, `loader`, `action`, and `component`.

- **`path`** — the URL pattern (must match the file location)
- **`loader`** — runs server-side on GET requests; returns data for the component
- **`action`** — runs server-side on POST/PUT/PATCH/DELETE requests
- **`component`** — JSX function that renders the HTML; receives `{ data }` from loader

## Code Example
```typescript
import { defineRoute } from "@rotiv/sdk";

export default defineRoute({
  path: "/users/:id",

  async loader(ctx) {
    const { id } = ctx.params;
    const users = await ctx.db.query<{ id: number; name: string }>(
      "SELECT id, name FROM users WHERE id = ?",
      [id]
    );
    return { user: users[0] ?? null };
  },

  component({ data }) {
    if (!data.user) return <p>User not found</p>;
    return (
      <main>
        <h1>{data.user.name}</h1>
      </main>
    );
  },
});
```

## Related
- loader
- action
- middleware
