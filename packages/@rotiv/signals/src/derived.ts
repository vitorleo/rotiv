import type { Getter } from "./types.js";

/**
 * Create a derived (computed) value.
 *
 * Phase 3 (SSR): computes once eagerly and memoizes the result.
 * Phase 4 will add dependency tracking and lazy re-computation on the client.
 *
 * @param fn - A function that reads one or more signals and returns a derived value.
 * @returns A getter that returns the computed value.
 *
 * @example
 * const [count] = signal(5);
 * const doubled = derived(() => count() * 2);
 * console.log(doubled()); // 10
 */
export function derived<T>(fn: () => T): Getter<T> {
  // Phase 3: compute once, memoize. Phase 4: track deps, recompute on change.
  const value = fn();
  return () => value;
}
