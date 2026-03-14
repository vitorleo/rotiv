# Signals

## Explanation
Signals are Rotiv's reactive primitive for fine-grained UI updates. They are currently used for server-side rendering (SSR) and will power client-side hydration in a future phase.

Three primitives are available from `@rotiv/signals`:
- `signal(initialValue)` — creates a reactive value; call it as a function to read, call `.set(value)` to write
- `derived(fn)` — creates a computed value that updates when its signal dependencies change
- `effect(fn)` — runs a side effect whenever its signal dependencies change

In SSR context (current phase), signals compute synchronously during `renderToString()`. Client-side reactivity (DOM patching) is planned for Phase 6.

## Code Example
```typescript
import { signal, derived, effect } from "@rotiv/signals";

// Create a reactive counter
const count = signal(0);

// Derived value — recomputes when count changes
const doubled = derived(() => count() * 2);

// Effect — runs whenever count changes
effect(() => {
  console.log("count is:", count());
});

// Update the signal
count.set(5);
// Effect runs: "count is: 5"
// doubled() === 10
```

## Related
- routes
- context
