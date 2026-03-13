import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { CompileResult } from "./compiler.js";

export interface Manifest {
  version: string;
  built_at: string;
  files: string[];
  warnings: string[];
}

export async function writeManifest(outDir: string, result: CompileResult): Promise<void> {
  const manifest: Manifest = {
    version: "1",
    built_at: new Date().toISOString(),
    files: result.files,
    warnings: result.warnings,
  };
  await writeFile(
    join(outDir, "manifest.json"),
    JSON.stringify(manifest, null, 2) + "\n"
  );
}
