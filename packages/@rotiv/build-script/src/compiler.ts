import { transform } from "@swc/core";
import { readFile, writeFile, mkdir, readdir, stat } from "node:fs/promises";
import { join, relative, basename, extname } from "node:path";

export interface CompileOptions {
  projectDir: string;
  outDir: string;
  minify: boolean;
  sourceMaps: boolean;
}

export interface CompileResult {
  files: string[];
  warnings: string[];
  durationMs: number;
}

/** Discover all .ts and .tsx route files under app/routes/ recursively. */
async function discoverRouteFiles(routesDir: string): Promise<string[]> {
  const files: string[] = [];
  try {
    await collectFiles(routesDir, files);
  } catch {
    // Routes dir doesn't exist yet — return empty
  }
  return files;
}

async function collectFiles(dir: string, acc: string[]): Promise<void> {
  const entries = await readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      await collectFiles(fullPath, acc);
    } else if (entry.isFile()) {
      const ext = extname(entry.name);
      if ((ext === ".ts" || ext === ".tsx") && !entry.name.startsWith("_")) {
        acc.push(fullPath);
      }
    }
  }
}

/** Transform a single route file and write the .mjs output. */
async function compileFile(
  filePath: string,
  outDir: string,
  routesDir: string,
  options: CompileOptions
): Promise<string> {
  const source = await readFile(filePath, "utf8");
  const result = await transform(source, {
    filename: filePath,
    jsc: {
      parser: { syntax: "typescript", tsx: true },
      transform: {
        react: {
          runtime: "automatic",
          importSource: "@rotiv",
        },
      },
      target: "es2022",
      ...(options.minify ? { minify: { compress: true, mangle: true } } : {}),
    },
    module: { type: "es6" },
    sourceMaps: options.sourceMaps && !options.minify ? "inline" : false,
  });

  // Preserve directory structure under outDir/server/routes/
  const rel = relative(routesDir, filePath);
  const outName = rel.replace(/\.(tsx?)$/, ".mjs");
  const outPath = join(outDir, "server", "routes", outName);

  await mkdir(join(outDir, "server", "routes", rel.split("/").slice(0, -1).join("/")), {
    recursive: true,
  });
  await writeFile(outPath, result.code);
  return outPath;
}

/** Compile all route files in a project. */
export async function compileRoutes(options: CompileOptions): Promise<CompileResult> {
  const start = Date.now();
  const routesDir = join(options.projectDir, "app", "routes");
  const files = await discoverRouteFiles(routesDir);
  const warnings: string[] = [];
  const written: string[] = [];

  await mkdir(join(options.outDir, "server", "routes"), { recursive: true });

  for (const file of files) {
    try {
      const out = await compileFile(file, options.outDir, routesDir, options);
      written.push(out);
    } catch (err) {
      warnings.push(`Failed to compile ${relative(options.projectDir, file)}: ${err}`);
    }
  }

  return {
    files: written,
    warnings,
    durationMs: Date.now() - start,
  };
}
