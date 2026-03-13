/**
 * @rotiv/migrate-script — CLI entry point.
 *
 * Invoked by rotiv-orm (Rust) as a child process:
 *   node --import tsx <script_path> --project <dir> [--generate | --migrate | --check | --introspect]
 *
 * Prints a single JSON object to stdout on completion.
 * Exits with code 1 and prints error JSON to stderr on failure.
 */
import {
  generateMigrations,
  applyMigrations,
  checkPending,
  introspectModels,
} from "./runner.js";

function parseArgs(): {
  projectDir: string;
  mode: "generate" | "migrate" | "check" | "introspect";
} {
  const args = process.argv.slice(2);
  let projectDir = "";
  let mode: "generate" | "migrate" | "check" | "introspect" = "migrate";

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--project" && args[i + 1]) {
      projectDir = args[++i] ?? "";
    } else if (args[i] === "--generate") {
      mode = "generate";
    } else if (args[i] === "--migrate") {
      mode = "migrate";
    } else if (args[i] === "--check") {
      mode = "check";
    } else if (args[i] === "--introspect") {
      mode = "introspect";
    }
  }

  if (!projectDir) {
    process.stderr.write(
      JSON.stringify({ error: "Missing required argument: --project" }) + "\n"
    );
    process.exit(1);
  }

  return { projectDir, mode };
}

async function main(): Promise<void> {
  const { projectDir, mode } = parseArgs();

  try {
    if (mode === "generate") {
      const result = await generateMigrations(projectDir);
      process.stdout.write(JSON.stringify(result) + "\n");
    } else if (mode === "migrate") {
      const result = await applyMigrations(projectDir);
      process.stdout.write(JSON.stringify(result) + "\n");
    } else if (mode === "check") {
      const result = await checkPending(projectDir);
      process.stdout.write(JSON.stringify(result) + "\n");
    } else if (mode === "introspect") {
      const result = await introspectModels(projectDir);
      process.stdout.write(JSON.stringify(result) + "\n");
    }
  } catch (err) {
    process.stderr.write(JSON.stringify({ error: String(err) }) + "\n");
    process.exit(1);
  }
}

main();
