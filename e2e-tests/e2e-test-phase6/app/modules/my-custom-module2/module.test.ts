// FRAMEWORK: Integration test stubs for the my-custom-module2 module.
// Replace the stub assertions with real tests for your implementation.
// Run with: pnpm test (or your test runner of choice)
import { my-custom-module2Middleware } from "./index.js";

describe("my-custom-module2 module", () => {
  it("exports my-custom-module2Middleware as a function", () => {
    expect(typeof my-custom-module2Middleware).toBe("function");
  });

  it("middleware calls next()", async () => {
    let nextCalled = false;
    const mockCtx = {} as Parameters<typeof my-custom-module2Middleware>[0];
    await my-custom-module2Middleware(mockCtx, async () => {
      nextCalled = true;
    });
    expect(nextCalled).toBe(true);
  });
});
