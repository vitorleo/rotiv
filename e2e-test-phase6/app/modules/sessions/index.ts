// FRAMEWORK: First-party sessions module.
// Provides cookie-based session management via sessionsMiddleware.
// Add to your route via the middleware array in defineRoute().
import type { MiddlewareFn, SessionStore } from "@rotiv/sdk";

export interface SessionOptions {
  secret: string;
  cookieName?: string;
  maxAge?: number; // seconds, default 86400 (1 day)
}

// FRAMEWORK: sessionsMiddleware injects ctx.session into every request.
// ctx.session.get(key) / ctx.session.set(key, value) / ctx.session.destroy()
export function sessionsMiddleware(options: SessionOptions): MiddlewareFn {
  const { secret, cookieName = "sid", maxAge = 86400 } = options;
  return async (ctx, next) => {
    const store: SessionStore = ctx.runtime.sessionStore;
    const cookieHeader = ctx.request.headers.get("cookie") ?? "";
    const sid = parseCookie(cookieHeader, cookieName);
    const data = sid ? await store.get(sid) : {};

    ctx.session = {
      get: (key: string) => data[key],
      set: (key: string, value: unknown) => { data[key] = value; },
      destroy: async () => { if (sid) await store.delete(sid); },
    };

    const response = await next(ctx);

    // Persist session
    const newSid = sid ?? crypto.randomUUID();
    await store.set(newSid, data, maxAge);
    const cookie = `${cookieName}=${newSid}; HttpOnly; SameSite=Lax; Max-Age=${maxAge}`;
    response.headers.append("Set-Cookie", cookie);
    return response;
  };
}

function parseCookie(header: string, name: string): string | undefined {
  for (const pair of header.split(";")) {
    const [k, v] = pair.trim().split("=");
    if (k === name) return v;
  }
  return undefined;
}
