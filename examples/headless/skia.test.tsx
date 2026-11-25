import { signal } from "nsh";
import { expect, open, test } from "nsh/test";
import {
  AlphaType, BackdropBlur, BlendColor, Blur, BlurMask, BlurStyle, Box, BoxShadow, Circle, ColorType, createPicture, DashPathEffect, drawAsImage,
  drawAsPicture, Fill, FillType, FitBox, Group, Image, ImageFormat, ImageSVG, Line, LinearGradient, Mask, matchFont, Paint, Paragraph, Path,
  PathOp, Points, Rect, rect, RoundedRect, rrect, Shader, Shadow, Skia, Sksl, type SkImage, Text, TileMode, vec,
} from "nsh-skia";

const pixel = (image: SkImage, x: number, y: number) => {
  const i = (y * image.width() + x) * 4;
  return Array.from(image.pixels().slice(i, i + 4)).join(",");
};
const size = { width: 100, height: 100 };
const draw = (element: any) => drawAsImage(element, size);
const RED = "255,0,0,255";
const WHITE = "255,255,255,255";
const CLEAR = "0,0,0,0";

test("the imperative API draws", () => {
  const surface = Skia.Surface.MakeOffscreen(100, 100)!;
  const canvas = surface.getCanvas();
  const paint = Skia.Paint();
  paint.setColor(Skia.Color("red"));
  canvas.clear("white");
  canvas.drawCircle(50, 50, 20, paint);
  const image = surface.makeImageSnapshot();
  expect(pixel(image, 50, 50)).toBe(RED);
  expect(pixel(image, 5, 5)).toBe(WHITE);
  const again = Skia.Image.MakeFromEncoded(Skia.Data.fromBytes(image.encode(ImageFormat.PNG)))!;
  expect(pixel(again, 50, 50)).toBe(RED);
  expect(Skia.Image.MakeFromEncoded(Skia.Data.fromBytes(new Uint8Array([1, 2, 3])))).toBe(null);
});

test("colors are react-native-skia's", () => {
  expect(Array.from(Skia.Color("#ff000080")).map((n) => Math.round(n * 255)).join()).toBe("255,0,0,128");
  expect(Array.from(Skia.Color(0xff00ff00)).join()).toBe("0,1,0,1");
  expect(Array.from(Skia.Color([0, 0, 1, 0.5])).join()).toBe("0,0,1,0.5");
  for (const name of ["cyan", "lightblue", "rebeccapurple", "rgba(1, 2, 3, 0.5)", "hsl(120, 100%, 50%)", "transparent"]) Skia.Color(name);
  expect(() => Skia.Color("no color")).toThrow();
});

test("paths: builder, SVG, operations, measure", () => {
  const p = Skia.PathBuilder.Make().moveTo(0, 0).lineTo(10, 0).lineTo(10, 10).close().build();
  expect(p.toSVGString()).toBe("M0 0L10 0L10 10L0 0Z");
  expect(JSON.stringify(p.toCmds())).toBe(JSON.stringify([[0, 0, 0], [1, 10, 0], [1, 10, 10], [1, 0, 0], [5]]));
  expect(Skia.Path.MakeFromSVGString("not a path")).toBe(null);
  const both = Skia.Path.MakeFromOp(Skia.Path.Rect(rect(0, 0, 10, 10)), Skia.Path.Rect(rect(5, 0, 10, 10)), PathOp.Intersect)!;
  expect(JSON.stringify(both.getBounds())).toBe(JSON.stringify(rect(5, 0, 5, 10)));
  const line = Skia.Path.Line(vec(0, 0), vec(100, 0));
  const measure = Skia.ContourMeasureIter(line, false, 1).next()!;
  expect(measure.length()).toBe(100);
  expect(JSON.stringify(measure.getPosTan(25))).toBe(JSON.stringify([vec(25, 0), vec(1, 0)]));
  expect(Skia.Path.Trim(line, 0.25, 0.75, false)!.getBounds().width).toBe(50);
  expect(Skia.Path.Stroke(line, { width: 10 })!.getBounds().height).toBe(10);
  const conic = Skia.PathBuilder.Make().moveTo(0, 0).conicTo(5, 5, 10, 0, 0.5).build();
  expect(JSON.stringify(conic.toCmds()[1])).toBe("[3,5,5,10,0,0.5]");
  expect(Skia.Path.MakeFromCmds(conic.toCmds())!.toSVGString()).toBe(conic.toSVGString());
  const from = Skia.Path.Circle(0, 0, 10);
  const to = Skia.Path.Circle(0, 0, 20);
  expect(Skia.Path.Interpolate(from, to, 0)!.getBounds().width).toBe(20);
  expect(Skia.Path.Interpolate(from, to, 1)!.getBounds().width).toBe(40);
  expect(from.interpolate(line, 0.5)).toBe(null);
});

test("matrices concatenate and read back", () => {
  const m = Skia.Matrix().translate(10, 20).scale(2);
  expect(m.get().join()).toBe("2,0,10,0,2,20,0,0,1");
  expect(Skia.Matrix().concat(m).postTranslate(1, 1).get().join()).toBe("2,0,11,0,2,21,0,0,1");
});

test("SkSL: shaders, uniforms by name, the compiler's errors", () => {
  const source = Sksl`uniform float2 size; uniform float4 tint; half4 main(float2 p) { return half4(tint * (p.x / size.x < 0.5 ? 1 : 0)); }`;
  const image = draw(<Fill><Shader source={source} uniforms={{ size: vec(100, 100), tint: [1, 0, 0, 1] }} /></Fill>);
  expect(pixel(image, 10, 50)).toBe(RED);
  expect(pixel(image, 90, 50)).toBe(CLEAR);
  expect(Skia.RuntimeEffect.Make("half4 main(float2 p) { return nothing; }")).toBe(null);
  expect(() => drawAsPicture(<Fill><Shader source={source} uniforms={{ size: vec(1, 1) }} /></Fill>, size)).toThrow("tint");
});

test("shapes, strokes and groups", () => {
  const image = draw(
    <Group>
      <Fill color="white" />
      <Rect x={0} y={0} width={10} height={10} color="red" />
      <RoundedRect x={20} y={0} width={20} height={20} r={10} color="red" />
      <Line p1={vec(0, 50)} p2={vec(100, 50)} color="red" strokeWidth={4} style="stroke" />
      <Path path="M 60 0 L 80 0 L 80 20 L 60 20 Z" color="red" style="stroke" strokeWidth={4} />
      <Points points={[vec(0, 90), vec(100, 90)]} mode="lines" color="red" style="stroke" strokeWidth={4} />
      <Group transform={[{ translateX: 50 }, { translateY: 70 }, { rotate: Math.PI / 2 }]} color="red">
        <Rect x={0} y={-5} width={10} height={10} />
      </Group>
    </Group>,
  );
  expect(pixel(image, 5, 5)).toBe(RED);
  expect(pixel(image, 20, 0)).toBe(WHITE);
  expect(pixel(image, 30, 10)).toBe(RED);
  expect(pixel(image, 50, 50)).toBe(RED);
  expect(pixel(image, 70, 10)).toBe(WHITE);
  expect(pixel(image, 60, 10)).toBe(RED);
  expect(pixel(image, 30, 90)).toBe(RED);
  expect(pixel(image, 50, 75)).toBe(RED);
  expect(pixel(image, 57, 72)).toBe(WHITE);
});

test("opacity multiplies down, a color replaces a shader", () => {
  const image = draw(
    <Group opacity={0.5}>
      <Group opacity={0.5}><Rect x={0} y={0} width={50} height={100} color="red" /></Group>
      <Group>
        <LinearGradient start={vec(0, 0)} end={vec(100, 0)} colors={["blue", "blue"]} />
        <Rect x={50} y={0} width={25} height={100} />
        <Rect x={75} y={0} width={25} height={100} color="red" />
      </Group>
    </Group>,
  );
  expect(pixel(image, 10, 10).split(",")[3]).toBe("64");
  expect(pixel(image, 60, 10)).toBe("0,0,255,128");
  expect(pixel(image, 90, 10)).toBe("255,0,0,128");
});

test("zIndex orders siblings", () => {
  const image = draw(
    <Group>
      <Group zIndex={1}><Fill color="red" /></Group>
      <Fill color="blue" />
    </Group>,
  );
  expect(pixel(image, 50, 50)).toBe(RED);
});

test("clips: rectangles, rounded rectangles, paths, inverted", () => {
  const of = (clip: any, invertClip = false) => draw(<Group clip={clip} invertClip={invertClip}><Fill color="red" /></Group>);
  expect(pixel(of(rect(0, 0, 50, 50)), 60, 60)).toBe(CLEAR);
  expect(pixel(of(rect(0, 0, 50, 50)), 25, 25)).toBe(RED);
  expect(pixel(of(rrect(rect(0, 0, 100, 100), 50, 50)), 2, 2)).toBe(CLEAR);
  expect(pixel(of("M 0 0 L 100 0 L 0 100 Z"), 90, 90)).toBe(CLEAR);
  expect(pixel(of(rect(0, 0, 50, 50), true), 25, 25)).toBe(CLEAR);
});

test("masks, layers, paints and backdrops", () => {
  const masked = draw(
    <Mask mask={<Circle cx={50} cy={50} r={20} color="black" />}><Fill color="red" /></Mask>,
  );
  expect(pixel(masked, 50, 50)).toBe(RED);
  expect(pixel(masked, 5, 5)).toBe(CLEAR);
  const luma = draw(
    <Mask mode="luminance" mask={<Group><Fill color="black" /><Rect x={0} y={0} width={50} height={100} color="white" /></Group>}>
      <Fill color="red" />
    </Mask>,
  );
  expect(pixel(luma, 25, 50)).toBe(RED);
  expect(pixel(luma, 75, 50)).toBe(CLEAR);
  const layer = draw(<Group layer={<Paint><BlendColor color="red" mode="srcIn" /></Paint>}><Fill color="blue" /></Group>);
  expect(pixel(layer, 50, 50)).toBe(RED);
  const twice = draw(
    <Circle cx={50} cy={50} r={30} color="red"><Paint color="blue" style="stroke" strokeWidth={10} /></Circle>,
  );
  expect(pixel(twice, 50, 50)).toBe(RED);
  expect(pixel(twice, 80, 50)).toBe("0,0,255,255");
  const backdrop = draw(
    <Group>
      <Fill color="white" />
      <Rect x={0} y={0} width={50} height={100} color="black" />
      <BackdropBlur blur={10} clip={rect(0, 0, 100, 50)} />
    </Group>,
  );
  const gray = Number(pixel(backdrop, 50, 25).split(",")[0]);
  expect(gray > 60 && gray < 200).toBe(true);
  expect(pixel(backdrop, 30, 75)).toBe("0,0,0,255");
  expect(pixel(backdrop, 70, 75)).toBe(WHITE);
});

test("filters and effects", () => {
  const blurred = draw(<Rect x={25} y={25} width={50} height={50} color="red"><Blur blur={5} /></Rect>);
  const edge = Number(pixel(blurred, 25, 50).split(",")[3]);
  expect(edge > 80 && edge < 180).toBe(true);
  const mask = draw(<Rect x={25} y={25} width={50} height={50} color="red"><BlurMask blur={5} style="solid" /></Rect>);
  expect(pixel(mask, 26, 50)).toBe(RED);
  expect(Number(pixel(mask, 20, 50).split(",")[3]) > 0).toBe(true);
  const shadow = draw(<Rect x={0} y={0} width={50} height={50} color="red"><Shadow dx={40} dy={40} blur={0} color="blue" /></Rect>);
  expect(pixel(shadow, 80, 80)).toBe("0,0,255,255");
  expect(pixel(shadow, 25, 25)).toBe(RED);
  const inner = draw(<Rect x={0} y={0} width={100} height={100} color="white"><Shadow inner dx={20} dy={0} blur={1} color="red" /></Rect>);
  expect(pixel(inner, 5, 50)).toBe(RED);
  expect(pixel(inner, 60, 50)).toBe(WHITE);
  const dashed = draw(<Line p1={vec(0, 50)} p2={vec(100, 50)} color="red" style="stroke" strokeWidth={10}><DashPathEffect intervals={[20, 20]} /></Line>);
  expect(pixel(dashed, 10, 50)).toBe(RED);
  expect(pixel(dashed, 30, 50)).toBe(CLEAR);
  const box = draw(
    <Box box={rrect(rect(25, 25, 50, 50), 5, 5)} color="white"><BoxShadow dx={20} dy={20} blur={0.1} color="red" /></Box>,
  );
  expect(pixel(box, 85, 85)).toBe(RED);
  expect(pixel(box, 50, 50)).toBe(WHITE);
});

test("images: pixels, fits, SVG documents, pictures", () => {
  const data = Skia.Data.fromBytes(new Uint8Array([255, 0, 0, 255, 0, 0, 255, 255]));
  const info = { width: 2, height: 1, colorType: ColorType.RGBA_8888, alphaType: AlphaType.Unpremul };
  const image = Skia.Image.MakeRaster(info, data, 8)!;
  const contained = draw(<Image image={image} x={0} y={0} width={100} height={100} sampling={{ filter: 0 }} />);
  expect(pixel(contained, 25, 50)).toBe(RED);
  expect(pixel(contained, 75, 50)).toBe("0,0,255,255");
  expect(pixel(contained, 25, 10)).toBe(CLEAR);
  const covered = draw(<Image image={image} fit="cover" x={0} y={0} width={100} height={100} sampling={{ filter: 0 }} />);
  expect(pixel(covered, 25, 10)).toBe(RED);
  const svg = Skia.SVG.MakeFromString(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="red"/></svg>`)!;
  expect(pixel(draw(<ImageSVG svg={svg} x={50} y={50} width={50} height={50} />), 95, 95)).toBe(RED);
  const picture = createPicture((c) => { const p = Skia.Paint(); p.setColor("red"); c.drawRect(rect(0, 0, 100, 100), p); });
  const surface = Skia.Surface.MakeOffscreen(10, 10)!;
  surface.getCanvas().drawPicture(picture);
  expect(pixel(surface.makeImageSnapshot(), 5, 5)).toBe(RED);
  const boxed = draw(<FitBox src={rect(0, 0, 10, 10)} dst={rect(0, 0, 100, 100)}><Rect x={5} y={5} width={5} height={5} color="red" /></FitBox>);
  expect(pixel(boxed, 75, 75)).toBe(RED);
  expect(pixel(boxed, 25, 25)).toBe(CLEAR);
});

test("text and paragraphs draw with the system's fonts", () => {
  const ink = (image: SkImage) => image.pixels().filter((_, i) => i % 4 === 3).reduce((n, a) => n + (a > 0 ? 1 : 0), 0);
  const font = matchFont({ fontSize: 30, fontWeight: "bold" });
  expect(font.getTextWidth("Skia") > 20).toBe(true);
  expect(ink(draw(<Text x={5} y={50} text="Skia" font={font} color="black" />)) > 100).toBe(true);
  expect(ink(draw(<Text x={5} y={50} text="Skia" font={null} color="black" />))).toBe(0);
  const paragraph = Skia.ParagraphBuilder.Make({ maxLines: 2, ellipsis: "…" })
    .pushStyle({ fontSize: 20, color: "red" })
    .addText("one two three four five six seven")
    .pop()
    .build();
  expect(ink(draw(<Paragraph paragraph={paragraph} x={0} y={0} width={90} />)) > 100).toBe(true);
  expect(paragraph.getLineMetrics().length).toBe(2);
  expect(paragraph.getLongestLine() <= 90).toBe(true);
});

test("a canvas of a finished recording refuses to draw", () => {
  const recorder = Skia.PictureRecorder();
  const canvas = recorder.beginRecording();
  recorder.finishRecordingAsPicture();
  expect(() => canvas.clear("red")).toThrow("finished");
  expect(() => recorder.finishRecordingAsPicture()).toThrow("nothing");
});

test("a node that cannot be drawn is named, and its siblings draw", () => {
  const bad: any = <Group><Circle cx={0} cy={0} {...({ r: "big" } as any)} color="red" /></Group>;
  expect(() => drawAsPicture(bad, size)).toThrow("<Circle>: `r` must be a number");
  const image = draw(<Group><Rect {...({} as any)} color="red" /><Circle cx={50} cy={50} r={10} color="red" /></Group>);
  expect(pixel(image, 50, 50)).toBe(RED);
  expect(() => draw(<Circle cx={0} cy={0} r={1} color="no color" />)).toThrow("color");
});

test("accessors are props", () => {
  const r = signal(10);
  expect(pixel(draw(<Circle cx={50} cy={50} r={r} color="red" />), 50, 50)).toBe(RED);
});

test("a <Canvas> lays out, follows its signals, and snapshots", async () => {
  const app = await open("./App.tsx");
  await app.waitUntil(() => app.texts().some((t) => t.startsWith("100x100")));
  const info = () => app.texts().find((t) => t.startsWith("100x100"));
  expect(info()).toBe(`100x100 50,50=${RED} 85,50=${WHITE} 10,10=${WHITE} only=${RED} many=6000`);
  await app.press("text:Grow");
  await app.waitUntil(() => info()!.includes(`85,50=${RED}`));
  await app.press("text:Show");
  await app.waitUntil(() => info()!.includes("10,10=0,0,255,255"));
  expect(info()!.split(" only=")[1]).toBe("0,0,255,255 many=400");
});

// Used by the types above only.
void [BlurStyle, FillType, TileMode];
