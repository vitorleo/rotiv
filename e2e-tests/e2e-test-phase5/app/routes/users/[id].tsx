import { defineRoute } from "@rotiv/sdk";
import type { BetterSQLite3Database } from "@rotiv/orm";
import { users } from "../../models/user.js";

export default defineRoute({
  path: "/users/:id",
  async loader(ctx) {
    const { id } = ctx.params;
    const db = ctx.db.drizzle as BetterSQLite3Database;
    const found = await db.select().from(users);
    const user = found.find((u) => String(u.id) === id) ?? null;
    return { user, id };
  },
  component({ data }) {
    return (
      <main>
        <h1>User {data.id}</h1>
        <p>{data.user ? JSON.stringify(data.user) : "Not found"}</p>
      </main>
    );
  },
});
