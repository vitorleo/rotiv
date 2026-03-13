/**
 * @rotiv/route-worker — Internal HTTP server that executes TypeScript route files.
 *
 * This process is spawned by the Rotiv dev server (rotiv-core) and communicates
 * over HTTP on localhost. It is NOT a public-facing server.
 *
 * Endpoints:
 *   GET  /_rotiv/health   — health check, used by wait_ready()
 *   POST /_rotiv/invoke   — execute a route file's loader/action
 *
 * Phase 2 only. Replaced by the SWC compiler in Phase 3.
 */
import express from "express";
import { invokeRoute } from "./invoke.js";
import { toRotivError } from "./errors.js";
import { initDb } from "./db.js";

const PORT = parseInt(process.env["ROTIV_WORKER_PORT"] ?? "3001", 10);

const app = express();
app.use(express.json({ limit: "10mb" }));

// Health check — used by rotiv-core worker.rs wait_ready()
app.get("/_rotiv/health", (_req, res) => {
  res.json({ ok: true });
});

// Route invocation endpoint
app.post("/_rotiv/invoke", async (req, res) => {
  const body = req.body as {
    route_file?: string;
    method?: string;
    params?: Record<string, string>;
    search_params?: string;
    headers?: Record<string, string>;
    body?: string | null;
  };

  if (!body.route_file || !body.method) {
    res.status(400).json({
      error: {
        code: "E_INVALID_REQUEST",
        message: "route_file and method are required",
      },
    });
    return;
  }

  try {
    const result = await invokeRoute({
      route_file: body.route_file,
      method: body.method,
      params: body.params ?? {},
      search_params: body.search_params ?? "",
      headers: body.headers ?? {},
      body: body.body ?? null,
    });
    res.status(result.status).set(result.headers).send(result.body);
  } catch (err) {
    const rotivErr = typeof err === "object" && err !== null && "code" in err
      ? err
      : toRotivError(err, body.route_file ?? "unknown");
    res.status(500).json({ error: rotivErr });
  }
});

const projectDir = process.env["ROTIV_PROJECT_DIR"] ?? process.cwd();
await initDb(projectDir);

app.listen(PORT, "127.0.0.1", () => {
  console.log(`[route-worker] listening on port ${PORT}`);
});
