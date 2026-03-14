// FRAMEWORK: Module entry point — export your module's public API here.
// Module: my-custom-module2
// provides: ["my-custom-module2"]
// configures: ["middleware"]
// tier: "slot"
//
// A module exports capabilities for use across your application:
//   - MiddlewareFn functions — injected into route pipelines via the middleware array
//   - Utility functions — imported directly by route files
//   - Type definitions — for route-file type safety
//
// Usage in a route:
//   import { my-custom-module2Middleware } from "../modules/my-custom-module2/index.js";
//   export default defineRoute({
//     path: "/protected",
//     middleware: [my-custom-module2Middleware],
//     async loader(ctx) { ... },
//   });
import type { MiddlewareFn } from "@rotiv/types";

// FRAMEWORK: Replace this stub with your module's actual implementation.
export const my-custom-module2Middleware: MiddlewareFn = async (ctx, next) => {
  // TODO: implement my-custom-module2 logic here
  await next();
};
