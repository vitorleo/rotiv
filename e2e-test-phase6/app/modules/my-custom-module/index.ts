// FRAMEWORK: Module entry point — export your module's public API here.
// Module: my-custom-module
// provides: ["my-custom-module"]
// configures: ["middleware"]
// tier: "slot"
//
// A module exports capabilities for use across your application:
//   - MiddlewareFn functions — injected into route pipelines via the middleware array
//   - Utility functions — imported directly by route files
//   - Type definitions — for route-file type safety
//
// Usage in a route:
//   import { my-custom-moduleMiddleware } from "../modules/my-custom-module/index.js";
//   export default defineRoute({
//     path: "/protected",
//     middleware: [my-custom-moduleMiddleware],
//     async loader(ctx) { ... },
//   });
import type { MiddlewareFn } from "@rotiv/types";

// FRAMEWORK: Replace this stub with your module's actual implementation.
export const my-custom-moduleMiddleware: MiddlewareFn = async (ctx, next) => {
  // TODO: implement my-custom-module logic here
  await next();
};
