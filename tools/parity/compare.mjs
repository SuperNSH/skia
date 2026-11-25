// node compare.mjs <react-native-skia's packages/skia/src> <log of `nsh test` in examples/parity> <out dir>
//
// Draws each scene of examples/parity/scenes.ts with react-native-skia (its
// own TypeScript, bundled here, on canvaskit-wasm, in memory) and compares
// the pixels with the PNG nsh-skia printed. Writes a sheet of both and what
// differs, and results.json.

import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import * as esbuild from "esbuild";

const here = path.dirname(fileURLToPath(import.meta.url));
const [src, log, out] = process.argv.slice(2);
fs.mkdirSync(out, { recursive: true });
const entry = path.join(out, "entry.ts");
fs.writeFileSync(
  entry,
  `import React from "react"; export { React };
export { JsiSkApi } from ${JSON.stringify(path.join(src, "skia/web"))};
export * from ${JSON.stringify(path.join(src, "skia/types"))};
export { SkiaSGRoot } from ${JSON.stringify(path.join(src, "sksg/Reconciler"))};
export * from ${JSON.stringify(path.join(src, "renderer/components"))};
export { scenes, SIZE } from ${JSON.stringify(path.join(here, "../../examples/parity/scenes.ts"))};`,
);
const require = createRequire(import.meta.url);
await esbuild.build({
  entryPoints: [entry], bundle: true, outfile: path.join(out, "bundle.cjs"), format: "cjs", platform: "neutral",
  mainFields: ["module", "main"], nodePaths: [path.join(here, "node_modules")], target: "es2022",
  resolveExtensions: [".web.tsx", ".web.ts", ".tsx", ".ts", ".js"], jsx: "automatic", logLevel: "error",
  define: { "process.env.NODE_ENV": '"production"', __DEV__: "false", global: "globalThis" },
  external: ["react-native", "react-native-*"], tsconfigRaw: { compilerOptions: { jsx: "react-jsx" } },
});
const CK = await require("canvaskit-wasm/bin/full/canvaskit.js")({});
globalThis.CanvasKit = CK;
const L = require(path.resolve(out, "bundle.cjs"));
const Skia = L.JsiSkApi(CK);
const { SIZE } = L;
const isTree = (v) => Array.isArray(v) && typeof v[0] === "string" && typeof v[1] === "object";
const h = ([name, props, ...children]) => {
  const all = {};
  for (const key in props) all[key] = isTree(props[key]) ? h(props[key]) : props[key];
  return L.React.createElement(L[name], all, ...children.map(h));
};
const pixelsOf = (image) =>
  image.readPixels(0, 0, { width: SIZE, height: SIZE, colorType: CK.ColorType.RGBA_8888, alphaType: CK.AlphaType.Premul, colorSpace: CK.ColorSpace.SRGB });
const ours = new Map();
for (const line of fs.readFileSync(log, "utf8").split("\n")) {
  const m = /PNG (\w+) ([A-Za-z0-9+/=]+)/.exec(line);
  if (m) ours.set(m[1], Buffer.from(m[2], "base64"));
}
const results = {};
const names = Object.keys(L.scenes(Skia));
const cols = 4;
const sheet = CK.MakeSurface(cols * (SIZE * 3 + 20), Math.ceil(names.length / cols) * (SIZE + 24));
const canvas = sheet.getCanvas();
canvas.clear(CK.Color(120, 120, 120));
for (const [i, name] of names.entries()) {
  try {
    const surface = Skia.Surface.MakeOffscreen(SIZE, SIZE);
    const root = new L.SkiaSGRoot(Skia);
    await root.render(h(L.scenes(Skia)[name]));
    root.drawOnCanvas(surface.getCanvas());
    surface.flush();
    const image = CK.MakeImageFromEncoded(await surface.makeImageSnapshot().encodeToBytes());
    const mine = CK.MakeImageFromEncoded(ours.get(name));
    const [a, b] = [pixelsOf(image), pixelsOf(mine)];
    const diff = new Uint8Array(a.length);
    let max = 0, over = 0, sum = 0;
    for (let p = 0; p < a.length; p += 4) {
      let d = 0;
      for (let c = 0; c < 4; c++) d = Math.max(d, Math.abs(a[p + c] - b[p + c]));
      max = Math.max(max, d);
      sum += d;
      if (d > 16) over++;
      diff.set([Math.min(255, d * 4), 0, 0, 255], p);
    }
    results[name] = { max, mean: +(sum / (a.length / 4)).toFixed(3), over16: +((over * 400) / a.length).toFixed(2) };
    const x = (i % cols) * (SIZE * 3 + 20), y = Math.floor(i / cols) * (SIZE + 24);
    const info = { width: SIZE, height: SIZE, colorType: CK.ColorType.RGBA_8888, alphaType: CK.AlphaType.Opaque, colorSpace: CK.ColorSpace.SRGB };
    const white = new CK.Paint();
    white.setColor(CK.WHITE);
    canvas.drawRect(CK.XYWHRect(x, y, SIZE * 2, SIZE), white);
    canvas.drawImage(image, x, y);
    canvas.drawImage(mine, x + SIZE, y);
    canvas.drawImage(CK.MakeImage(info, diff, SIZE * 4), x + SIZE * 2, y);
  } catch (e) {
    results[name] = { error: String(e).slice(0, 200) };
  }
}
fs.writeFileSync(path.join(out, "sheet.png"), sheet.makeImageSnapshot().encodeToBytes());
fs.writeFileSync(path.join(out, "results.json"), `${JSON.stringify(results, null, 1)}\n`);
for (const [name, r] of Object.entries(results)) console.log(name.padEnd(20), r.error ?? `max ${r.max} mean ${r.mean} over16 ${r.over16}%`);
