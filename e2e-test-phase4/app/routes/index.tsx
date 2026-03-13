import { defineRoute } from "@rotiv/sdk";
import type { BetterSQLite3Database } from "@rotiv/orm";
import { users } from "../models/user.js";

export default defineRoute({
  path: "/",
  async loader(ctx) {
    // Verify raw DB connectivity
    const ping = await ctx.db.query<{ n: number }>("SELECT 1 as n");
    // Type-safe Drizzle query — cast to concrete SQLite driver
    const db = ctx.db.drizzle as BetterSQLite3Database;
    const userRows = await db.select().from(users);
    return { ping: ping[0]?.n ?? 0, users: userRows };
  },
  component({ data }) {
    return (
      <main>
        <h1>E2E Phase 4 — Data Layer</h1>
        <p>DB ping: {data.ping}</p>
        <p>Users: {JSON.stringify(data.users)}</p>
      </main>
    );
  },
});
