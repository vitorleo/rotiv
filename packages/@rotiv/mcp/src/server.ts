#!/usr/bin/env node
/**
 * @rotiv/mcp — MCP (Model Context Protocol) server for the Rotiv CLI.
 *
 * Reads JSON-RPC 2.0 requests from stdin (one per line), dispatches to the
 * `rotiv` binary as a subprocess with --json flag, and writes responses to stdout.
 *
 * Usage (in MCP client config):
 *   {
 *     "mcpServers": {
 *       "rotiv": {
 *         "command": "node",
 *         "args": ["/path/to/@rotiv/mcp/dist/server.js"],
 *         "cwd": "/path/to/your/rotiv/project"
 *       }
 *     }
 *   }
 */

import { spawnSync } from "node:child_process";
import * as readline from "node:readline";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const MANIFEST = JSON.parse(
  readFileSync(join(__dirname, "..", "index.json"), "utf8")
) as ToolManifest;

interface ToolManifest {
  tools: Array<{ name: string; description: string; inputSchema: object }>;
}

interface JsonRpcRequest {
  jsonrpc: "2.0";
  id: number | string;
  method: string;
  params?: Record<string, unknown>;
}

interface JsonRpcResponse {
  jsonrpc: "2.0";
  id: number | string;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

function respond(id: number | string, result: unknown): void {
  const response: JsonRpcResponse = { jsonrpc: "2.0", id, result };
  process.stdout.write(JSON.stringify(response) + "\n");
}

function respondError(id: number | string, code: number, message: string): void {
  const response: JsonRpcResponse = {
    jsonrpc: "2.0",
    id,
    error: { code, message },
  };
  process.stdout.write(JSON.stringify(response) + "\n");
}

function toolNameToArgs(toolName: string, params: Record<string, unknown>): string[] {
  // Convert MCP tool names to rotiv CLI args
  // rotiv_add_route { path } → ["add", "route", "<path>"]
  // rotiv_validate { fix } → ["validate", "--fix"]
  // etc.

  switch (toolName) {
    case "rotiv_new":
      return ["new", String(params.name)];

    case "rotiv_info":
      return ["info"];

    case "rotiv_add_route":
      return ["add", "route", String(params.path)];

    case "rotiv_add_model":
      return ["add", "model", String(params.name)];

    case "rotiv_add_module":
      return ["add", "module", String(params.name)];

    case "rotiv_spec_sync":
      return ["spec-sync"];

    case "rotiv_validate": {
      const args = ["validate"];
      if (params.fix) args.push("--fix");
      return args;
    }

    case "rotiv_explain":
      return ["explain", String(params.topic)];

    case "rotiv_context_regen":
      return ["context-regen"];

    case "rotiv_diff_impact":
      return ["diff-impact", String(params.file)];

    case "rotiv_migrate": {
      const args = ["migrate"];
      if (params.generate_only) args.push("--generate-only");
      if (params.check) args.push("--check");
      return args;
    }

    case "rotiv_deploy": {
      const args = ["deploy"];
      if (params.dry_run) args.push("--dry-run");
      if (params.skip_build) args.push("--skip-build");
      if (params.host) args.push("--host", String(params.host));
      if (params.user) args.push("--user", String(params.user));
      if (params.path) args.push("--path", String(params.path));
      if (params.service) args.push("--service", String(params.service));
      return args;
    }

    default:
      return [];
  }
}

function handleToolCall(
  toolName: string,
  params: Record<string, unknown>
): unknown {
  const args = toolNameToArgs(toolName, params);
  if (args.length === 0) {
    return { error: `Unknown tool: ${toolName}` };
  }

  const result = spawnSync("rotiv", [...args, "--json"], {
    encoding: "utf8",
    cwd: process.cwd(),
  });

  const stdout = result.stdout?.trim() ?? "";
  const stderr = result.stderr?.trim() ?? "";

  try {
    const parsed = JSON.parse(stdout);
    return {
      content: [{ type: "text", text: JSON.stringify(parsed, null, 2) }],
    };
  } catch {
    // Not JSON — return as plain text
    const output = stdout || stderr || "Command produced no output";
    return {
      content: [{ type: "text", text: output }],
      isError: result.status !== 0,
    };
  }
}

function handleRequest(req: JsonRpcRequest): void {
  const { id, method, params = {} } = req;

  switch (method) {
    case "initialize":
      respond(id, {
        protocolVersion: "2024-11-05",
        capabilities: { tools: {} },
        serverInfo: { name: "rotiv", version: "0.1.0" },
      });
      break;

    case "tools/list":
      respond(id, { tools: MANIFEST.tools });
      break;

    case "tools/call": {
      const toolName = params.name as string;
      const toolParams = (params.arguments ?? {}) as Record<string, unknown>;
      const result = handleToolCall(toolName, toolParams);
      respond(id, result);
      break;
    }

    default:
      respondError(id, -32601, `Method not found: ${method}`);
  }
}

// --- Main loop: read newline-delimited JSON from stdin ---
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });

rl.on("line", (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;

  try {
    const req = JSON.parse(trimmed) as JsonRpcRequest;
    handleRequest(req);
  } catch (e) {
    // Parse error
    process.stdout.write(
      JSON.stringify({
        jsonrpc: "2.0",
        id: null,
        error: { code: -32700, message: "Parse error" },
      }) + "\n"
    );
  }
});

rl.on("close", () => {
  process.exit(0);
});
