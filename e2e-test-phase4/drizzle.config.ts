import { defineConfig } from "drizzle-kit";

export default defineConfig({
  schema: "./app/models/*.ts",
  out: "./.rotiv/migrations",
  dialect: "sqlite",
  dbCredentials: { url: "./app/.rotiv/dev.db" },
});
