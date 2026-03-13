import { Fragment } from "./types.js";
import type { VNode, Children, VNodeType } from "./types.js";

// Void elements — must not have a closing tag
const VOID_ELEMENTS = new Set([
  "area", "base", "br", "col", "embed", "hr", "img", "input",
  "link", "meta", "param", "source", "track", "wbr",
]);

// HTML attribute name remapping
const ATTR_MAP: Record<string, string> = {
  className: "class",
  htmlFor: "for",
  tabIndex: "tabindex",
  readOnly: "readonly",
  autoComplete: "autocomplete",
  autoFocus: "autofocus",
  crossOrigin: "crossorigin",
  encType: "enctype",
  accessKey: "accesskey",
  contentEditable: "contenteditable",
  spellCheck: "spellcheck",
};

// Boolean HTML attributes — emit name only when true, omit when false
const BOOLEAN_ATTRS = new Set([
  "checked", "disabled", "readonly", "required", "selected",
  "autofocus", "autoplay", "controls", "default", "defer",
  "formnovalidate", "hidden", "ismap", "loop", "multiple",
  "muted", "nomodule", "novalidate", "open", "reversed",
  "scoped", "seamless",
]);

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function serializeStyle(style: unknown): string {
  if (typeof style === "string") return style;
  if (typeof style !== "object" || style === null) return "";
  return Object.entries(style as Record<string, unknown>)
    .filter(([, v]) => v != null && v !== false && v !== "")
    .map(([k, v]) => {
      // camelCase → kebab-case
      const prop = k.replace(/([A-Z])/g, (m) => `-${m.toLowerCase()}`);
      return `${prop}: ${v}`;
    })
    .join("; ");
}

function serializeProps(props: Record<string, unknown>): string {
  let result = "";
  for (const [rawKey, val] of Object.entries(props)) {
    if (rawKey === "children" || rawKey === "key" || rawKey === "ref") continue;
    if (rawKey === "dangerouslySetInnerHTML") continue;

    // Omit event handlers during SSR
    if (rawKey.startsWith("on") && rawKey.length > 2 && rawKey[2]?.toUpperCase() === rawKey[2]) {
      continue;
    }

    const key = ATTR_MAP[rawKey] ?? rawKey;

    if (rawKey === "style") {
      const css = serializeStyle(val);
      if (css) result += ` style="${escapeHtml(css)}"`;
      continue;
    }

    if (BOOLEAN_ATTRS.has(key)) {
      if (val) result += ` ${key}`;
      continue;
    }

    if (val == null || val === false) continue;
    if (val === true) {
      result += ` ${key}`;
      continue;
    }

    result += ` ${key}="${escapeHtml(String(val))}"`;
  }
  return result;
}

/**
 * Render a VNode (or primitive) to an HTML string.
 * Safe for server-side use — escapes text content, omits event handlers.
 */
export function renderToString(node: unknown): string {
  if (node == null || node === false || node === true) return "";

  if (typeof node === "string") return escapeHtml(node);
  if (typeof node === "number") return String(node);

  if (Array.isArray(node)) {
    return (node as unknown[]).map(renderToString).join("");
  }

  // VNode shape check
  if (typeof node !== "object" || !("type" in node) || !("props" in node)) {
    return escapeHtml(String(node));
  }

  const vnode = node as VNode;
  const { type, props } = vnode;

  // Fragment — render children only
  if (type === Fragment) {
    return renderToString(props["children"] as Children);
  }

  // Component function — call and recurse
  if (typeof type === "function") {
    const result = (type as (props: Record<string, unknown>) => unknown)(props);
    return renderToString(result);
  }

  // HTML element string
  if (typeof type === "string") {
    const tag = type;
    const attrs = serializeProps(props);

    if (VOID_ELEMENTS.has(tag)) {
      return `<${tag}${attrs}>`;
    }

    // dangerouslySetInnerHTML
    const dangerous = props["dangerouslySetInnerHTML"] as { __html?: string } | undefined;
    if (dangerous?.__html != null) {
      return `<${tag}${attrs}>${dangerous.__html}</${tag}>`;
    }

    const children = renderToString(props["children"] as Children);
    return `<${tag}${attrs}>${children}</${tag}>`;
  }

  return "";
}
