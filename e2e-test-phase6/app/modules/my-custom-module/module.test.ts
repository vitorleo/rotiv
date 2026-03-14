// FRAMEWORK: Integration test stubs for the my-custom-module module.
// Replace the stub assertions with real tests for your implementation.
// Run with: pnpm test (or your test runner of choice)
import { my-custom-moduleMiddleware } from "./index.js";

describe("my-custom-module module", () => {
  it("exports my-custom-moduleMiddleware as a function", () => {
    expect(typeof my-custom-moduleMiddleware).toBe("function");
  });

  it("middleware calls next()", async () => {
    let nextCalled = false;
    const mockCtx = {} as Parameters<typeof my-custom-moduleMiddleware>[0];
    await my-custom-moduleMiddleware(mockCtx, async () => {
      nextCalled = true;
    });
    expect(nextCalled).toBe(true);
  });
});
