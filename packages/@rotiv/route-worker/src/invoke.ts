import { pathToFileURL } from "node:url";
import { toRotivError } from "./errors.js";
import { renderToString, wrapHtml } from "./render.js";

function pathToFileUrl(filePath: string): URL {
  return pathToFileURL(filePath);
}

export interface InvokeRequest {
  route_file: string;
  method: string;
  params: Record<string, string>;
  search_params: string;
  headers: Record<string, string>;
  body: string | null;
}

export interface InvokeResponse {
  status: number;
  headers: Record<string, string>;
  body: string;
}

/**
 * Dynamically import and execute a route file.
 *
 * Cache-busts on every call so edits within a session are picked up
 * without restarting the worker. (The worker is also restarted by the
 * file watcher on changes, but cache-busting ensures correctness even
 * if the watcher event fires slightly late.)
 */
export async function invokeRoute(req: InvokeRequest): Promise<InvokeResponse> {
  // Convert absolute path to file:// URL (required by Node ESM on Windows)
  // and cache-bust so edits within a session are picked up without restart.
  const fileUrl = pathToFileUrl(req.route_file);
  fileUrl.searchParams.set("t", String(Date.now()));
  const moduleUrl = fileUrl.href;

  let routeModule: unknown;
  try {
    routeModule = await import(moduleUrl);
  } catch (err) {
    throw toRotivError(err, req.route_file);
  }

  const route = (routeModule as Record<string, unknown>)["default"];

  if (
    !route ||
    typeof route !== "object" ||
    (route as Record<string, unknown>)["_type"] !== "RouteDefinition"
  ) {
    throw toRotivError(
      new Error(
        `File does not export a RouteDefinition as default export. ` +
          `Make sure to use defineRoute() from @rotiv/sdk.`
      ),
      req.route_file
    );
  }

  const routeDef = route as {
    loader?: (ctx: unknown) => Promise<unknown> | unknown;
    action?: (ctx: unknown) => Promise<unknown> | unknown;
    component?: (props: { data: unknown }) => unknown;
  };

  const isGet = req.method.toUpperCase() === "GET";

  try {
    if (isGet && routeDef.loader) {
      // Build loader context
      const ctx = buildContext(req);
      const data = await routeDef.loader(ctx);

      if (routeDef.component) {
        const html = renderToString(routeDef.component, { data });
        return {
          status: 200,
          headers: { "content-type": "text/html; charset=utf-8" },
          body: wrapHtml(html),
        };
      }

      // No component — return data as JSON
      return {
        status: 200,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(data),
      };
    }

    if (!isGet && routeDef.action) {
      const ctx = buildContext(req);
      const result = await routeDef.action(ctx);

      if (result instanceof Response) {
        const text = await result.text();
        const responseHeaders: Record<string, string> = {};
        result.headers.forEach((v, k) => {
          responseHeaders[k] = v;
        });
        return { status: result.status, headers: responseHeaders, body: text };
      }

      return {
        status: 200,
        headers: { "content-type": "application/json" },
        body: JSON.stringify(result),
      };
    }

    // No loader/action — render component with null data or return 204
    if (routeDef.component) {
      const html = renderToString(routeDef.component, { data: null });
      return {
        status: 200,
        headers: { "content-type": "text/html; charset=utf-8" },
        body: wrapHtml(html),
      };
    }

    return { status: 204, headers: {}, body: "" };
  } catch (err) {
    throw toRotivError(err, req.route_file);
  }
}

function buildContext(req: InvokeRequest): unknown {
  return {
    params: req.params,
    searchParams: new URLSearchParams(req.search_params.replace(/^\?/, "")),
    headers: new Headers(req.headers),
    method: req.method,
    request: new Request(`http://localhost${req.search_params}`, {
      method: req.method,
      headers: req.headers,
      body: req.body,
    }),
  };
}
