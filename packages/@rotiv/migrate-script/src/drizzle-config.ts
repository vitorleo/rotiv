import { writeFile } from "node:fs/promises";
import { join } from "node:path";

/**
 * Write a drizzle.config.ts to <projectDir>/.rotiv/ so drizzle-kit can find the schema.
 */
export async function writeDrizzleConfig(
  projectDir: string,
  dialect: "sqlite" | "postgresql" = "sqlite"
): Promise<string> {
  // Write config to project root so all paths are relative to projectDir (the cwd)
  const configPath = join(projectDir, "drizzle.config.ts");

  const dbCredentials =
    dialect === "sqlite"
      ? `{ url: "./app/.rotiv/dev.db" }`
      : `{ url: process.env["DATABASE_URL"] ?? "" }`;

  const content = `import { defineConfig } from "drizzle-kit";

export default defineConfig({
  schema: "./app/models/*.ts",
  out: "./.rotiv/migrations",
  dialect: "${dialect}",
  dbCredentials: ${dbCredentials},
});
`;

  await writeFile(configPath, content, "utf8");
  return configPath;
}
