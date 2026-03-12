#!/usr/bin/env node
/**
 * create-rotiv — bootstraps a new Rotiv project.
 * Usage: npx create-rotiv my-app
 *
 * Phase 1: TypeScript scaffold implementation.
 * Phase 2: Will delegate to the Rust CLI binary when available.
 */
import * as path from "node:path";
import { scaffoldProject } from "./scaffold.js";

async function main(): Promise<void> {
  const name = process.argv[2];

  if (!name) {
    console.error("Usage: create-rotiv <project-name>");
    process.exit(1);
  }

  if (!/^[a-zA-Z0-9_-]+$/.test(name)) {
    console.error(
      `Error: invalid project name "${name}". Only alphanumeric characters, hyphens, and underscores are allowed.`
    );
    process.exit(1);
  }

  const dest = path.resolve(process.cwd(), name);

  console.log(`Creating Rotiv project: ${name}`);
  await scaffoldProject({ name, dest });
  console.log(`\n✓ Project created at ${dest}`);
  console.log(`\n  Next steps:`);
  console.log(`    cd ${name}`);
  console.log(`    pnpm install`);
  console.log(`    rotiv dev`);
}

main().catch((err: unknown) => {
  console.error("Error:", err instanceof Error ? err.message : String(err));
  process.exit(1);
});
