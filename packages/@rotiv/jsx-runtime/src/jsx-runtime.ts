import { Fragment as _Fragment } from "./types.js";
import type { VNode, Props, VNodeType } from "./types.js";

export { _Fragment as Fragment };
export type { VNode, Props };

export function jsx(
  type: VNodeType,
  props: Record<string, unknown>,
  key?: string | null
): VNode {
  return { type, props: props as Props, key: key ?? null };
}

// jsxs is identical to jsx at runtime — TypeScript uses it as a hint for static children arrays
export const jsxs = jsx;
