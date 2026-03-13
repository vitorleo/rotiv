import { createDb } from "@rotiv/orm";
import type { RotivDb } from "@rotiv/types";

let _db: RotivDb | null = null;

/**
 * Initialize the database connection. Must be called once before serving requests.
 *
 * @param projectDir - Root directory of the user's Rotiv project.
 */
export async function initDb(projectDir: string): Promise<void> {
  const databaseUrl = process.env["DATABASE_URL"];
  _db = await createDb(
    databaseUrl !== undefined ? { databaseUrl, projectDir } : { projectDir }
  );
}

/**
 * Get the database connection. Throws if initDb() has not been called.
 */
export function getDb(): RotivDb {
  if (!_db) {
    throw new Error("[route-worker] Database not initialized. Call initDb() before serving requests.");
  }
  return _db;
}
