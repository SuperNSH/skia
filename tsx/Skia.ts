// The imperative API: react-native-skia's `Skia` object on this crate's
// classes. Same names; docs/parity.md lists what is left out.

import {
  SkColorFilter,
  SkContourMeasureIter,
  SkData,
  SkFont,
  SkFontMgr,
  SkImage,
  SkImageFilter,
  SkMaskFilter,
  SkMatrix,
  SkPaint,
  SkParagraphBuilder,
  SkPath,
  SkPathBuilder,
  SkPathEffect,
  SkPicture,
  SkPictureRecorder,
  SkRuntimeEffect,
  SkRuntimeShaderBuilder,
  SkShader,
  SkSurface,
  SkSvg,
  SkTextBlob,
  SkTypeface,
  SkTypefaceFontProvider,
  SkVertices,
} from "nsh-skia/native";
import type {
  Color,
  FontStyle,
  ImageInfo,
  InputRRect,
  PathOp,
  SkColor,
  SkParagraphStyle,
  SkPoint,
  SkRect,
  SkRRect,
  SkRSXform,
  StrokeOpts,
  VertexMode,
} from "./types";
import { ColorType } from "./types";

const colorPaint = () => (paint ??= new SkPaint());
let paint: SkPaint | undefined;

const bytesOf = (base64: string): Uint8Array => {
  const text = atob(base64);
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) out[i] = text.charCodeAt(i);
  return out;
};

export const Skia = {
  Point: (x: number, y: number): SkPoint => ({ x, y }),
  XYWHRect: (x: number, y: number, width: number, height: number): SkRect => ({ x, y, width, height }),
  RRectXY: (rect: SkRect, rx: number, ry: number): SkRRect => ({ rect, rx, ry }),
  RSXform: (scos: number, ssin: number, tx: number, ty: number): SkRSXform => ({ scos, ssin, tx, ty }),
  RSXformFromRadians: (scale: number, radians: number, tx: number, ty: number, px: number, py: number): SkRSXform => {
    const ssin = Math.sin(radians) * scale;
    const scos = Math.cos(radians) * scale;
    return { scos, ssin, tx: tx - scos * px + ssin * py, ty: ty - ssin * px - scos * py };
  },
  /** A color as red, green, blue and alpha. */
  Color: (color: Color): SkColor => {
    const p = colorPaint();
    p.setColor(color);
    return new Float32Array(p.getColor());
  },
  Paint: () => new SkPaint(),
  Matrix: (matrix?: readonly number[]) => new SkMatrix(matrix),
  PictureRecorder: () => new SkPictureRecorder(),
  ContourMeasureIter: (path: SkPath, forceClosed: boolean, resScale: number) =>
    new SkContourMeasureIter(path, forceClosed, resScale),
  RuntimeShaderBuilder: (effect: SkRuntimeEffect) => new SkRuntimeShaderBuilder(effect),
  MakeVertices: (
    mode: VertexMode,
    positions: readonly SkPoint[] | Float32Array,
    textures?: readonly SkPoint[] | Float32Array | null,
    colors?: readonly Color[] | null,
    indices?: readonly number[] | null,
  ) => SkVertices.make(mode, positions, textures, colors, indices),
  Font: (typeface?: SkTypeface | null, size?: number) => new SkFont(typeface, size),
  Path: {
    Make: () => SkPath.make(),
    MakeFromSVGString: (svg: string) => SkPath.fromSvg(svg),
    MakeFromOp: (one: SkPath, two: SkPath, op: PathOp) => SkPath.fromOp(one, two, op),
    MakeFromCmds: (cmds: readonly (readonly number[])[]) => SkPath.fromCmds(cmds.flat()),
    MakeFromText: (text: string, x: number, y: number, font: SkFont) => SkPath.fromText(text, x, y, font),
    Rect: (rect: SkRect, isCCW?: boolean) => SkPath.rect(rect, isCCW),
    Oval: (rect: SkRect, isCCW?: boolean, startIndex?: number) => SkPath.oval(rect, isCCW, startIndex),
    Circle: (x: number, y: number, r: number) => SkPath.circle(x, y, r),
    RRect: (rrect: InputRRect, isCCW?: boolean) => SkPath.rRect(rrect, isCCW),
    Line: (p1: SkPoint, p2: SkPoint) => SkPath.line(p1, p2),
    Polygon: (points: readonly SkPoint[], close: boolean) => SkPath.polygon(points, close),
    Stroke: (path: SkPath, opts: StrokeOpts = {}) =>
      SkPath.stroke(path, opts.width, opts.miter_limit, opts.precision, opts.join, opts.cap),
    Trim: (path: SkPath, start: number, end: number, isComplement: boolean) =>
      SkPath.trim(path, start, end, isComplement),
    Simplify: (path: SkPath) => SkPath.simplify(path),
    Dash: (path: SkPath, on: number, off: number, phase: number) => SkPath.dash(path, on, off, phase),
    AsWinding: (path: SkPath) => SkPath.asWinding(path),
    Interpolate: (start: SkPath, end: SkPath, weight: number) => SkPath.interpolateBetween(start, end, weight),
  },
  PathBuilder: {
    Make: () => new SkPathBuilder(),
    MakeFromPath: (path: SkPath) => new SkPathBuilder(path),
  },
  PathEffect: {
    MakeCorner: SkPathEffect.makeCorner,
    MakeDash: SkPathEffect.makeDash,
    MakeDiscrete: SkPathEffect.makeDiscrete,
    MakeCompose: SkPathEffect.makeCompose,
    MakeSum: SkPathEffect.makeSum,
    MakeLine2D: SkPathEffect.makeLine2d,
    MakePath1D: SkPathEffect.makePath1d,
    MakePath2D: SkPathEffect.makePath2d,
  },
  Shader: {
    MakeColor: SkShader.makeColor,
    MakeLinearGradient: SkShader.makeLinearGradient,
    MakeRadialGradient: SkShader.makeRadialGradient,
    MakeSweepGradient: SkShader.makeSweepGradient,
    MakeTwoPointConicalGradient: SkShader.makeTwoPointConicalGradient,
    MakeTurbulence: SkShader.makeTurbulence,
    MakeFractalNoise: SkShader.makeFractalNoise,
    MakeBlend: SkShader.makeBlend,
  },
  ColorFilter: {
    MakeMatrix: SkColorFilter.makeMatrix,
    MakeBlend: SkColorFilter.makeBlend,
    MakeCompose: SkColorFilter.makeCompose,
    MakeLerp: SkColorFilter.makeLerp,
    MakeLinearToSRGBGamma: SkColorFilter.makeLinearToSrgbGamma,
    MakeSRGBToLinearGamma: SkColorFilter.makeSrgbToLinearGamma,
    MakeLuma: SkColorFilter.makeLuma,
  },
  MaskFilter: { MakeBlur: SkMaskFilter.makeBlur },
  ImageFilter: {
    MakeOffset: SkImageFilter.makeOffset,
    MakeDisplacementMap: SkImageFilter.makeDisplacementMap,
    MakeShader: SkImageFilter.makeShader,
    MakeBlur: SkImageFilter.makeBlur,
    MakeColorFilter: SkImageFilter.makeColorFilter,
    MakeCompose: SkImageFilter.makeCompose,
    MakeDropShadow: SkImageFilter.makeDropShadow,
    MakeDropShadowOnly: SkImageFilter.makeDropShadowOnly,
    MakeErode: SkImageFilter.makeErode,
    MakeDilate: SkImageFilter.makeDilate,
    MakeBlend: SkImageFilter.makeBlend,
    MakeEmpty: SkImageFilter.makeEmpty,
    MakeCrop: SkImageFilter.makeCrop,
    MakeArithmetic: SkImageFilter.makeArithmetic,
    MakeRuntimeShader: SkImageFilter.makeRuntimeShader,
  },
  /** `Make` gives `null` for SkSL that does not compile, and logs the compiler's error. */
  RuntimeEffect: {
    Make: (sksl: string): SkRuntimeEffect | null => {
      try {
        return SkRuntimeEffect.make(sksl);
      } catch (e) {
        console.error(`SkSL: ${e instanceof Error ? e.message : String(e)}`);
        return null;
      }
    },
  },
  Data: {
    fromBytes: (bytes: Uint8Array) => SkData.fromBytes(bytes),
    fromBase64: (base64: string) => SkData.fromBytes(bytesOf(base64)),
    fromURI: async (uri: string) => {
      const response = await fetch(uri);
      if (!response.ok) throw new Error(`${uri}: ${response.status}`);
      return SkData.fromBytes(new Uint8Array(await response.arrayBuffer()));
    },
  },
  Image: {
    MakeFromEncoded: (data: SkData) => SkImage.fromEncoded(data),
    MakeRaster: (info: ImageInfo, data: SkData, bytesPerRow: number) =>
      SkImage.fromPixels(
        info.width,
        info.height,
        info.colorType === ColorType.BGRA_8888,
        info.alphaType,
        data,
        bytesPerRow,
      ),
  },
  SVG: {
    MakeFromString: (svg: string) => SkSvg.fromText(svg),
  },
  Typeface: { MakeFromData: (data: SkData) => SkTypeface.fromData(data) },
  FontMgr: { System: () => SkFontMgr.system() },
  TypefaceFontProvider: { Make: () => SkTypefaceFontProvider.make() },
  ParagraphBuilder: {
    Make: (style: SkParagraphStyle = {}, provider?: SkTypefaceFontProvider) => new SkParagraphBuilder(style, provider),
  },
  TextBlob: {
    MakeFromText: (text: string, font: SkFont) => SkTextBlob.fromText(text, font),
    MakeFromGlyphs: (glyphs: readonly number[], font: SkFont) => SkTextBlob.fromGlyphs(glyphs, font),
    MakeFromRSXform: (text: string, transforms: readonly SkRSXform[], font: SkFont) =>
      SkTextBlob.fromRsXform(text, transforms, font),
    MakeFromRSXformGlyphs: (glyphs: readonly number[], transforms: readonly SkRSXform[], font: SkFont) =>
      SkTextBlob.fromRsXformGlyphs(glyphs, transforms, font),
  },
  Picture: { Make: (bytes: Uint8Array | ArrayBuffer) => SkPicture.fromBytes(bytes) },
  Surface: {
    /** A surface in memory. */
    MakeRaster: (width: number, height: number) => SkSurface.make(width, height),
    /** A surface in memory (react-native-skia's is on the GPU). */
    MakeOffscreen: (width: number, height: number) => SkSurface.make(width, height),
  },
};

/** The font of a family of the system's, or its default. */
export const matchFont = (
  style: { fontFamily?: string; fontSize?: number; fontStyle?: "normal" | "italic" | "oblique"; fontWeight?: string } = {},
): SkFont => {
  const slant = { normal: 0, italic: 1, oblique: 2 }[style.fontStyle ?? "normal"];
  const weight = { normal: 400, bold: 700 }[style.fontWeight ?? "normal"] ?? (Number(style.fontWeight) || 400);
  const face = style.fontFamily
    ? SkFontMgr.system().matchFamilyStyleNumbers(style.fontFamily, weight, 5, slant)
    : SkTypeface.system(weight, 5, slant);
  return new SkFont(face, style.fontSize ?? 14);
};

/** The typeface of a family of the system's. */
export const matchTypeface = (family: string, style: FontStyle = {}): SkTypeface | null =>
  SkFontMgr.system().matchFamilyStyleNumbers(family, style.weight, style.width, style.slant);

/** Records what `draw` draws. */
export const createPicture = (draw: (canvas: ReturnType<SkPictureRecorder["beginRecording"]>) => void, bounds?: SkRect) => {
  const recorder = new SkPictureRecorder();
  draw(recorder.beginRecording(bounds));
  return recorder.finishRecordingAsPicture();
};

/** The program of a template literal of SkSL; throws what the compiler says. */
export const Sksl = (strings: TemplateStringsArray, ...values: (string | number)[]): SkRuntimeEffect =>
  SkRuntimeEffect.make(strings.reduce((out, s, i) => out + s + (i < values.length ? String(values[i]) : ""), ""));
