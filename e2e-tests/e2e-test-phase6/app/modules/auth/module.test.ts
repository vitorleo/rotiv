// FRAMEWORK: Integration test for the auth module.
// Tests that auth middleware redirects unauthenticated and passes authenticated requests.
import { describe, it, expect } from "vitest";
import { authMiddleware, login, logout, getCurrentUser } from "./index.js";

describe("auth module", () => {
  it("exports authMiddleware as a function", () => {
    expect(typeof authMiddleware).toBe("function");
  });

  it("authMiddleware redirects when no userId in session", async () => {
    const mw = authMiddleware({ redirectTo: "/login" });
    const mockCtx = {
      session: { get: (_k: string) => undefined },
      request: new Request("http://localhost/protected"),
    };
    const next = async () => new Response("ok");
    // @ts-expect-error partial ctx
    const response = await mw(mockCtx, next);
    expect(response.status).toBe(302);
    expect(response.headers.get("Location")).toBe("/login");
  });

  it("authMiddleware calls next when userId present", async () => {
    const mw = authMiddleware();
    const mockCtx = {
      session: { get: (k: string) => k === "userId" ? "42" : undefined },
      request: new Request("http://localhost/protected"),
    };
    const next = async () => new Response("ok");
    // @ts-expect-error partial ctx
    const response = await mw(mockCtx, next);
    expect(response.status).toBe(200);
  });

  it("login sets userId in session", async () => {
    const stored: Record<string, unknown> = {};
    const ctx = { session: { set: (k: string, v: unknown) => { stored[k] = v; } } };
    await login(ctx, "99");
    expect(stored["userId"]).toBe("99");
  });

  it("logout calls session.destroy", async () => {
    let destroyed = false;
    const ctx = { session: { destroy: async () => { destroyed = true; } } };
    await logout(ctx);
    expect(destroyed).toBe(true);
  });

  it("getCurrentUser returns null when no session", () => {
    const ctx = { session: { get: (_k: string) => undefined } };
    expect(getCurrentUser(ctx)).toBeNull();
  });
});
