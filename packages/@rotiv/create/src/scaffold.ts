import * as fs from "node:fs";
import * as path from "node:path";
import {
  SPEC_JSON_TEMPLATE,
  CONTEXT_MD_TEMPLATE,
  ROUTES_INDEX_TSX_TEMPLATE,
  PACKAGE_JSON_TEMPLATE,
  TSCONFIG_JSON_TEMPLATE,
  renderTemplate,
} from "./templates.js";

export interface ScaffoldOptions {
  name: string;
  dest: string;
}

/**
 * Scaffold a new Rotiv project at the given destination.
 * This is the TypeScript equivalent of `rotiv new` for use in `create-rotiv`.
 * Phase 1: delegates to the Rust CLI binary. If not found, scaffolds directly.
 */
export async function scaffoldProject(options: ScaffoldOptions): Promise<void> {
  const { name, dest } = options;
  const createdAt = new Date().toISOString();
  const vars = { project_name: name, created_at: createdAt };

  const dirs = [
    path.join(dest, ".rotiv"),
    path.join(dest, "app", "routes"),
    path.join(dest, "app", "models"),
  ];

  for (const dir of dirs) {
    fs.mkdirSync(dir, { recursive: true });
  }

  const files: Array<[string, string]> = [
    [path.join(dest, ".rotiv", "spec.json"), renderTemplate(SPEC_JSON_TEMPLATE, vars)],
    [path.join(dest, ".rotiv", "context.md"), renderTemplate(CONTEXT_MD_TEMPLATE, vars)],
    [path.join(dest, "app", "routes", "index.tsx"), renderTemplate(ROUTES_INDEX_TSX_TEMPLATE, vars)],
    [path.join(dest, "app", "models", ".gitkeep"), ""],
    [path.join(dest, "package.json"), renderTemplate(PACKAGE_JSON_TEMPLATE, vars)],
    [path.join(dest, "tsconfig.json"), renderTemplate(TSCONFIG_JSON_TEMPLATE, vars)],
  ];

  for (const [filePath, content] of files) {
    fs.writeFileSync(filePath, content, "utf8");
  }
}
