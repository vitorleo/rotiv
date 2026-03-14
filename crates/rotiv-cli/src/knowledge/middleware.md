# Middleware

## Explanation
Middleware functions run before the loader or action for a route. They receive a `RequestContext` and a `next` function. Call `await next()` to continue the request chain; return without calling `next()` to short-circuit (e.g. for auth checks).

Middleware is declared per-route as an array in `defineRoute({ middleware: [...] })`. Middleware functions run in order.

## Code Example
```typescript
import { defineRoute } from "@rotiv/sdk";
import type { MiddlewareFn } from "@rotiv/types";

// Auth middleware — short-circuits if no Authorization header
const requireAuth: MiddlewareFn = async (ctx, next) => {
  const token = ctx.headers.get("Authorization");
  if (!token) {
    // Return early without calling next() — route will not be invoked
    throw new Error("Unauthorized");
  }
  await next();
};

// Logging middleware
const logRequest: MiddlewareFn = async (ctx, next) => {
  console.log(`[${new Date().toISOString()}] ${ctx.request.method} ${ctx.request.url}`);
  await next();
};

export default defineRoute({
  path: "/admin",
  middleware: [logRequest, requireAuth],

  async loader(ctx) {
    return { message: "Welcome to admin" };
  },

  component({ data }) {
    return <main><h1>{data.message}</h1></main>;
  },
});
```

## Related
- routes
- loader
- action
