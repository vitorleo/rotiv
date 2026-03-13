export const Fragment = Symbol.for("rotiv.Fragment");

export type VNodeType = string | typeof Fragment | ((props: Props) => VNode | string | null);

export type Props = Record<string, unknown> & { children?: Children };

export type Children =
  | VNode
  | string
  | number
  | boolean
  | null
  | undefined
  | Children[];

export interface VNode {
  type: VNodeType;
  props: Props;
  key: string | null;
}

// Module-scoped JSX namespace — avoids conflicts with @types/react global namespace
export namespace JSX {
  export type Element = VNode;
  export interface ElementClass {
    props: {};
  }
  export interface ElementAttributesProperty {
    props: {};
  }
  export interface IntrinsicElements {
    [elemName: string]: Record<string, unknown>;
  }
  export interface IntrinsicAttributes {
    key?: string | number | null;
  }
}
