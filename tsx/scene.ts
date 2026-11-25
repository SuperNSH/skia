// The scene behind a <Canvas>: each component makes a node kept in Rust
// when it runs (once), a changing prop is an effect that sets that prop,
// and a change asks the canvas for a frame. No tree is compared or walked
// in JS.

import { computed, effect, type JSX, untracked } from "nsh";
import { SkNode } from "nsh-skia/native";

export class SceneNode {
  readonly native: SkNode;
  parent: SceneNode | null = null;
  /** The root's: a change under it. */
  onChange: (() => void) | null = null;

  constructor(readonly kind: string) {
    this.native = new SkNode(kind);
  }

  set(name: string, value: unknown): void {
    this.native.set(name, value === undefined ? null : value);
    this.changed();
  }

  setChildren(children: SceneNode[]): void {
    for (const child of children) child.parent = this;
    this.native.setChildren(children.map((c) => c.native));
    this.changed();
  }

  changed(): void {
    let node: SceneNode = this;
    while (node.parent) node = node.parent;
    node.onChange?.();
  }
}

/** What a component's children are once read: nodes, arrays, and accessors of them. */
type Tree = SceneNode | Tree[] | (() => Tree) | null;

/** `children` with each function (a condition, `<If>`, `<Each>`) kept as a computed, so it makes its nodes once. */
const prepare = (children: unknown): Tree => {
  if (children === null || children === undefined || typeof children === "boolean") return null;
  if (children instanceof SceneNode) return children;
  if (Array.isArray(children)) return children.map(prepare);
  if (typeof children === "function") return computed(() => prepare((children as () => unknown)()));
  if (typeof children === "string" && children.trim() === "") return null;
  if (typeof children === "object") {
    // NSH's <Each> leaves rows past its 32nd as placeholder views, which it
    // renders when they are laid out: never, in a scene.
    throw new Error(
      "a <Canvas> holds Skia components only (<Circle>, <Group>, ...), not views; for more than 32 rows use <Repeat> in place of <Each>",
    );
  }
  throw new Error(`a <Canvas> holds Skia components only (<Circle>, <Group>, ...), not the ${typeof children} ${String(children)}`);
};

const flatten = (tree: Tree, out: SceneNode[] = []): SceneNode[] => {
  if (tree instanceof SceneNode) out.push(tree);
  else if (Array.isArray(tree)) for (const t of tree) flatten(t, out);
  else if (typeof tree === "function") flatten(tree(), out);
  return out;
};

const dynamic = (tree: Tree): boolean => typeof tree === "function" || (Array.isArray(tree) && tree.some(dynamic));

/** Gives `node` children, and again when they change. */
export function adopt(node: SceneNode, children: unknown): void {
  const tree = untracked(() => prepare(children));
  let last = untracked(() => flatten(tree));
  if (last.length > 0) node.setChildren(last);
  if (!dynamic(tree)) return;
  effect(() => {
    const now = flatten(tree);
    if (now.length === last.length && now.every((n, i) => n === last[i])) return;
    last = now;
    untracked(() => node.setChildren(now));
  });
}

type Props = Record<string, unknown>;

/** A component's children: an accessor of them when they are an expression (`{on() ? <A /> : null}`). */
export const childrenOf = (props: object): unknown => {
  const field = Object.getOwnPropertyDescriptor(props, "children");
  if (field) return "value" in field ? field.value : () => (props as Props).children;
  return "children" in props ? () => (props as Props).children : undefined;
};

/** A prop's value now: an accessor's (`cx={x}`) is what it gives. */
const read = (props: Props, key: string): unknown => {
  const value = props[key];
  return typeof value === "function" ? (value as () => unknown)() : value;
};

/** Follows one prop of `props`. */
export function bind(node: SceneNode, name: string, get: () => unknown, isStatic: boolean): void {
  let last = untracked(get);
  if (last !== undefined) node.set(name, last);
  if (isStatic) return;
  effect(() => {
    const now = get();
    if (Object.is(now, last)) return;
    last = now;
    untracked(() => node.set(name, now));
  });
}

/**
 * A node of a kind with a component's props: `defaults` for the ones left
 * out, `children` in place of the props' own.
 */
export function node(
  kind: string,
  props: Props = {},
  defaults: Props = {},
  children?: unknown,
  skip: readonly string[] = [],
): SceneNode {
  const made = new SceneNode(kind);
  for (const key of new Set([...Object.keys(defaults), ...Object.keys(props)])) {
    if (key === "children" || key === "key" || key === "ref" || skip.includes(key)) continue;
    const field = Object.getOwnPropertyDescriptor(props, key);
    const isStatic = field ? "value" in field && typeof field.value !== "function" : !(key in props);
    bind(made, key, () => read(props, key) ?? defaults[key], isStatic);
  }
  adopt(made, children === undefined ? childrenOf(props) : children);
  return made;
}

/** A node where JSX wants an element: only a <Canvas> shows it. */
export const element = (n: SceneNode): JSX.Element => n as unknown as JSX.Element;
