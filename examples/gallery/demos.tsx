// What the gallery draws.

import { computed } from "nsh";
import {
  BackdropBlur, Blur, BlurMask, Box, BoxShadow, Circle, DashPathEffect, Fill, Group, ImageShader, ImageSVG, LinearGradient, Mask, matchFont, Paragraph,
  Path, rect, Rect, Repeat, RoundedRect, rrect, Shader, Shadow, Skia, Sksl, SweepGradient, Text, TextAlign, TextPath, useClock, vec,
} from "nsh-skia";

/** A demo's width and height. */
export const S = 220;
const clock = useClock();
const t = computed(() => clock() / 1000);

const plasma = Sksl`
uniform float2 size; uniform float time;
half4 main(float2 p) {
  float2 uv = p / size * 3;
  float v = sin(uv.x + time) + sin(uv.y + time * 0.7) + sin(uv.x + uv.y + time * 0.5);
  return half4(0.5 + 0.5 * cos(v * 2 + float3(0, 2, 4)), 1);
}`;

const Plasma = () => (
  <Fill><Shader source={plasma} uniforms={{ size: vec(S, S), time: t() }} /></Fill>
);

// react-native-skia's Breathe example.
const Breathe = () => (
  <Group blendMode="screen">
    <Fill color="rgb(36, 43, 56)" />
    <Group origin={vec(S / 2, S / 2)} transform={[{ rotate: -Math.PI * (0.5 + 0.5 * Math.sin(t())) }]}>
      <BlurMask style="solid" blur={20} />
      <Each of={[0, 1, 2, 3, 4, 5]}>
        {(i: any) => {
          const at = computed(() => {
            const p = 0.5 + 0.5 * Math.sin(t());
            const a = (i() * Math.PI) / 3;
            return vec(S / 2 + Math.cos(a) * 40 * p, S / 2 + Math.sin(a) * 40 * p);
          });
          return <Circle c={at()} r={40} color={i() % 2 ? "#529ca0" : "#61bea2"} />;
        }}
      </Each>
    </Group>
  </Group>
);

const heart = "M 110 190 C 20 120 40 40 110 80 C 180 40 200 120 110 190 Z";
const Paths = () => (
  <Group>
    <Fill color="#fff7ed" />
    <Path path={heart} color="#fecaca" />
    <Path path={heart} color="#dc2626" style="stroke" strokeWidth={8} strokeCap="round" strokeJoin="round" start={0} end={0.5 + 0.5 * Math.sin(t() * 1.5)} />
    <Circle cx={110} cy={110} r={100} color="#fb923c" style="stroke" strokeWidth={4}>
      <DashPathEffect intervals={[12, 8]} phase={t() * 20} />
    </Circle>
  </Group>
);

const Glass = () => (
  <Group>
    <Fill><LinearGradient start={vec(0, 0)} end={vec(S, S)} colors={["#0ea5e9", "#a855f7"]} /></Fill>
    <Circle cx={S / 2 + 60 * Math.cos(t())} cy={S / 2 + 60 * Math.sin(t())} r={40} color="#fde047" />
    <BackdropBlur blur={8} clip={rrect(rect(30, 60, 160, 100), 20, 20)}>
      <Fill color="rgba(255, 255, 255, 0.25)" />
    </BackdropBlur>
    <Text x={58} y={118} text="Glass" font={matchFont({ fontSize: 36, fontWeight: "bold" })} color="white" />
  </Group>
);

const paragraph = Skia.ParagraphBuilder.Make({ textAlign: TextAlign.Center })
  .pushStyle({ fontSize: 22, color: "#111827" })
  .addText("Paragraphs with ")
  .pushStyle({ fontSize: 22, color: "#2563eb", fontStyle: { weight: 700 } })
  .addText("bold")
  .pop()
  .addText(" runs, العربية, 日本語 and 🎨 in one layout.")
  .build();
const small = matchFont({ fontSize: 15 });
const ring = Skia.PathBuilder.Make().addCircle(110, 160, 32).build();
const Words = () => (
  <Group>
    <Fill color="white" />
    <Paragraph paragraph={paragraph} x={10} y={8} width={S - 20} />
    <TextPath font={small} path={ring} text="text along a path that turns  " initialOffset={(t() * 20) % 20} color="#be185d" />
  </Group>
);

const Cards = () => (
  <Group>
    <Fill color="#e5e7eb" />
    <Box box={rrect(rect(30, 30, 160, 70), 16, 16)} color="#f3f4f6">
      <BoxShadow dx={8} dy={8} blur={10} color="#9ca3af" />
      <BoxShadow dx={-8} dy={-8} blur={10} color="white" />
    </Box>
    <RoundedRect x={30} y={130} width={70} height={60} r={14} color="white"><Shadow dx={0} dy={6} blur={6} color="#6b7280" /></RoundedRect>
    <RoundedRect x={120} y={130} width={70} height={60} r={14} color="#f9fafb"><Shadow inner dx={3} dy={3} blur={4} color="#6b7280" /></RoundedRect>
  </Group>
);

const svg = Skia.SVG.MakeFromString(
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><linearGradient id="g" x2="1" y2="1"><stop stop-color="#f43f5e"/><stop offset="1" stop-color="#f59e0b"/></linearGradient></defs><rect width="100" height="100" rx="20" fill="url(#g)"/><path d="M50 20 L80 75 L20 75 Z" fill="none" stroke="white" stroke-width="8" stroke-linejoin="round"/></svg>`,
);
const checks = (() => {
  const surface = Skia.Surface.MakeRaster(20, 20)!;
  const c = surface.getCanvas();
  const p = Skia.Paint();
  c.clear("#c7d2fe");
  p.setColor("#4338ca");
  c.drawRect(rect(0, 0, 10, 10), p);
  c.drawRect(rect(10, 10, 10, 10), p);
  return surface.makeImageSnapshot();
})();
const Images = () => (
  <Group>
    <Fill><ImageShader image={checks} tx="repeat" ty="repeat" transform={[{ translateX: t() * 10 }]} /></Fill>
    <ImageSVG svg={svg} x={20} y={20} width={80} height={80} />
    <Mask mask={<Circle cx={155} cy={155} r={45 + 8 * Math.sin(t() * 2)} color="black"><Blur blur={3} /></Circle>}>
      <Rect x={100} y={100} width={120} height={120}>
        <SweepGradient c={vec(155, 155)} colors={["red", "yellow", "lime", "cyan", "blue", "magenta", "red"]} />
      </Rect>
    </Mask>
  </Group>
);

const COUNT = 500;
const dots = Array.from({ length: COUNT }, (_, i) => ({ a: i * 2.399963, d: Math.sqrt(i / COUNT) * 100, h: (i * 360) / COUNT }));
const Dots = () => (
  <Group transform={[{ translateX: S / 2 }, { translateY: S / 2 }, { rotate: t() * 0.3 }]}>
    <Repeat of={dots}>
      {(dot: any) => (
        <Circle
          cx={Math.cos(dot().a) * dot().d * (0.9 + 0.1 * Math.sin(t() * 3 + dot().d / 10))}
          cy={Math.sin(dot().a) * dot().d * (0.9 + 0.1 * Math.sin(t() * 3 + dot().d / 10))}
          r={3}
          color={`hsl(${Math.round(dot().h)}, 80%, 55%)`}
        />
      )}
    </Repeat>
  </Group>
);

export const DEMOS: [string, () => any][] = [
  ["SkSL shader", Plasma], ["Breathe", Breathe], ["Paths", Paths], ["Backdrop blur", Glass],
  ["Text", Words], ["Shadows", Cards], ["Images, SVG, mask", Images], [`${COUNT} moving circles`, Dots],
];

