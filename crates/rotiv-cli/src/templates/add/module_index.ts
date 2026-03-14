// FRAMEWORK: Module entry point — export your module's public API here.
// Module: {{module_name}}
// provides: ["{{module_name}}"]
// configures: ["middleware"]
// tier: "slot"
//
// A module exports capabilities for use across your application:
//   - MiddlewareFn functions — injected into route pipelines via the middleware array
//   - Utility functions — imported directly by route files
//   - Type definitions — for route-file type safety
//
// Usage in a route:
//   import { {{module_name}}Middleware } from "../modules/{{module_name}}/index.js";
//   export default defineRoute({
//     path: "/protected",
//     middleware: [{{module_name}}Middleware],
//     async loader(ctx) { ... },
//   });
import type { MiddlewareFn } from "@rotiv/types";

// FRAMEWORK: Replace this stub with your module's actual implementation.
export const {{module_name}}Middleware: MiddlewareFn = async (ctx, next) => {
  // TODO: implement {{module_name}} logic here
  await next();
};
