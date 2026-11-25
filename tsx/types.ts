// Values and enums, as react-native-skia has them (MIT, see NOTICE.md).

export interface SkPoint {
  readonly x: number;
  readonly y: number;
}
export type Vector = SkPoint;

export interface SkRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface SkRRect {
  readonly rect: SkRect;
  readonly rx: number;
  readonly ry: number;
}

export interface NonUniformRRect {
  readonly rect: SkRect;
  readonly topLeft: SkPoint;
  readonly topRight: SkPoint;
  readonly bottomRight: SkPoint;
  readonly bottomLeft: SkPoint;
}

export type InputRRect = SkRRect | NonUniformRRect;

export interface SkRSXform {
  readonly scos: number;
  readonly ssin: number;
  readonly tx: number;
  readonly ty: number;
}

/** Red, green, blue and alpha, from 0 to 1. */
export type SkColor = Float32Array;
/** A CSS color, `0xAARRGGBB`, or red, green, blue and alpha from 0 to 1. */
export type Color = string | number | readonly number[] | Float32Array;

export enum TileMode { Clamp, Repeat, Mirror, Decal }
export enum BlurStyle { Normal, Solid, Outer, Inner }
export enum ClipOp { Difference, Intersect }
export enum ColorChannel { R, G, B, A }
export enum FilterMode { Nearest, Linear }
export enum MipmapMode { None, Nearest, Linear }
export enum FillType { Winding, EvenOdd, InverseWinding, InverseEvenOdd }
export enum PathOp { Difference, Intersect, Union, XOR, ReverseDifference }
export enum PathVerb { Move, Line, Quad, Conic, Cubic, Close }
export enum PointMode { Points, Lines, Polygon }
export enum VertexMode { Triangles, TrianglesStrip, TriangleFan }
export enum PaintStyle { Fill, Stroke }
export enum StrokeCap { Butt, Round, Square }
export enum StrokeJoin { Miter, Round, Bevel }
export enum Path1DEffectStyle { Translate, Rotate, Morph }
export enum ImageFormat { JPEG = 3, PNG = 4, WEBP = 6 }
export enum AlphaType { Unknown, Opaque, Premul, Unpremul }
export enum ColorType { RGBA_8888 = 4, BGRA_8888 = 6 }
export enum FontWeight {
  Invisible = 0, Thin = 100, ExtraLight = 200, Light = 300, Normal = 400, Medium = 500, SemiBold = 600, Bold = 700,
  ExtraBold = 800, Black = 900, ExtraBlack = 1000,
}
export enum FontWidth {
  UltraCondensed = 1, ExtraCondensed, Condensed, SemiCondensed, Normal, SemiExpanded, Expanded, ExtraExpanded,
  UltraExpanded,
}
export enum FontSlant { Upright, Italic, Oblique }
export enum FontEdging { Alias, AntiAlias, SubpixelAntiAlias }
export enum FontHinting { None, Slight, Normal, Full }
export enum TextDirection { RTL, LTR }
export enum TextAlign { Left, Right, Center, Justify, Start, End }
export enum TextBaseline { Alphabetic, Ideographic }
export enum TextDecoration { NoDecoration = 0, Underline = 1, Overline = 2, LineThrough = 4 }
export enum TextDecorationStyle { Solid, Double, Dotted, Dashed, Wavy }
export enum TextHeightBehavior { All, DisableFirstAscent, DisableLastDescent, DisableAll }
export enum PlaceholderAlignment { Baseline, AboveBaseline, BelowBaseline, Top, Bottom, Middle }
export enum BlendMode {
  Clear, Src, Dst, SrcOver, DstOver, SrcIn, DstIn, SrcOut, DstOut, SrcATop, DstATop, Xor, Plus, Modulate, Screen,
  Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion, Multiply, Hue,
  Saturation, Color, Luminosity,
}

/** An enum's names, as props take them (`"srcOver"`). */
export type SkEnum<T> = Uncapitalize<keyof T extends string ? keyof T : never>;

export interface CubicResampler {
  B: number;
  C: number;
}
export interface FilterOptions {
  filter: FilterMode;
  mipmap?: MipmapMode;
}
export type SamplingOptions = CubicResampler | FilterOptions;
export const MitchellCubicSampling = { B: 1 / 3, C: 1 / 3 };
export const CatmullRomCubicSampling = { B: 0, C: 1 / 2 };
export const CubicSampling = { B: 0, C: 0 };
export const MakeCubic = (B: number, C: number): CubicResampler => ({ B, C });

export interface ImageInfo {
  alphaType: AlphaType;
  colorType: ColorType;
  height: number;
  width: number;
}

export interface StrokeOpts {
  width?: number;
  miter_limit?: number;
  precision?: number;
  join?: StrokeJoin;
  cap?: StrokeCap;
}

export interface FontStyle {
  weight?: FontWeight;
  width?: FontWidth;
  slant?: FontSlant;
}
export const FontStyle = {
  Normal: { weight: FontWeight.Normal, width: FontWidth.Normal, slant: FontSlant.Upright },
  Bold: { weight: FontWeight.Bold, width: FontWidth.Normal, slant: FontSlant.Upright },
  Italic: { weight: FontWeight.Normal, width: FontWidth.Normal, slant: FontSlant.Italic },
  BoldItalic: { weight: FontWeight.Bold, width: FontWidth.Normal, slant: FontSlant.Italic },
};

export interface SkTextShadow {
  color?: Color;
  offset?: SkPoint;
  blurRadius?: number;
}
export interface SkTextStyle {
  backgroundColor?: Color;
  color?: Color;
  decoration?: TextDecoration;
  decorationColor?: Color;
  decorationThickness?: number;
  decorationStyle?: TextDecorationStyle;
  fontFamilies?: string[];
  fontFeatures?: { name: string; value: number }[];
  fontSize?: number;
  fontStyle?: FontStyle;
  foregroundColor?: Color;
  heightMultiplier?: number;
  halfLeading?: boolean;
  letterSpacing?: number;
  locale?: string;
  shadows?: SkTextShadow[];
  textBaseline?: TextBaseline;
  wordSpacing?: number;
}
export interface SkStrutStyle {
  strutEnabled?: boolean;
  fontFamilies?: string[];
  fontStyle?: FontStyle;
  fontSize?: number;
  heightMultiplier?: number;
  halfLeading?: boolean;
  leading?: number;
  forceStrutHeight?: boolean;
}
export interface SkParagraphStyle {
  disableHinting?: boolean;
  ellipsis?: string;
  heightMultiplier?: number;
  maxLines?: number;
  replaceTabCharacters?: boolean;
  strutStyle?: SkStrutStyle;
  textAlign?: TextAlign;
  textDirection?: TextDirection;
  textHeightBehavior?: TextHeightBehavior;
  textStyle?: SkTextStyle;
}

type Transform3d =
  | { translateX: number }
  | { translateY: number }
  | { translateZ: number }
  | { translate: readonly [number, number] | readonly [number, number, number] }
  | { scale: number }
  | { scaleX: number }
  | { scaleY: number }
  | { skewX: number }
  | { skewY: number }
  | { perspective: number }
  | { rotate: number }
  | { rotateX: number }
  | { rotateY: number }
  | { rotateZ: number }
  | { matrix: readonly number[] };
export type Transforms3d = Transform3d[];

/** A uniform: a number, a point, or numbers in arrays. */
export type Uniform = number | Vector | Float32Array | readonly Uniform[];
export interface Uniforms {
  [name: string]: Uniform;
}

export type Fit = "cover" | "contain" | "fill" | "fitHeight" | "fitWidth" | "none" | "scaleDown";
export type Radius = number | Vector;

export const PI = Math.PI;
export const TAU = Math.PI * 2;
export const vec = (x = 0, y?: number): SkPoint => ({ x, y: y ?? x });
export const point = vec;
export const rect = (x: number, y: number, width: number, height: number): SkRect => ({ x, y, width, height });
export const rrect = (r: SkRect, rx: number, ry: number): SkRRect => ({ rect: r, rx, ry });
export const add = (a: SkPoint, b: SkPoint) => vec(a.x + b.x, a.y + b.y);
export const sub = (a: SkPoint, b: SkPoint) => vec(a.x - b.x, a.y - b.y);
export const dist = (a: SkPoint, b: SkPoint) => Math.hypot(a.x - b.x, a.y - b.y);
export const center = (r: SkRect) => vec(r.x + r.width / 2, r.y + r.height / 2);
export const topLeft = (r: SkRect) => vec(r.x, r.y);
export const topRight = (r: SkRect) => vec(r.x + r.width, r.y);
export const bottomLeft = (r: SkRect) => vec(r.x, r.y + r.height);
export const bottomRight = (r: SkRect) => vec(r.x + r.width, r.y + r.height);
export const bounds = (rects: SkRect[]): SkRect => {
  const x = Math.min(...rects.map((r) => r.x));
  const y = Math.min(...rects.map((r) => r.y));
  const right = Math.max(...rects.map((r) => r.x + r.width));
  const bottom = Math.max(...rects.map((r) => r.y + r.height));
  return rect(x, y, right - x, bottom - y);
};
export const mix = (value: number, x: number, y: number) => x * (1 - value) + y * value;
export const mixVector = (value: number, from: Vector, to: Vector) =>
  vec(mix(value, from.x, to.x), mix(value, from.y, to.y));
export const clamp = (value: number, lo: number, hi: number) => Math.min(Math.max(lo, value), hi);
export const polar2Canvas = (p: { theta: number; radius: number }, origin: Vector) =>
  vec(origin.x + p.radius * Math.cos(p.theta), origin.y + p.radius * Math.sin(p.theta));
export const canvas2Polar = (v: Vector, origin: Vector) => {
  const x = v.x - origin.x;
  const y = v.y - origin.y;
  return { theta: Math.atan2(y, x), radius: Math.sqrt(x ** 2 + y ** 2) };
};
