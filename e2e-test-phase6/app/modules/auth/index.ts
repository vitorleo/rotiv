// FRAMEWORK: First-party auth module.
// Provides authentication helpers: authMiddleware (protect routes), login(), logout().
// Requires the sessions module to be installed first.
import type { MiddlewareFn } from "@rotiv/sdk";

export interface AuthOptions {
  redirectTo?: string; // path to redirect unauthenticated requests, default "/login"
}

// FRAMEWORK: authMiddleware protects a route, redirecting to redirectTo if not logged in.
// Requires ctx.session (provided by the sessions module).
export function authMiddleware(options: AuthOptions = {}): MiddlewareFn {
  const { redirectTo = "/login" } = options;
  return async (ctx, next) => {
    const userId = ctx.session?.get("userId");
    if (!userId) {
      return Response.redirect(redirectTo, 302);
    }
    return next(ctx);
  };
}

// FRAMEWORK: login() stores the userId in the session.
// Call from an action() after verifying credentials.
export async function login(ctx: { session: { set: (k: string, v: unknown) => void } }, userId: string | number): Promise<void> {
  ctx.session.set("userId", userId);
}

// FRAMEWORK: logout() destroys the session entirely.
export async function logout(ctx: { session: { destroy: () => Promise<void> } }): Promise<void> {
  await ctx.session.destroy();
}

// FRAMEWORK: getCurrentUser() returns the userId from the session, or null.
export function getCurrentUser(ctx: { session: { get: (k: string) => unknown } }): string | null {
  const id = ctx.session?.get("userId");
  return id != null ? String(id) : null;
}
