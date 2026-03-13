import { renderToString as jsxRenderToString } from "@rotiv/jsx-runtime";
import type { VNode } from "@rotiv/jsx-runtime";

function isVNode(value: unknown): value is VNode {
  return (
    value !== null &&
    typeof value === "object" &&
    "type" in value &&
    "props" in value &&
    "key" in value
  );
}

/**
 * Phase 3 renderToString.
 *
 * Dispatches to @rotiv/jsx-runtime for VNode output (JSX components),
 * with Phase 2 backward compatibility for plain HTML string returns.
 */
export function renderToString(
  component: ((props: { data: unknown }) => unknown) | undefined,
  props: { data: unknown }
): string {
  if (!component) return "";
  const result = component(props);

  // Phase 3: VNode from @rotiv/jsx-runtime (JSX-compiled component)
  if (isVNode(result)) {
    return jsxRenderToString(result);
  }

  // Phase 2 backward compat: plain HTML string
  if (typeof result === "string") return result;

  // API-only (no component body expected) — should not normally be reached
  return JSON.stringify(result);
}

export function wrapHtml(body: string, title = "Rotiv"): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${title}</title>
</head>
<body>
${body}
</body>
</html>`;
}
