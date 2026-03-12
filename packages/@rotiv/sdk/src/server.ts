import type { RotivServer, ServerConfig } from "@rotiv/types";

/**
 * Create a Rotiv server instance.
 * Phase 1: stub — throws "Not implemented".
 * Phase 2: connects to the Rust HTTP server via napi-rs.
 */
export function createServer(_config?: Partial<ServerConfig>): RotivServer {
  throw new Error("Not implemented: Phase 2");
}
