import type { ProjectConfig } from "./config.js";
import type { RotivDb } from "./db.js";

export type { ProjectConfig };

/**
 * The context object available inside a request handler.
 */
export interface RequestContext {
  request: Request;
  params: Record<string, string>;
  searchParams: URLSearchParams;
  headers: Headers;
}

/**
 * The context passed to a route's loader function.
 */
export interface LoaderContext extends RequestContext {
  readonly method: "GET";
  readonly db: RotivDb;
}

/**
 * The context passed to a route's action function.
 */
export interface ActionContext extends RequestContext {
  readonly method: "POST" | "PUT" | "PATCH" | "DELETE";
  readonly db: RotivDb;
  json(): Promise<unknown>;
  formData(): Promise<FormData>;
}

/**
 * A middleware function. Call `next()` to pass control to the next middleware.
 */
export type MiddlewareFn = (
  ctx: RequestContext,
  next: () => Promise<void>
) => Promise<void>;

/**
 * Configuration object passed to `defineRoute()`.
 */
export interface RouteConfig<TData = unknown> {
  path: string;
  loader?: (ctx: LoaderContext) => Promise<TData> | TData;
  action?: (ctx: ActionContext) => Promise<Response | TData> | Response | TData;
  middleware?: MiddlewareFn[];
  component?: (props: { data: TData }) => unknown;
}

/**
 * A fully-resolved route definition, returned by `defineRoute()`.
 */
export interface RouteDefinition<TData = unknown> extends RouteConfig<TData> {
  readonly _type: "RouteDefinition";
}

/**
 * Rotiv server configuration.
 */
export interface ServerConfig {
  port: number;
  host: string;
  routesDir: string;
}

/**
 * The server object returned by `createServer()`.
 * Phase 2 implementation.
 */
export interface RotivServer {
  start(): Promise<void>;
  stop(): Promise<void>;
  readonly port: number;
}
