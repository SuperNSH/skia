// Scenes drawn by nsh-skia and by react-native-skia (tools/parity), as data:
// [component, props, ...children]. No text: the two engines' system fonts differ.

export type Tree = [string, Record<string, unknown>, ...unknown[]];

export const SIZE = 200;

// biome-ignore lint: the two libraries' own objects
export const scenes = (Skia: any): Record<string, Tree> => {
  const v = (x: number, y: number) => ({ x, y });
  const r = (x: number, y: number, width: number, height: number) => ({ x, y, width, height });
  const rr = (x: number, y: number, w: number, h: number, rx: number, ry = rx) => ({ rect: r(x, y, w, h), rx, ry });
  const fill = (color: string): Tree => ["Fill", { color }];
  const sksl = (s: string) => Skia.RuntimeEffect.Make(s);
  const checks = Skia.Image.MakeRaster(
    { width: 2, height: 2, colorType: 4, alphaType: 3 },
    Skia.Data.fromBytes(new Uint8Array([255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255])),
    8,
  );
  const heart = "M 100 170 C 20 110 40 40 100 75 C 160 40 180 110 100 170 Z";
  return {
    shapes: ["Group", {}, fill("white"), ["Rect", { x: 10, y: 10, width: 50, height: 30, color: "#ef4444" }],
      ["RoundedRect", { x: 70, y: 10, width: 50, height: 30, r: 8, color: "#22c55e" }], ["Oval", { x: 130, y: 10, width: 60, height: 30, color: "#3b82f6" }],
      ["Circle", { cx: 40, cy: 90, r: 28, color: "#a855f7" }], ["DiffRect", { outer: rr(80, 60, 60, 60, 20), inner: rr(95, 75, 30, 30, 8), color: "black" }],
      ["Line", { p1: v(150, 60), p2: v(190, 120), color: "#f97316", style: "stroke", strokeWidth: 6, strokeCap: "round" }],
      ["Points", { points: [v(20, 150), v(60, 190), v(100, 150), v(140, 190)], mode: "polygon", color: "#0f766e", style: "stroke", strokeWidth: 4, strokeJoin: "round" }]],
    strokes: ["Group", { style: "stroke", strokeWidth: 12, color: "black" }, fill("white"),
      ["Path", { path: "M 20 60 L 60 20 L 100 60", strokeJoin: "miter" }], ["Path", { path: "M 120 60 L 150 20 L 180 60", strokeJoin: "bevel", strokeCap: "square" }],
      ["Path", { path: heart, strokeWidth: 3, start: 0.2, end: 0.7, color: "red" }], ["Path", { path: heart, style: "fill", color: "#fde68a", stroke: { width: 4, join: 1 } }]],
    fillType: ["Path", { path: "M 100 10 L 160 190 L 10 70 L 190 70 L 40 190 Z", fillType: "evenOdd", color: "teal" }],
    transforms: ["Group", {}, fill("white"), ["Group", { transform: [{ translateX: 100 }, { translateY: 100 }, { rotate: 0.6 }, { scale: 1.2 }] }, ["Rect", { x: -40, y: -20, width: 80, height: 40, color: "#2563eb" }]],
      ["Group", { origin: v(50, 50), transform: [{ skewX: 0.4 }, { scaleY: 0.5 }] }, ["Rect", { x: 20, y: 20, width: 60, height: 60, color: "rgba(255,0,0,0.5)" }]],
      ["Group", { origin: v(150, 150), transform: [{ perspective: 300 }, { rotateY: 0.8 }, { rotateX: 0.3 }] }, ["Rect", { x: 110, y: 110, width: 80, height: 80, color: "#16a34a" }]]],
    opacity: ["Group", { opacity: 0.7 }, ["Rect", { x: 0, y: 0, width: 120, height: 120, color: "red" }], ["Group", { opacity: 0.5, color: "blue" }, ["Circle", { cx: 120, cy: 120, r: 60 }], ["Rect", { x: 150, y: 0, width: 50, height: 50, color: "rgba(0,0,0,0.5)" }]]],
    inherit: ["Group", { color: "#0ea5e9", style: "stroke", strokeWidth: 8 }, ["Circle", { cx: 60, cy: 60, r: 40 }], ["Group", { color: "#e11d48" }, ["Circle", { cx: 140, cy: 60, r: 40 }], ["Rect", { x: 40, y: 120, width: 120, height: 60, style: "fill" }]]],
    zIndex: ["Group", {}, ["Rect", { zIndex: 2, x: 10, y: 10, width: 80, height: 80, color: "red" }], ["Rect", { zIndex: 1, x: 50, y: 50, width: 80, height: 80, color: "green" }], ["Rect", { x: 90, y: 90, width: 80, height: 80, color: "blue" }], ["Rect", { zIndex: -1, x: 0, y: 70, width: 200, height: 40, color: "gold" }]],
    linear: ["Fill", {}, ["LinearGradient", { start: v(0, 0), end: v(200, 200), colors: ["cyan", "magenta", "yellow"], positions: [0, 0.3, 1] }]],
    radial: ["Fill", {}, ["RadialGradient", { c: v(100, 100), r: 60, colors: ["white", "black"], mode: "mirror" }]],
    sweep: ["Fill", {}, ["SweepGradient", { c: v(100, 100), colors: ["red", "lime", "blue", "red"], start: 30, end: 300 }]],
    conical: ["Fill", {}, ["TwoPointConicalGradient", { start: v(60, 100), startR: 20, end: v(150, 100), endR: 60, colors: ["orange", "navy"], mode: "repeat" }]],
    gradientTransform: ["Circle", { cx: 100, cy: 100, r: 90 }, ["LinearGradient", { start: v(0, 0), end: v(40, 0), colors: ["#111", "#eee"], mode: "repeat", origin: v(100, 100), transform: [{ rotate: 0.7 }] }]],
    noise: ["Fill", {}, ["FractalNoise", { freqX: 0.05, freqY: 0.05, octaves: 4 }]],
    turbulence: ["Fill", {}, ["Turbulence", { freqX: 0.03, freqY: 0.06, octaves: 2, seed: 3 }]],
    sksl: ["Fill", {}, ["Shader", { source: sksl("uniform float2 c; uniform float r; uniform float4 a; half4 main(float2 p) { float d = length(p - c) / r; return half4(a * (0.5 + 0.5 * sin(d * 20))); }"), uniforms: { c: v(100, 100), r: 100, a: [1, 0.5, 0, 1] } }]],
    skslChildren: ["Fill", {}, ["Shader", { source: sksl("uniform shader a; uniform shader b; half4 main(float2 p) { return mix(a.eval(p), b.eval(p), step(100, p.x)); }"), uniforms: {} },
      ["LinearGradient", { start: v(0, 0), end: v(0, 200), colors: ["red", "blue"] }], ["ColorShader", { color: "lime" }]]],
    blendShaders: ["Fill", {}, ["Blend", { mode: "difference" }, ["RadialGradient", { c: v(100, 100), r: 100, colors: ["white", "black"] }], ["LinearGradient", { start: v(0, 0), end: v(200, 0), colors: ["red", "blue"] }]]],
    blendModes: ["Group", {}, fill("#fef3c7"), ["Circle", { cx: 75, cy: 80, r: 50, color: "cyan" }], ["Group", { blendMode: "multiply" }, ["Circle", { cx: 125, cy: 80, r: 50, color: "magenta" }], ["Circle", { cx: 100, cy: 125, r: 50, color: "yellow" }]]],
    imageFits: ["Group", {}, fill("#ddd"), ["Image", { image: checks, x: 5, y: 5, width: 90, height: 40, fit: "contain", sampling: { filter: 0 } }], ["Image", { image: checks, x: 105, y: 5, width: 90, height: 40, fit: "cover", sampling: { filter: 0 } }],
      ["Image", { image: checks, x: 5, y: 55, width: 90, height: 40, fit: "fill" }], ["Image", { image: checks, x: 105, y: 55, width: 40, height: 90, fit: "fitWidth", sampling: { filter: 0 } }], ["Image", { image: checks, rect: r(150, 55, 40, 90), fit: "fitHeight", sampling: { B: 1 / 3, C: 1 / 3 } }], ["Image", { image: checks, x: 5, y: 105, width: 90, height: 90, fit: "none", sampling: { filter: 0 } }]],
    imageShader: ["Circle", { cx: 100, cy: 100, r: 90 }, ["ImageShader", { image: checks, tx: "repeat", ty: "mirror", fit: "fill", rect: r(0, 0, 50, 50), sampling: { filter: 0 } }]],
    blur: ["Group", {}, fill("white"), ["Circle", { cx: 100, cy: 100, r: 60, color: "#dc2626" }, ["Blur", { blur: 8 }]]],
    blurClamp: ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill", sampling: { filter: 0 } }, ["Blur", { blur: v(12, 2), mode: "clamp" }]],
    blurMask: ["Group", {}, fill("white"), ...["normal", "solid", "outer", "inner"].map((style, i): Tree => ["Circle", { cx: 50 + (i % 2) * 100, cy: 50 + (i >> 1) * 100, r: 28, color: "#1d4ed8" }, ["BlurMask", { blur: 6, style }]])],
    shadows: ["Group", {}, fill("white"), ["RoundedRect", { x: 20, y: 20, width: 70, height: 70, r: 12, color: "#93c5fd" }, ["Shadow", { dx: 8, dy: 8, blur: 5, color: "#1e3a8a" }]],
      ["RoundedRect", { x: 110, y: 20, width: 70, height: 70, r: 12, color: "#93c5fd" }, ["Shadow", { inner: true, dx: 8, dy: 8, blur: 5, color: "#1e3a8a" }]],
      ["RoundedRect", { x: 20, y: 110, width: 70, height: 70, r: 12, color: "#93c5fd" }, ["Shadow", { shadowOnly: true, dx: 4, dy: 4, blur: 5, color: "#1e3a8a" }]],
      ["RoundedRect", { x: 110, y: 110, width: 70, height: 70, r: 12, color: "#93c5fd" }, ["Shadow", { dx: 6, dy: 6, blur: 3, color: "red" }, ["Shadow", { dx: -6, dy: -6, blur: 3, color: "lime" }]]]],
    box: ["Group", {}, fill("#e5e7eb"), ["Box", { box: rr(40, 40, 120, 120, 20), color: "#f9fafb" }, ["BoxShadow", { dx: 10, dy: 10, blur: 10, color: "#9ca3af" }], ["BoxShadow", { dx: -10, dy: -10, blur: 10, color: "white" }], ["BoxShadow", { inner: true, dx: 5, dy: 5, blur: 5, spread: 2, color: "#d1d5db" }]]],
    morphology: ["Group", {}, fill("white"), ["Path", { path: heart, style: "stroke", strokeWidth: 6, color: "black" }, ["Morphology", { radius: 3 }]], ["Circle", { cx: 100, cy: 110, r: 20, color: "red" }, ["Morphology", { operator: "erode", radius: v(8, 2) }]]],
    displacement: ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill", sampling: { filter: 0 } }, ["DisplacementMap", { channelX: "r", channelY: "g", scale: 30 }, ["Turbulence", { freqX: 0.02, freqY: 0.02, octaves: 2 }]]],
    blendFilters: ["Circle", { cx: 100, cy: 100, r: 50, color: "#22d3ee" }, ["Blend", { mode: "color" }, ["Blur", { blur: 10 }], ["Offset", { x: 20, y: 20 }]]],
    offset: ["Rect", { x: 20, y: 20, width: 60, height: 60, color: "purple" }, ["Offset", { x: 60, y: 40 }, ["Blur", { blur: 4 }]]],
    colorMatrix: ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill" }, ["ColorMatrix", { matrix: [0.3, 0.6, 0.1, 0, 0, 0.3, 0.6, 0.1, 0, 0, 0.3, 0.6, 0.1, 0, 0, 0, 0, 0, 0.8, 0] }]],
    colorNested: ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill" }, ["SRGBToLinearGamma", {}, ["BlendColor", { color: "cyan", mode: "multiply" }]]],
    lerp: ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill" }, ["Lerp", { t: 0.4 }, ["BlendColor", { color: "red", mode: "srcIn" }], ["LinearToSRGBGamma", {}]]],
    pathEffects: ["Group", { style: "stroke", strokeWidth: 4, color: "black" }, fill("white"), ["Circle", { cx: 55, cy: 55, r: 40 }, ["DashPathEffect", { intervals: [10, 6], phase: 3 }]], ["Rect", { x: 110, y: 15, width: 75, height: 75 }, ["CornerPathEffect", { r: 24 }]],
      ["Circle", { cx: 55, cy: 145, r: 40 }, ["DiscretePathEffect", { length: 10, deviation: 4, seed: 2 }]], ["Path", { path: "M 110 150 C 130 110 160 190 190 150", strokeWidth: 8 }, ["Path1DPathEffect", { path: "M 0 -4 L 8 0 L 0 4 Z", advance: 12, phase: 0, style: "rotate" }]]],
    pathCompose: ["Rect", { x: 20, y: 20, width: 160, height: 160, style: "stroke", strokeWidth: 5, color: "navy" }, ["DashPathEffect", { intervals: [20, 10] }, ["CornerPathEffect", { r: 40 }]]],
    clips: ["Group", {}, fill("#f3f4f6"), ["Group", { clip: r(10, 10, 80, 80) }, fill("red")], ["Group", { clip: rr(110, 10, 80, 80, 30) }, fill("green")], ["Group", { clip: heart, transform: [{ translateX: 10 }, { translateY: 90 }, { scale: 0.5 }] }, fill("blue")], ["Group", { clip: rr(120, 120, 60, 60, 30), invertClip: true, opacity: 0.2 }, fill("black")]],
    layer: ["Group", { layer: ["Paint", { opacity: 0.6 }, ["Blur", { blur: 6 }]] }, ["Circle", { cx: 80, cy: 100, r: 50, color: "red" }], ["Circle", { cx: 120, cy: 100, r: 50, color: "blue" }]],
    paints: ["Circle", { cx: 100, cy: 100, r: 70, color: "#fbbf24" }, ["Paint", { color: "#92400e", style: "stroke", strokeWidth: 14 }], ["Paint", { color: "white", style: "stroke", strokeWidth: 4, opacity: 0.8 }]],
    mask: ["Group", {}, fill("#111827"), ["Mask", { mask: ["Circle", { cx: 100, cy: 100, r: 70 }, ["RadialGradient", { c: v(100, 100), r: 70, colors: ["black", "transparent"] }]] }, ["Fill", {}, ["LinearGradient", { start: v(0, 0), end: v(200, 0), colors: ["red", "blue"] }]]]],
    luminance: ["Mask", { mode: "luminance", mask: ["Group", {}, fill("black"), ["Circle", { cx: 100, cy: 100, r: 70, color: "#888" }], ["Circle", { cx: 100, cy: 100, r: 40, color: "white" }]] }, fill("teal")],
    maskNoClip: ["Mask", { clip: false, mask: ["Circle", { cx: 120, cy: 100, r: 50, color: "lime" }] }, ["Rect", { x: 20, y: 60, width: 100, height: 80, color: "red" }]],
    backdrop: ["Group", {}, ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill", sampling: { filter: 0 } }], ["BackdropBlur", { blur: 10, clip: rr(40, 40, 120, 120, 20) }, ["Fill", { color: "rgba(255,255,255,0.3)" }]]],
    backdropFilter: ["Group", {}, ["Image", { image: checks, x: 0, y: 0, width: 200, height: 200, fit: "fill", sampling: { filter: 0 } }], ["BackdropFilter", { clip: r(0, 100, 200, 100), filter: ["BlendColor", { color: "black", mode: "color" }] }]],
    vertices: ["Vertices", { vertices: [v(100, 10), v(190, 190), v(10, 190)], colors: ["red", "lime", "blue"] }],
    fan: ["Vertices", { mode: "triangleFan", vertices: [v(100, 100), v(180, 20), v(190, 120), v(100, 190), v(20, 120)], indices: [0, 1, 2, 3, 4], color: "orange" }],
    patch: ["Patch", { colors: ["red", "yellow", "lime", "blue"], patch: [{ pos: v(20, 20), c1: v(20, 80), c2: v(80, 0) }, { pos: v(180, 20), c1: v(120, 60), c2: v(200, 80) }, { pos: v(180, 180), c1: v(160, 120), c2: v(120, 200) }, { pos: v(20, 180), c1: v(80, 140), c2: v(0, 120) }] }],
    atlas: ["Atlas", { image: checks, sprites: [r(0, 0, 1, 1), r(1, 0, 1, 1), r(0, 1, 2, 1)], transforms: [0, 1, 2].map((i) => Skia.RSXform(30 * Math.cos(i), 30 * Math.sin(i), 40 + i * 50, 60 + i * 30)) }],
    fitbox: ["FitBox", { src: r(0, 0, 10, 5), dst: r(0, 0, 200, 200), fit: "contain" }, ["Rect", { x: 0, y: 0, width: 10, height: 5, color: "skyblue" }], ["Circle", { cx: 5, cy: 2.5, r: 2, color: "navy" }]],
    runtimeFilter: ["Circle", { cx: 100, cy: 100, r: 60, color: "red" }, ["RuntimeShader", { source: sksl("uniform shader image; half4 main(float2 p) { p.x += sin(p.y / 8) * 6; return image.eval(p).bgra; }") }]],
  };
};
