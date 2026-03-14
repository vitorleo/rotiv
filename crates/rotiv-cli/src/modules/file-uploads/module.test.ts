// FRAMEWORK: Integration test for the file-uploads module.
// Tests that fileUploadsMiddleware parses multipart uploads and rejects oversized files.
import { describe, it, expect } from "vitest";
import { fileUploadsMiddleware } from "./index.js";

describe("file-uploads module", () => {
  it("exports fileUploadsMiddleware as a function", () => {
    expect(typeof fileUploadsMiddleware).toBe("function");
  });

  it("middleware returns a MiddlewareFn", () => {
    const mw = fileUploadsMiddleware();
    expect(typeof mw).toBe("function");
  });

  it("passes through non-multipart requests and injects empty ctx.files", async () => {
    const mw = fileUploadsMiddleware();
    const mockCtx = {
      request: new Request("http://localhost/upload", { method: "POST" }),
      files: null as unknown,
    };
    const next = async (ctx: typeof mockCtx) => {
      expect(ctx.files).toEqual({});
      return new Response("ok");
    };
    // @ts-expect-error partial ctx
    const response = await mw(mockCtx, next);
    expect(response.status).toBe(200);
  });

  it("rejects files exceeding maxFileSizeBytes", async () => {
    const mw = fileUploadsMiddleware({ maxFileSizeBytes: 10 });
    const formData = new FormData();
    formData.append("avatar", new File([new Uint8Array(100)], "big.png", { type: "image/png" }));
    const mockCtx = {
      request: new Request("http://localhost/upload", {
        method: "POST",
        body: formData,
      }),
      files: null as unknown,
    };
    const next = async () => new Response("ok");
    // @ts-expect-error partial ctx
    const response = await mw(mockCtx, next);
    expect(response.status).toBe(413);
  });
});
