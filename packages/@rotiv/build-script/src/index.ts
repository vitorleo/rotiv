/**
 * @rotiv/build-script — CLI entry point.
 *
 * Invoked by rotiv-compiler (Rust) as a child process:
 *   node --import tsx <script_path> --project <dir> --out <dir> [--minify]
 *
 * Prints a single JSON object to stdout on completion.
 * Exits with code 1 and prints error JSON to stderr on failure.
 */
import { compileRoutes } from "./compiler.js";
import { writeManifest } from "./manifest.js";
import { join } from "node:path";

function parseArgs(): { projectDir: string; outDir: string; minify: boolean } {
  const args = process.argv.slice(2);
  let projectDir = "";
  let outDir = "";
  let minify = false;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--project" && args[i + 1]) {
      projectDir = args[++i] ?? "";
    } else if (args[i] === "--out" && args[i + 1]) {
      outDir = args[++i] ?? "";
    } else if (args[i] === "--minify") {
      minify = true;
    }
  }

  if (!projectDir) {
    process.stderr.write(
      JSON.stringify({ error: "Missing required argument: --project" }) + "\n"
    );
    process.exit(1);
  }

  if (!outDir) {
    outDir = join(projectDir, "dist");
  }

  return { projectDir, outDir, minify };
}

async function main(): Promise<void> {
  const { projectDir, outDir, minify } = parseArgs();

  try {
    const result = await compileRoutes({
      projectDir,
      outDir,
      minify,
      sourceMaps: !minify,
    });

    await writeManifest(outDir, result);

    process.stdout.write(
      JSON.stringify({
        files: result.files,
        warnings: result.warnings,
        duration_ms: result.durationMs,
      }) + "\n"
    );
  } catch (err) {
    process.stderr.write(
      JSON.stringify({ error: String(err) }) + "\n"
    );
    process.exit(1);
  }
}

main();
