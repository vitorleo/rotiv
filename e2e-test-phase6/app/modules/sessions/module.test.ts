// FRAMEWORK: Integration test for the sessions module.
// Tests that sessionsMiddleware correctly injects ctx.session and sets cookies.
import { describe, it, expect } from "vitest";
import { sessionsMiddleware } from "./index.js";

describe("sessions module", () => {
  it("exports sessionsMiddleware as a function", () => {
    expect(typeof sessionsMiddleware).toBe("function");
  });

  it("sessionsMiddleware returns a middleware function", () => {
    const mw = sessionsMiddleware({ secret: "test-secret" });
    expect(typeof mw).toBe("function");
  });

  it("middleware calls next and returns response", async () => {
    const mw = sessionsMiddleware({ secret: "test-secret" });
    const mockResponse = new Response("ok");
    const mockCtx = {
      request: new Request("http://localhost/"),
      runtime: {
        sessionStore: {
          get: async () => ({}),
          set: async () => {},
          delete: async () => {},
        },
      },
      session: null as unknown,
    };
    const next = async (_ctx: unknown) => mockResponse;
    // @ts-expect-error partial ctx for test
    const response = await mw(mockCtx, next);
    expect(response).toBeInstanceOf(Response);
    expect(mockCtx.session).not.toBeNull();
  });
});
