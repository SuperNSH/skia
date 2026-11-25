// Draws the scenes of scenes.ts and prints each as a PNG: tools/parity
// compares them with react-native-skia's.

import { expect, test } from "nsh/test";
import * as Sk from "nsh-skia";
import { scenes, SIZE, type Tree } from "./scenes";

const isTree = (v: unknown): v is Tree => Array.isArray(v) && typeof v[0] === "string" && typeof v[1] === "object";
const build = ([name, props, ...children]: Tree): any => {
  const all: Record<string, unknown> = {};
  for (const key in props) all[key] = isTree(props[key]) ? build(props[key] as Tree) : props[key];
  if (children.length > 0) all.children = children.map((c) => build(c as Tree));
  return (Sk as any)[name](all);
};

const base64 = (bytes: Uint8Array) => {
  let text = "";
  for (let i = 0; i < bytes.length; i += 4096) text += String.fromCharCode(...bytes.subarray(i, i + 4096));
  return btoa(text);
};

test("every scene draws without errors", () => {
  const all = scenes(Sk.Skia);
  for (const name in all) {
    Sk.drawAsPicture(build(all[name]), { width: SIZE, height: SIZE });
    console.log(`PNG ${name} ${base64(Sk.drawAsImage(build(all[name]), { width: SIZE, height: SIZE }).encode())}`);
  }
  expect(Object.keys(all).length > 40).toBe(true);
});
