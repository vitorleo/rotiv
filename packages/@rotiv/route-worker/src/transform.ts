import { transform } from "@swc/core";
import { createHash } from "node:crypto";
import { readFile, writeFile, mkdir, stat } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const CACHE_DIR = join(tmpdir(), "rotiv-transform-cache");

/**
 * Transform a TypeScript/TSX route file using SWC and cache the result.
 *
 * Cache key = sha1(filePath + mtime) so stale transforms are never served.
 * Always writes a .mjs file — bypasses the `tsx` ESM loader for clean imports.
 *
 * @returns Absolute path to the compiled .mjs file.
 */
export async function transformAndCache(filePath: string): Promise<string> {
  const fileStat = await stat(filePath);
  const key = createHash("sha1")
    .update(filePath + String(fileStat.mtimeMs))
    .digest("hex")
    .slice(0, 16);
  const outPath = join(CACHE_DIR, `${key}.mjs`);

  // Cache hit — same mtime means same source
  try {
    await stat(outPath);
    return outPath;
  } catch {
    // Cache miss — fall through to transform
  }

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
    },
    module: { type: "es6" },
    sourceMaps: false,
  });

  // Rewrite bare package specifiers to absolute file:// URLs.
  // Cached .mjs files live in the OS temp dir where Node's ESM resolver
  // cannot locate scoped packages by name — we must use absolute paths.
  // We try to resolve from both the worker package dir and the route file dir.
  const patched = await rewriteImports(result.code, filePath);

  await mkdir(CACHE_DIR, { recursive: true });
  await writeFile(outPath, patched);
  return outPath;
}

/**
 * Replace every `from "some-package"` in compiled output with the resolved
 * absolute `file://` URL. Only bare specifiers (no `.`, `/`, or `file:` prefix)
 * are rewritten.
 *
 * Resolution order per specifier:
 *   1. import.meta.resolve (worker package's own node_modules)
 *   2. require.resolve from the route file's directory (project node_modules)
 */
async function rewriteImports(code: string, routeFilePath: string): Promise<string> {
  // Match: from "package" or from 'package' (bare specifiers only)
  const BARE_IMPORT_RE = /\bfrom\s+(["'])([^./"'][^"']*)\1/g;

  // require() resolver rooted at the route file — finds project packages
  const requireFromProject = createRequire(routeFilePath);

  const seen = new Map<string, string>();

  for (const match of code.matchAll(BARE_IMPORT_RE)) {
    const quote = match[1];
    const specifier = match[2];
    if (specifier === undefined || quote === undefined) continue;
    if (seen.has(specifier)) continue;

    let resolved: string | undefined;

    // 1. Try worker's own node_modules (e.g. @rotiv/jsx-runtime)
    try {
      resolved = import.meta.resolve(specifier);
    } catch {
      // not in worker node_modules
    }

    // 2. Fall back to project node_modules (e.g. @rotiv/sdk)
    if (!resolved) {
      try {
        const cjsPath = requireFromProject.resolve(specifier);
        resolved = pathToFileURL(cjsPath).href;
      } catch {
        // unresolvable — leave as-is
      }
    }

    if (resolved) {
      seen.set(specifier, resolved);
    }
  }

  let patched = code;
  for (const [specifier, resolved] of seen) {
    // Replace all occurrences of the bare specifier
    patched = patched.replaceAll(`"${specifier}"`, `"${resolved}"`);
    patched = patched.replaceAll(`'${specifier}'`, `'${resolved}'`);
  }
  return patched;
}
