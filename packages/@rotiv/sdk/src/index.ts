// Router
export { defineRoute } from "./router.js";

// Server
export { createServer } from "./server.js";

// Context
export type { RequestContext, LoaderContext, ActionContext } from "./context.js";

// Middleware
export type { MiddlewareFn } from "./middleware.js";

// Errors
export { RotivRuntimeError } from "./errors.js";
export type { RotivError } from "./errors.js";

// Re-export core types for convenience
export type {
  RouteDefinition,
  RouteConfig,
  ServerConfig,
  RotivServer,
} from "@rotiv/types";
