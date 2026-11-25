// The components a <Canvas> holds, react-native-skia's (props and defaults:
// its src/renderer/components and src/dom/types; MIT, see NOTICE.md).

import { computed, type JSX, untracked } from "nsh";
import type {
  SkFont,
  SkImage,
  SkImageFilter,
  SkMatrix,
  SkPaint,
  SkParagraph,
  SkPath,
  SkPicture,
  SkRuntimeEffect,
  SkSvg,
  SkTextBlob,
} from "nsh-skia/native";
import { childrenOf, element, node, SceneNode } from "./scene";
import type {
  BlendMode,
  BlurStyle,
  Color,
  ColorChannel,
  FillType,
  Fit,
  InputRRect,
  PaintStyle,
  Path1DEffectStyle,
  PointMode,
  Radius,
  SamplingOptions,
  SkEnum,
  SkPoint,
  SkRect,
  SkRRect,
  SkRSXform,
  StrokeCap,
  StrokeJoin,
  StrokeOpts,
  TileMode,
  Transforms3d,
  Uniforms,
  Vector,
  VertexMode,
} from "./types";

type Children = JSX.Element | JSX.Element[];
export interface ChildrenProps {
  children?: Children;
}

/** A component's props: each a value, or an accessor of one (a signal). */
export type SkiaProps<P> = { [K in keyof P]: K extends "children" ? P[K] : P[K] | (() => P[K]) };

export type PathDef = string | SkPath;
export type ClipDef = SkRRect | SkRect | InputRRect | PathDef;
type InputMatrix = SkMatrix | readonly number[];

export interface TransformProps {
  transform?: Transforms3d;
  origin?: Vector;
  matrix?: InputMatrix;
}
export interface PaintProps extends ChildrenProps {
  color?: Color;
  strokeWidth?: number;
  blendMode?: SkEnum<typeof BlendMode>;
  style?: SkEnum<typeof PaintStyle>;
  strokeJoin?: SkEnum<typeof StrokeJoin>;
  strokeCap?: SkEnum<typeof StrokeCap>;
  strokeMiter?: number;
  opacity?: number;
  antiAlias?: boolean;
  dither?: boolean;
}
export interface GroupProps extends PaintProps, TransformProps {
  clip?: ClipDef;
  invertClip?: boolean;
  layer?: SkPaint | boolean;
  zIndex?: number;
}
export interface DrawingNodeProps extends GroupProps {
  paint?: SkPaint;
}

interface RectCtor {
  x?: number;
  y?: number;
  width?: number;
  height?: number;
  rect?: SkRect;
}
interface RRectCtor extends Omit<RectCtor, "rect"> {
  r?: Radius;
  rect?: InputRRect;
}

const make =
  <P,>(kind: string, defaults: Record<string, unknown> = {}) =>
  (props: SkiaProps<P>): JSX.Element =>
    element(node(kind, props as Record<string, unknown>, defaults));

// --- Groups and paint ---------------------------------------------------------

export interface PublicGroupProps extends Omit<GroupProps, "layer"> {
  layer?: SkPaint | boolean | JSX.Element;
}

/** Children, under one transform, clip, layer and paint. `layer` may be a `<Paint>`. */
export const Group = (props: SkiaProps<PublicGroupProps>): JSX.Element => {
  const layer = untracked(() => props.layer) as unknown;
  if (!(layer instanceof SceneNode)) return element(node("Group", props as Record<string, unknown>));
  return element(node("Layer", {}, {}, [layer, node("Group", props as Record<string, unknown>, {}, undefined, ["layer"])]));
};

/** A paint: of the node it is in, drawn with after the node's own. */
export const Paint = make<DrawingNodeProps>("Paint");

/** Children, where `mask` is. */
export const Mask = (props: {
  mode?: "luminance" | "alpha";
  clip?: boolean;
  mask: Children;
  children?: Children;
}): JSX.Element => {
  const luma = untracked(() => props.mode) === "luminance" ? [node("LumaColorFilter")] : [];
  const paint = node("Paint", { blendMode: untracked(() => props.clip) === false ? "dstATop" : "dstIn" }, {}, luma);
  const mask = node("Layer", {}, {}, [paint, node("Group", {}, {}, () => props.mask)]);
  return element(node("Group", { layer: true }, {}, [childrenOf(props), mask]));
};

export interface BackdropFilterProps extends Omit<GroupProps, "children"> {
  filter: JSX.Element;
  children?: Children;
}

/** What is behind, filtered; children over it. */
export const BackdropFilter = (props: SkiaProps<BackdropFilterProps>): JSX.Element => {
  const filter = node("BackdropFilter", {}, {}, () => props.filter);
  const children = [filter, childrenOf(props)];
  return element(node("Group", props as unknown as Record<string, unknown>, {}, children, ["filter"]));
};

/** What is behind, blurred; children over it. */
export const BackdropBlur = (props: SkiaProps<Omit<BackdropFilterProps, "filter"> & { blur: Radius }>): JSX.Element => {
  const all = props as unknown as Record<string, unknown>;
  const others = Object.keys(all).filter((k) => k !== "blur");
  const blur = node("BlurImageFilter", all, { mode: "clamp" }, [], others);
  const children = [node("BackdropFilter", {}, {}, [blur]), childrenOf(props)];
  return element(node("Group", all, {}, children, ["blur"]));
};

/** The transform that puts `src` into `dst`. */
export const fitbox = (fit: Fit, src: SkRect, dst: SkRect): Transforms3d => {
  const rects = fitRects(fit, src, dst);
  const scaleX = rects.dst.width / rects.src.width;
  const scaleY = rects.dst.height / rects.src.height;
  return [
    { translateX: rects.dst.x - rects.src.x * scaleX },
    { translateY: rects.dst.y - rects.src.y * scaleY },
    { scaleX },
    { scaleY },
  ];
};

/** The part of `src` and the place in `dst` a fit takes. */
export const fitRects = (fit: Fit, src: SkRect, dst: SkRect): { src: SkRect; dst: SkRect } => {
  const size = (width: number, height: number) => ({ width, height });
  const inscribe = (s: { width: number; height: number }, r: SkRect): SkRect => ({
    x: r.x + (r.width - s.width) / 2,
    y: r.y + (r.height - s.height) / 2,
    ...s,
  });
  let from = size(src.width, src.height);
  let to = size(dst.width, dst.height);
  const wider = to.width / to.height > from.width / from.height;
  if (from.width <= 0 || from.height <= 0 || to.width <= 0 || to.height <= 0) {
    from = size(0, 0);
    to = size(0, 0);
  } else if (fit === "contain") {
    to = wider
      ? size((from.width * to.height) / from.height, to.height)
      : size(to.width, (from.height * to.width) / from.width);
  } else if (fit === "cover") {
    from = wider
      ? size(from.width, (from.width * to.height) / to.width)
      : size((from.height * to.width) / to.height, from.height);
  } else if (fit === "fitWidth") {
    from = size(from.width, (from.width * to.height) / to.width);
    to = size(to.width, (from.height * to.width) / from.width);
  } else if (fit === "fitHeight") {
    from = size((from.height * to.width) / to.height, from.height);
    to = size((from.width * to.height) / from.height, to.height);
  } else if (fit === "none") {
    from = size(Math.min(from.width, to.width), Math.min(from.height, to.height));
    to = from;
  } else if (fit === "scaleDown") {
    const ratio = from.width / from.height;
    let fitted = from;
    if (fitted.height > to.height) fitted = size(to.height * ratio, to.height);
    if (fitted.width > to.width) fitted = size(to.width, to.width / ratio);
    to = fitted;
  }
  return { src: inscribe(from, src), dst: inscribe(to, dst) };
};

/** Children drawn in `src`, put into `dst`. */
export const FitBox = (props: { fit?: Fit; src: SkRect; dst: SkRect; children?: Children }): JSX.Element =>
  element(
    node(
      "Group",
      { transform: () => fitbox(props.fit ?? "contain", props.src, props.dst) },
      {},
      childrenOf(props),
    ),
  );

const ROWS = 32;

/**
 * `<Each>` for a scene: `children(item, index)` for each item of `of`, kept
 * by `key`. NSH renders an `<Each>`'s rows past the 32nd when they come on
 * screen, which a scene's never do; this shows them all.
 */
export function Repeat<T>(props: {
  of: readonly T[] | null | undefined;
  key?: "index" | ((item: T, index: number) => unknown);
  children: (item: () => T, index: () => number) => JSX.Element;
}): JSX.Element {
  const count = computed(() => Math.ceil((props.of?.length ?? 0) / ROWS));
  const chunks = computed(() => Array.from({ length: count() }, (_, i) => i));
  const deep = computed(() => count() > ROWS);
  const chunk = (at: () => number) => (
    <Each of={props.of?.slice(at() * ROWS, (at() + 1) * ROWS)} key={props.key as any}>
      {(item: () => T, index: () => number) => props.children(item, () => at() * ROWS + index())}
    </Each>
  );
  return (() =>
    deep() ? <Repeat of={chunks()} key="index">{chunk}</Repeat> : <Each of={chunks()} key="index">{chunk}</Each>) as any;
}

// --- Shapes -------------------------------------------------------------------

export const Fill = make<DrawingNodeProps>("Fill");
export const Circle = make<DrawingNodeProps & { c?: Vector; cx?: number; cy?: number; r: number }>("Circle");
export const Rect = make<DrawingNodeProps & RectCtor>("Rect");
export const RoundedRect = make<DrawingNodeProps & RRectCtor>("RRect");
export const DiffRect = make<DrawingNodeProps & { inner: InputRRect; outer: InputRRect }>("DiffRect");
export const Oval = make<DrawingNodeProps & RectCtor>("Oval");
export const Line = make<DrawingNodeProps & { p1: Vector; p2: Vector }>("Line");
export const Points = make<
  DrawingNodeProps & { points: readonly SkPoint[] | Float32Array; mode?: SkEnum<typeof PointMode> }
>("Points", { mode: "points" });
export const Path = make<
  DrawingNodeProps & {
    path: PathDef;
    start?: number;
    end?: number;
    stroke?: StrokeOpts;
    fillType?: SkEnum<typeof FillType>;
  }
>("Path", { start: 0, end: 1 });
export const Vertices = make<
  DrawingNodeProps & {
    vertices: readonly SkPoint[] | Float32Array;
    textures?: readonly SkPoint[] | Float32Array;
    colors?: readonly Color[];
    mode?: SkEnum<typeof VertexMode>;
    indices?: readonly number[];
  }
>("Vertices", { mode: "triangles" });
export interface CubicBezierHandle {
  pos: Vector;
  c1: Vector;
  c2: Vector;
}
export const Patch = make<
  DrawingNodeProps & { patch: readonly CubicBezierHandle[]; colors?: readonly Color[]; texture?: readonly SkPoint[] }
>("Patch");
export const Atlas = make<
  DrawingNodeProps & {
    image: SkImage | null;
    sprites: readonly SkRect[];
    transforms: readonly SkRSXform[];
    colors?: readonly Color[];
    colorBlendMode?: SkEnum<typeof BlendMode>;
    sampling?: SamplingOptions;
  }
>("Atlas");
export const Box = make<DrawingNodeProps & { box: InputRRect | SkRect }>("Box");
export const BoxShadow = make<{
  dx?: number;
  dy?: number;
  spread?: number;
  blur: number;
  color?: Color;
  inner?: boolean;
}>("BoxShadow");

// --- Text, images, pictures ---------------------------------------------------

export const Text = make<DrawingNodeProps & { font: SkFont | null; text: string; x?: number; y?: number }>("Text", {
  x: 0,
  y: 0,
});
export const TextPath = make<
  DrawingNodeProps & { font: SkFont | null; text: string; path: PathDef; initialOffset?: number }
>("TextPath", { initialOffset: 0 });
export const TextBlob = make<DrawingNodeProps & { blob: SkTextBlob; x?: number; y?: number }>("TextBlob", {
  x: 0,
  y: 0,
});
export const Glyphs = make<
  DrawingNodeProps & { font: SkFont | null; glyphs: readonly { id: number; pos: SkPoint }[]; x?: number; y?: number }
>("Glyphs", { x: 0, y: 0 });
export const Paragraph = make<
  GroupProps & { paragraph: SkParagraph | null; x?: number; y?: number; width: number }
>("Paragraph", { x: 0, y: 0 });
export const Image = make<
  DrawingNodeProps & RectCtor & { image: SkImage | null; fit?: Fit; sampling?: SamplingOptions }
>("Image", { fit: "contain" });
export const ImageSVG = make<DrawingNodeProps & RectCtor & { svg: SkSvg | null }>("ImageSVG");
export const Picture = make<DrawingNodeProps & { picture: SkPicture }>("Picture");

// --- Shaders ------------------------------------------------------------------

interface GradientProps extends TransformProps {
  colors: readonly Color[];
  positions?: readonly number[];
  mode?: SkEnum<typeof TileMode>;
  flags?: number;
}
interface NoiseProps {
  freqX: number;
  freqY: number;
  octaves: number;
  seed?: number;
  tileWidth?: number;
  tileHeight?: number;
}
const noise = { seed: 0, tileWidth: 0, tileHeight: 0 };

export const Shader = make<TransformProps & ChildrenProps & { source: SkRuntimeEffect; uniforms?: Uniforms }>("Shader", {
  uniforms: {},
});
export const ImageShader = make<
  TransformProps &
    RectCtor & {
      image: SkImage | null;
      tx?: SkEnum<typeof TileMode>;
      ty?: SkEnum<typeof TileMode>;
      fit?: Fit;
      sampling?: SamplingOptions;
    }
>("ImageShader", { tx: "decal", ty: "decal", fit: "none" });
export const ColorShader = make<{ color: Color }>("ColorShader");
export const Turbulence = make<NoiseProps>("Turbulence", noise);
export const FractalNoise = make<NoiseProps>("FractalNoise", noise);
export const LinearGradient = make<GradientProps & { start: Vector; end: Vector }>("LinearGradient");
export const RadialGradient = make<GradientProps & { c: Vector; r: number }>("RadialGradient");
export const SweepGradient = make<GradientProps & { c: Vector; start?: number; end?: number }>("SweepGradient");
export const TwoPointConicalGradient = make<
  GradientProps & { start: Vector; startR: number; end: Vector; endR: number }
>("TwoPointConicalGradient");
/** Shaders, or image filters, blended. */
export const Blend = make<ChildrenProps & { mode: SkEnum<typeof BlendMode> }>("Blend");

// --- Filters and effects ------------------------------------------------------

export const BlurMask = make<{ blur: number; style?: SkEnum<typeof BlurStyle>; respectCTM?: boolean }>("BlurMaskFilter", {
  style: "normal",
  respectCTM: true,
});
export const Blur = make<ChildrenProps & { blur: Radius; mode?: SkEnum<typeof TileMode> }>("BlurImageFilter", {
  mode: "decal",
});
export const Offset = make<ChildrenProps & { x?: number; y?: number }>("OffsetImageFilter", { x: 0, y: 0 });
export const Morphology = make<ChildrenProps & { operator?: "erode" | "dilate"; radius: Radius }>(
  "MorphologyImageFilter",
  { operator: "dilate" },
);
export const Shadow = make<
  ChildrenProps & { dx: number; dy: number; blur: number; color: Color; inner?: boolean; shadowOnly?: boolean }
>("DropShadowImageFilter");
export const DisplacementMap = make<
  ChildrenProps & { channelX: SkEnum<typeof ColorChannel>; channelY: SkEnum<typeof ColorChannel>; scale: number }
>("DisplacementMapImageFilter");
export const RuntimeShader = make<ChildrenProps & { source: SkRuntimeEffect; uniforms?: Uniforms }>(
  "RuntimeShaderImageFilter",
);
export const ImageFilter = make<ChildrenProps & { filter: SkImageFilter }>("ImageFilter");

export const ColorMatrix = make<ChildrenProps & { matrix: readonly number[] }>("MatrixColorFilter");
export const BlendColor = make<ChildrenProps & { mode: SkEnum<typeof BlendMode>; color: Color }>("BlendColorFilter");
export const Lerp = make<ChildrenProps & { t: number }>("LerpColorFilter");
export const LinearToSRGBGamma = make<ChildrenProps>("LinearToSRGBGammaColorFilter");
export const SRGBToLinearGamma = make<ChildrenProps>("SRGBToLinearGammaColorFilter");
export const LumaColorFilter = make<ChildrenProps>("LumaColorFilter");

export const CornerPathEffect = make<ChildrenProps & { r: number }>("CornerPathEffect");
export const DashPathEffect = make<ChildrenProps & { intervals: readonly number[]; phase?: number }>("DashPathEffect");
export const DiscretePathEffect = make<ChildrenProps & { length: number; deviation: number; seed?: number }>(
  "DiscretePathEffect",
  { seed: 0 },
);
export const Path1DPathEffect = make<
  ChildrenProps & { path: PathDef; advance: number; phase: number; style: SkEnum<typeof Path1DEffectStyle> }
>("Path1DPathEffect");
export const Path2DPathEffect = make<ChildrenProps & { matrix: InputMatrix; path: PathDef }>("Path2DPathEffect");
export const Line2DPathEffect = make<ChildrenProps & { matrix: InputMatrix; width: number }>("Line2DPathEffect");
export const SumPathEffect = make<ChildrenProps>("SumPathEffect");

/** An identity color matrix and a few others. */
export const OpacityMatrix = (opacity: number) => [1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, opacity, 0];
