/**
 * Phase 2 renderToString shim.
 *
 * Route components must return an HTML string in Phase 2.
 * JSX syntax will be supported in Phase 3 after the Rotiv compiler ships.
 *
 * If the component returns a non-string (e.g., a POJO for an API-only route),
 * it is JSON-serialized.
 */
export function renderToString(
  component: ((props: { data: unknown }) => unknown) | undefined,
  props: { data: unknown }
): string {
  if (!component) return "";
  const result = component(props);
  if (typeof result === "string") return result;
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
