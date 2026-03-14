// FRAMEWORK: Integration test stubs for the {{module_name}} module.
// Replace the stub assertions with real tests for your implementation.
// Run with: pnpm test (or your test runner of choice)
import { {{module_name}}Middleware } from "./index.js";

describe("{{module_name}} module", () => {
  it("exports {{module_name}}Middleware as a function", () => {
    expect(typeof {{module_name}}Middleware).toBe("function");
  });

  it("middleware calls next()", async () => {
    let nextCalled = false;
    const mockCtx = {} as Parameters<typeof {{module_name}}Middleware>[0];
    await {{module_name}}Middleware(mockCtx, async () => {
      nextCalled = true;
    });
    expect(nextCalled).toBe(true);
  });
});
