import { defineRoute } from "@rotiv/sdk";
import type { BetterSQLite3Database } from "@rotiv/orm";
import { users } from "../models/user.js";

export default defineRoute({
  path: "/",
  async loader(ctx) {
    const db = ctx.db.drizzle as BetterSQLite3Database;
    const userRows = await db.select().from(users);
    return { users: userRows, phase: 5 };
  },
  component({ data }) {
    return (
      <main>
        <h1>E2E Phase 5 — Agent Tooling</h1>
        <p>Users: {JSON.stringify(data.users)}</p>
        <p>Phase: {data.phase}</p>
      </main>
    );
  },
});
