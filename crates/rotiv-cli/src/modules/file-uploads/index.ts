// FRAMEWORK: First-party file-uploads module.
// Provides multipart form data parsing and file handling via fileUploadsMiddleware.
// After this middleware runs, ctx.files contains uploaded files keyed by field name.
import type { MiddlewareFn } from "@rotiv/sdk";

export interface UploadedFile {
  name: string;
  type: string;
  size: number;
  bytes: Uint8Array;
}

export interface FileUploadsOptions {
  maxFileSizeBytes?: number; // default 10MB
  allowedTypes?: string[];   // e.g. ["image/jpeg", "image/png"]
}

// FRAMEWORK: fileUploadsMiddleware parses multipart/form-data and injects ctx.files.
// Access uploaded files via ctx.files["fieldName"] in your action().
export function fileUploadsMiddleware(options: FileUploadsOptions = {}): MiddlewareFn {
  const { maxFileSizeBytes = 10 * 1024 * 1024, allowedTypes } = options;
  return async (ctx, next) => {
    const contentType = ctx.request.headers.get("content-type") ?? "";
    ctx.files = {};

    if (contentType.startsWith("multipart/form-data")) {
      const formData = await ctx.request.formData();
      for (const [key, value] of formData.entries()) {
        if (value instanceof File) {
          if (value.size > maxFileSizeBytes) {
            return new Response(
              JSON.stringify({ error: `File '${key}' exceeds size limit` }),
              { status: 413, headers: { "Content-Type": "application/json" } }
            );
          }
          if (allowedTypes && !allowedTypes.includes(value.type)) {
            return new Response(
              JSON.stringify({ error: `File type '${value.type}' is not allowed` }),
              { status: 415, headers: { "Content-Type": "application/json" } }
            );
          }
          ctx.files[key] = {
            name: value.name,
            type: value.type,
            size: value.size,
            bytes: new Uint8Array(await value.arrayBuffer()),
          };
        }
      }
    }

    return next(ctx);
  };
}
