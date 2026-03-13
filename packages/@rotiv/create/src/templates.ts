/**
 * Inline template strings for generated project files.
 * These mirror the Rust embedded templates in rotiv-cli.
 */

export const SPEC_JSON_TEMPLATE = `{
  "$schema": "https://rotiv.dev/spec/v1",
  "version": "1",
  "framework_version": "0.1.0",
  "project": {
    "name": "{{project_name}}",
    "created_at": "{{created_at}}"
  },
  "routes": [],
  "models": [],
  "modules": [],
  "conventions": {
    "routes_dir": "app/routes",
    "models_dir": "app/models",
    "components_dir": "app/components"
  }
}
`;

export const CONTEXT_MD_TEMPLATE = `# {{project_name}}

Project created with Rotiv framework on {{created_at}}.

## Description

_Add a description of your project here._

## Architecture

_Describe the architecture and key design decisions._
`;

export const ROUTES_INDEX_TSX_TEMPLATE = `import { defineRoute } from "@rotiv/sdk";

// FRAMEWORK: Component returns JSX, compiled by @swc/core to @rotiv/jsx-runtime
// calls during \`rotiv dev\`. Rendering is server-side only in Phase 3.
// Client-side interactivity (signals, DOM binding) arrives in Phase 4.
export default defineRoute({
  path: "/",
  async loader() {
    return { message: "Hello from {{project_name}}!" };
  },
  component({ data }) {
    return (
      <main>
        <h1>{data.message}</h1>
        <p>Edit <code>app/routes/index.tsx</code> to get started.</p>
      </main>
    );
  },
});
`;

export const PACKAGE_JSON_TEMPLATE = `{
  "name": "{{project_name}}",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "rotiv dev",
    "build": "rotiv build",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "@rotiv/jsx-runtime": "^0.1.0",
    "@rotiv/sdk": "^0.1.0",
    "@rotiv/signals": "^0.1.0"
  },
  "devDependencies": {
    "@rotiv/types": "^0.1.0",
    "tsx": "^4.0.0",
    "typescript": "^5.0.0"
  }
}
`;

export const TSCONFIG_JSON_TEMPLATE = `{
  "extends": "@rotiv/types/tsconfig.base.json",
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "@rotiv",
    "outDir": "dist",
    "rootDir": "."
  },
  "include": ["app/**/*"],
  "exclude": ["node_modules", "dist"]
}
`;

export function renderTemplate(template: string, vars: Record<string, string>): string {
  return Object.entries(vars).reduce(
    (result, [key, value]) => result.replaceAll(`{{${key}}}`, value),
    template
  );
}
