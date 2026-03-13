import type { SignalPair } from "./types.js";

/**
 * Create a reactive signal.
 *
 * Phase 3 (SSR): synchronous value container. Setting the signal updates the
 * value immediately; no subscription notifications are fired.
 * Phase 4 will add client-side DOM subscriptions.
 *
 * @param initial - The initial value.
 * @returns A [getter, setter] tuple.
 *
 * @example
 * const [count, setCount] = signal(0);
 * setCount(1);
 * console.log(count()); // 1
 */
export function signal<T>(initial: T): SignalPair<T> {
  let value = initial;

  const get = (): T => value;

  const set = (next: T | ((prev: T) => T)): void => {
    value = typeof next === "function"
      ? (next as (prev: T) => T)(value)
      : next;
    // Phase 4: notify subscribers here
  };

  return [get, set];
}
