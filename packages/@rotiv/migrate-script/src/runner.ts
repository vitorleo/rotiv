import { spawnSync } from "node:child_process";
import { readFile, readdir, mkdir } from "node:fs/promises";
import { statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { writeDrizzleConfig } from "./drizzle-config.js";

const __dirname = dirname(fileURLToPath(import.meta.url));

export interface MigrateResult {
  ok: boolean;
  migrations_applied: number;
  migration_files: string[];
  warnings: string[];
  duration_ms: number;
}

export interface PendingResult {
  pending: number;
  ok: boolean;
  duration_ms: number;
}

export interface ModelIntrospection {
  name: string;
  file: string;
  columns: string[];
}

/** Resolve drizzle-kit binary — prefer local node_modules/.bin */
function resolveDrizzleKit(): string {
  const candidates = [
    join(__dirname, "../../../node_modules/.bin/drizzle-kit"),
    join(__dirname, "../../../../node_modules/.bin/drizzle-kit"),
    join(__dirname, "../node_modules/.bin/drizzle-kit"),
  ];
  for (const c of candidates) {
    try {
      statSync(c);
      return c;
    } catch {
      // try next
    }
  }
  return "drizzle-kit";
}

function detectDialect(): "sqlite" | "postgresql" {
  const dbUrl = process.env["DATABASE_URL"] ?? "";
  return dbUrl.startsWith("postgres") ? "postgresql" : "sqlite";
}

export async function generateMigrations(projectDir: string): Promise<MigrateResult> {
  const start = Date.now();
  const dialect = detectDialect();
  const configPath = await writeDrizzleConfig(projectDir, dialect);

  const result = spawnSync(
    resolveDrizzleKit(),
    ["generate", "--config", configPath],
    {
      cwd: projectDir,
      encoding: "utf8",
      shell: true,
      env: { ...process.env, NODE_OPTIONS: "--import tsx" },
    }
  );

  if (result.status !== 0) {
    throw new Error(
      `drizzle-kit generate failed:\n${result.stderr ?? result.stdout}`
    );
  }

  return {
    ok: true,
    migrations_applied: 0,
    migration_files: [],
    warnings: [],
    duration_ms: Date.now() - start,
  };
}

export async function applyMigrations(projectDir: string): Promise<MigrateResult> {
  const start = Date.now();
  const dialect = detectDialect();
  const configPath = await writeDrizzleConfig(projectDir, dialect);

  // Ensure SQLite DB directory exists before drizzle-kit tries to open it
  if (dialect === "sqlite") {
    await mkdir(join(projectDir, "app", ".rotiv"), { recursive: true });
  }

  const result = spawnSync(
    resolveDrizzleKit(),
    ["migrate", "--config", configPath],
    {
      cwd: projectDir,
      encoding: "utf8",
      shell: true,
      env: { ...process.env, NODE_OPTIONS: "--import tsx" },
    }
  );

  if (result.status !== 0) {
    throw new Error(
      `drizzle-kit migrate failed:\n${result.stderr ?? result.stdout}`
    );
  }

  return {
    ok: true,
    migrations_applied: 1,
    migration_files: [],
    warnings: [],
    duration_ms: Date.now() - start,
  };
}

/**
 * Check for pending migrations by reading the journal JSON — no subprocess spawn.
 * Returns { pending: N, ok: true } where N is the number of journal entries.
 */
export async function checkPending(projectDir: string): Promise<PendingResult> {
  const start = Date.now();
  const journalPath = join(projectDir, ".rotiv", "migrations", "_journal.json");

  try {
    const raw = await readFile(journalPath, "utf8");
    const journal = JSON.parse(raw) as { entries?: unknown[] };
    const entries = journal.entries ?? [];
    return { pending: entries.length, ok: true, duration_ms: Date.now() - start };
  } catch {
    // No journal yet — 0 pending
    return { pending: 0, ok: true, duration_ms: Date.now() - start };
  }
}

export async function introspectModels(projectDir: string): Promise<ModelIntrospection[]> {
  const modelsDir = join(projectDir, "app", "models");
  let files: string[];

  try {
    const entries = await readdir(modelsDir);
    files = entries
      .filter((f) => f.endsWith(".ts"))
      .map((f) => join(modelsDir, f));
  } catch {
    return [];
  }

  const results: ModelIntrospection[] = [];

  for (const file of files) {
    try {
      const mod = (await import(file)) as Record<string, unknown>;
      for (const value of Object.values(mod)) {
        if (
          value !== null &&
          typeof value === "object" &&
          (value as Record<string, unknown>)["_type"] === "ModelDefinition"
        ) {
          const def = value as { _name: string; table: Record<string, unknown> };
          results.push({ name: def._name, file, columns: Object.keys(def.table) });
        }
      }
    } catch {
      // Skip files that fail to import
    }
  }

  return results;
}
