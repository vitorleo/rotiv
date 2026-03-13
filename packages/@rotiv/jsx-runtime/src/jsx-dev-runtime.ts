import { Fragment as _Fragment } from "./types.js";
import type { VNode, Props, VNodeType } from "./types.js";

export { _Fragment as Fragment };

// Dev runtime — same shape as jsx but with extra source-map parameters that TypeScript passes
export function jsxDEV(
  type: VNodeType,
  props: Record<string, unknown>,
  key?: string | null,
  _isStaticChildren?: boolean,
  _source?: unknown,
  _self?: unknown
): VNode {
  return { type, props: props as Props, key: key ?? null };
}
