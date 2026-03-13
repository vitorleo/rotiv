import type { Disposer } from "./types.js";

/**
 * Run a side effect that depends on signals.
 *
 * Phase 3 (SSR): runs the function once immediately and returns a no-op disposer.
 * Phase 4 will re-run the effect whenever its signal dependencies change.
 *
 * @param fn - The effect function. May return a cleanup function.
 * @returns A disposer that stops the effect.
 *
 * @example
 * const [count] = signal(0);
 * const stop = effect(() => console.log("count is", count()));
 * stop(); // unsubscribe
 */
export function effect(fn: () => void | Disposer): Disposer {
  // Phase 3: run once, return cleanup if provided.
  const cleanup = fn();
  return typeof cleanup === "function" ? cleanup : () => {};
}
