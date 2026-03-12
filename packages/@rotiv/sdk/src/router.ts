import type { RouteConfig, RouteDefinition } from "@rotiv/types";

/**
 * Define a route. Returns a RouteDefinition with the `_type` brand.
 * Phase 1: type-safe stub. Phase 2: wires into the Rust HTTP server.
 */
export function defineRoute<TData = unknown>(
  config: RouteConfig<TData>
): RouteDefinition<TData> {
  return {
    ...config,
    _type: "RouteDefinition",
  } as RouteDefinition<TData>;
}
